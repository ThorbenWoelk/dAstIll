use std::{
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    },
    time::Duration,
};

use chrono::Utc;
use reqwest::Client;
use serde::Serialize;
use serde_json::Value;
use sha2::{Digest, Sha256};
use tokio::sync::{Mutex, mpsc};

use crate::config::DatabricksRuntimeConfig;

const STATEMENT_POLL_INTERVAL_MS: u64 = 500;
const STATEMENT_MAX_POLLS: usize = 600;
const INGEST_QUEUE_CAPACITY: usize = 256;
const INGEST_RETRY_DELAYS_MS: [u64; 5] = [1_000, 5_000, 15_000, 30_000, 60_000];
/// When Databricks returns quota, rate limits, or warehouse capacity errors, avoid tight retry loops.
const INGEST_QUOTA_RETRY_DELAYS_MS: [u64; 6] = [
    900_000,    // 15 min
    1_800_000,  // 30 min
    3_600_000,  // 1 h
    7_200_000,  // 2 h
    14_400_000, // 4 h
    28_800_000, // 8 h
];
/// Databricks SQL warehouses reject statement requests with more than 256 named parameters (HTTP 400).
/// See https://github.com/databricks/databricks-sql-nodejs/issues/308
const MAX_NAMED_PARAMETERS_PER_STATEMENT: usize = 256;

#[derive(Clone)]
pub struct DatabricksSqlService {
    client: Client,
    config: DatabricksRuntimeConfig,
    initialized: Arc<Mutex<bool>>,
    disabled: Arc<AtomicBool>,
    sender: mpsc::Sender<Vec<Value>>,
}

impl DatabricksSqlService {
    pub fn new(client: Client, config: DatabricksRuntimeConfig) -> Self {
        let (sender, receiver) = mpsc::channel(INGEST_QUEUE_CAPACITY);
        let service = Self {
            client,
            config,
            initialized: Arc::new(Mutex::new(false)),
            disabled: Arc::new(AtomicBool::new(false)),
            sender,
        };
        service.spawn_worker(receiver);
        service
    }

    pub fn enqueue_events(&self, events: Vec<Value>) -> Result<(), DatabricksSqlError> {
        self.sender.try_send(events).map_err(|err| match err {
            mpsc::error::TrySendError::Full(_) => DatabricksSqlError::QueueFull,
            mpsc::error::TrySendError::Closed(_) => DatabricksSqlError::QueueClosed,
        })
    }

    async fn ingest_events(&self, events: &[Value]) -> Result<(), DatabricksSqlError> {
        if self.disabled.load(Ordering::Relaxed) {
            return Err(DatabricksSqlError::PermanentlyDisabled);
        }
        self.ensure_table_ready().await?;

        let rows = build_insert_rows(events)?;
        if rows.is_empty() {
            return Ok(());
        }

        let table = self.full_table_name();
        let mut start = 0usize;
        while start < rows.len() {
            let mut param_count = 0usize;
            let mut end = start;
            while end < rows.len() {
                let row_params = row_named_param_count(&rows[end]);
                if param_count + row_params > MAX_NAMED_PARAMETERS_PER_STATEMENT {
                    break;
                }
                param_count += row_params;
                end += 1;
            }

            if end == start {
                // Safety: one row uses at most nine parameters, well under the server limit.
                end = start + 1;
            }

            let chunk = &rows[start..end];
            let statement = build_insert_statement(&table, chunk);
            let parameters = build_insert_parameters(chunk);
            self.execute_statement("insert_events", &statement, parameters)
                .await?;
            start = end;
        }

        Ok(())
    }

    fn spawn_worker(&self, mut receiver: mpsc::Receiver<Vec<Value>>) {
        let service = self.clone();
        tokio::spawn(async move {
            while let Some(batch) = receiver.recv().await {
                service.process_batch(batch).await;
            }
        });
    }

    async fn process_batch(&self, batch: Vec<Value>) {
        let mut attempt = 0usize;

        loop {
            match self.ingest_events(&batch).await {
                Ok(()) => return,
                Err(error) => {
                    if is_permanent_configuration_error(&error) {
                        self.disabled.store(true, Ordering::Relaxed);
                        tracing::warn!(
                            error = %error,
                            "analytics Databricks ingest disabled due to permanent configuration error"
                        );
                        return;
                    }

                    if let Some(delay_ms) = retry_delay_ms(attempt, &error) {
                        let quota_backoff = should_use_quota_style_backoff(&error);
                        tracing::warn!(
                            error = %error,
                            attempt = attempt + 1,
                            retry_in_ms = delay_ms,
                            quota_style_backoff = quota_backoff,
                            "analytics Databricks ingest failed; retrying in background"
                        );
                        tokio::time::sleep(Duration::from_millis(delay_ms)).await;
                        attempt += 1;
                        continue;
                    }

                    tracing::warn!(
                        error = %error,
                        attempts = attempt + 1,
                        "analytics Databricks ingest failed permanently"
                    );
                    return;
                }
            }
        }
    }

    async fn ensure_table_ready(&self) -> Result<(), DatabricksSqlError> {
        let mut initialized = self.initialized.lock().await;
        if *initialized {
            return Ok(());
        }

        let schema_stmt = format!(
            "CREATE SCHEMA IF NOT EXISTS {}.{}",
            quote_ident(&self.config.catalog),
            quote_ident(&self.config.schema)
        );
        self.execute_statement("create_schema", &schema_stmt, Vec::new())
            .await?;

        let create_table_stmt = format!(
            "CREATE TABLE IF NOT EXISTS {} (
                received_at TIMESTAMP NOT NULL,
                event_id STRING NOT NULL,
                event_time STRING,
                event_name STRING,
                session_id STRING,
                channel_id STRING,
                video_id STRING,
                summary_id STRING,
                raw_json STRING NOT NULL
            ) USING DELTA",
            self.full_table_name()
        );
        self.execute_statement("create_table", &create_table_stmt, Vec::new())
            .await?;

        *initialized = true;
        Ok(())
    }

    async fn execute_statement(
        &self,
        operation: &'static str,
        statement: &str,
        parameters: Vec<StatementParameter>,
    ) -> Result<(), DatabricksSqlError> {
        let url = format!(
            "{}/api/2.0/sql/statements",
            self.config.host.trim_end_matches('/')
        );
        let parameter_count = parameters.len();
        let response = self
            .client
            .post(url)
            .bearer_auth(&self.config.token)
            .json(&ExecuteStatementRequest {
                warehouse_id: &self.config.warehouse_id,
                catalog: Some(self.config.catalog.as_str()),
                schema: Some(self.config.schema.as_str()),
                statement,
                wait_timeout: "0s",
                on_wait_timeout: "CONTINUE",
                parameters,
            })
            .send()
            .await
            .map_err(DatabricksSqlError::Http)?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            tracing::warn!(
                operation,
                parameter_count,
                statement_len = statement.len(),
                status = %status,
                body = %body,
                "databricks executeStatement HTTP error"
            );
            return Err(DatabricksSqlError::ApiStatus { status, body });
        }

        let mut body: StatementResponse =
            response.json().await.map_err(DatabricksSqlError::Http)?;

        for _ in 0..STATEMENT_MAX_POLLS {
            match body.status.state.as_str() {
                "SUCCEEDED" => return Ok(()),
                "PENDING" | "RUNNING" => {
                    let statement_id = body
                        .statement_id
                        .clone()
                        .ok_or(DatabricksSqlError::MissingStatementId)?;
                    tokio::time::sleep(Duration::from_millis(STATEMENT_POLL_INTERVAL_MS)).await;
                    body = self.fetch_statement(&statement_id).await?;
                }
                other => {
                    return Err(DatabricksSqlError::StatementFailed {
                        state: other.to_string(),
                        message: body.status.error_message(),
                    });
                }
            }
        }

        Err(DatabricksSqlError::StatementTimedOut)
    }

    async fn fetch_statement(
        &self,
        statement_id: &str,
    ) -> Result<StatementResponse, DatabricksSqlError> {
        let url = format!(
            "{}/api/2.0/sql/statements/{}",
            self.config.host.trim_end_matches('/'),
            statement_id
        );
        let response = self
            .client
            .get(url)
            .bearer_auth(&self.config.token)
            .send()
            .await
            .map_err(DatabricksSqlError::Http)?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            return Err(DatabricksSqlError::ApiStatus { status, body });
        }

        response.json().await.map_err(DatabricksSqlError::Http)
    }

    fn full_table_name(&self) -> String {
        format!(
            "{}.{}.{}",
            quote_ident(&self.config.catalog),
            quote_ident(&self.config.schema),
            quote_ident(&self.config.bronze_table)
        )
    }
}

#[derive(Debug, thiserror::Error)]
pub enum DatabricksSqlError {
    #[error("failed to serialize analytics events: {0}")]
    Serialization(#[from] serde_json::Error),
    #[error("databricks http error: {0}")]
    Http(#[from] reqwest::Error),
    #[error("databricks api returned {status}: {body}")]
    ApiStatus {
        status: reqwest::StatusCode,
        body: String,
    },
    #[error("statement execution did not return a statement id")]
    MissingStatementId,
    #[error("statement execution timed out while waiting for completion")]
    StatementTimedOut,
    #[error("statement execution failed in state {state}: {message}")]
    StatementFailed { state: String, message: String },
    #[error("analytics ingest queue is full")]
    QueueFull,
    #[error("analytics ingest queue is closed")]
    QueueClosed,
    #[error("analytics ingest disabled due to permanent Databricks configuration error")]
    PermanentlyDisabled,
}

#[derive(Debug, Clone)]
struct AnalyticsInsertRow {
    received_at: String,
    event_id: String,
    event_time: Option<String>,
    event_name: Option<String>,
    session_id: Option<String>,
    channel_id: Option<String>,
    video_id: Option<String>,
    summary_id: Option<String>,
    raw_json: String,
}

#[derive(Debug, Serialize)]
struct ExecuteStatementRequest<'a> {
    warehouse_id: &'a str,
    #[serde(skip_serializing_if = "Option::is_none")]
    catalog: Option<&'a str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    schema: Option<&'a str>,
    statement: &'a str,
    wait_timeout: &'a str,
    on_wait_timeout: &'a str,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    parameters: Vec<StatementParameter>,
}

#[derive(Debug, Clone, Serialize)]
struct StatementParameter {
    name: String,
    value: String,
    #[serde(rename = "type")]
    type_name: String,
}

#[derive(Debug, serde::Deserialize)]
struct StatementResponse {
    statement_id: Option<String>,
    status: StatementStatus,
}

#[derive(Debug, serde::Deserialize)]
struct StatementStatus {
    state: String,
    error: Option<StatementError>,
    message: Option<String>,
}

impl StatementStatus {
    fn error_message(&self) -> String {
        self.error
            .as_ref()
            .and_then(|error| error.message.clone())
            .or_else(|| self.message.clone())
            .unwrap_or_else(|| "no error details returned".to_string())
    }
}

#[derive(Debug, serde::Deserialize)]
struct StatementError {
    message: Option<String>,
}

fn build_insert_rows(events: &[Value]) -> Result<Vec<AnalyticsInsertRow>, serde_json::Error> {
    let received_at = Utc::now().format("%Y-%m-%d %H:%M:%S%.3f").to_string();
    events
        .iter()
        .map(|event| {
            let raw_json = serde_json::to_string(event)?;
            Ok(AnalyticsInsertRow {
                received_at: received_at.clone(),
                event_id: extract_string(event, "event_id")
                    .unwrap_or_else(|| stable_event_id(&raw_json)),
                event_time: extract_string(event, "ts"),
                event_name: extract_string(event, "event"),
                session_id: extract_string(event, "session_id"),
                channel_id: extract_string(event, "channel_id"),
                video_id: extract_string(event, "video_id"),
                summary_id: extract_string(event, "summary_id"),
                raw_json,
            })
        })
        .collect()
}

fn build_insert_statement(table_name: &str, rows: &[AnalyticsInsertRow]) -> String {
    let values = rows
        .iter()
        .enumerate()
        .map(|(index, row)| {
            format!(
                "(:received_at_{n}, :event_id_{n}, {event_time}, {event_name}, {session_id}, {channel_id}, {video_id}, {summary_id}, :raw_json_{n})",
                n = index,
                event_time = maybe_placeholder("event_time", index, row.event_time.as_ref()),
                event_name = maybe_placeholder("event_name", index, row.event_name.as_ref()),
                session_id = maybe_placeholder("session_id", index, row.session_id.as_ref()),
                channel_id = maybe_placeholder("channel_id", index, row.channel_id.as_ref()),
                video_id = maybe_placeholder("video_id", index, row.video_id.as_ref()),
                summary_id = maybe_placeholder("summary_id", index, row.summary_id.as_ref()),
            )
        })
        .collect::<Vec<_>>()
        .join(", ");

    format!(
        "INSERT INTO {table_name} (received_at, event_id, event_time, event_name, session_id, channel_id, video_id, summary_id, raw_json) VALUES {values}"
    )
}

fn build_insert_parameters(rows: &[AnalyticsInsertRow]) -> Vec<StatementParameter> {
    let mut params = Vec::with_capacity(rows.len() * 8);
    for (index, row) in rows.iter().enumerate() {
        params.push(statement_param(
            format!("received_at_{index}"),
            row.received_at.clone(),
            "TIMESTAMP",
        ));
        params.push(statement_param(
            format!("event_id_{index}"),
            row.event_id.clone(),
            "STRING",
        ));
        push_optional_param(
            &mut params,
            "event_time",
            index,
            row.event_time.as_ref(),
            "STRING",
        );
        push_optional_param(
            &mut params,
            "event_name",
            index,
            row.event_name.as_ref(),
            "STRING",
        );
        push_optional_param(
            &mut params,
            "session_id",
            index,
            row.session_id.as_ref(),
            "STRING",
        );
        push_optional_param(
            &mut params,
            "channel_id",
            index,
            row.channel_id.as_ref(),
            "STRING",
        );
        push_optional_param(
            &mut params,
            "video_id",
            index,
            row.video_id.as_ref(),
            "STRING",
        );
        push_optional_param(
            &mut params,
            "summary_id",
            index,
            row.summary_id.as_ref(),
            "STRING",
        );
        params.push(statement_param(
            format!("raw_json_{index}"),
            row.raw_json.clone(),
            "STRING",
        ));
    }
    params
}

fn statement_param(name: String, value: String, type_name: &str) -> StatementParameter {
    StatementParameter {
        name,
        value,
        type_name: type_name.to_string(),
    }
}

fn push_optional_param(
    params: &mut Vec<StatementParameter>,
    prefix: &str,
    index: usize,
    value: Option<&String>,
    type_name: &str,
) {
    if let Some(value) = value {
        params.push(statement_param(
            format!("{prefix}_{index}"),
            value.clone(),
            type_name,
        ));
    }
}

fn maybe_placeholder(prefix: &str, index: usize, value: Option<&String>) -> String {
    if value.is_some() {
        format!(":{prefix}_{index}")
    } else {
        "NULL".to_string()
    }
}

fn extract_string(event: &Value, key: &str) -> Option<String> {
    event
        .get(key)
        .and_then(Value::as_str)
        .map(ToString::to_string)
}

fn stable_event_id(raw_json: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(raw_json.as_bytes());
    format!("{:x}", hasher.finalize())
}

fn quote_ident(value: &str) -> String {
    format!("`{}`", value.replace('`', "``"))
}

/// Count of named parameters Databricks will receive for one insert row (NULL optionals omit params).
fn row_named_param_count(row: &AnalyticsInsertRow) -> usize {
    let mut n = 3usize;
    if row.event_time.is_some() {
        n += 1;
    }
    if row.event_name.is_some() {
        n += 1;
    }
    if row.session_id.is_some() {
        n += 1;
    }
    if row.channel_id.is_some() {
        n += 1;
    }
    if row.video_id.is_some() {
        n += 1;
    }
    if row.summary_id.is_some() {
        n += 1;
    }
    n
}

/// HTTP 429, documented quota error classes, or warehouse messages that imply no quick recovery.
fn should_use_quota_style_backoff(error: &DatabricksSqlError) -> bool {
    match error {
        DatabricksSqlError::ApiStatus { status, body } => {
            api_response_suggests_quota_style_backoff(*status, body)
        }
        DatabricksSqlError::StatementFailed { message, .. } => {
            message_suggests_quota_style_backoff(message)
        }
        _ => false,
    }
}

fn api_response_suggests_quota_style_backoff(status: reqwest::StatusCode, body: &str) -> bool {
    if status == reqwest::StatusCode::TOO_MANY_REQUESTS {
        return true;
    }
    message_suggests_quota_style_backoff(body)
}

fn message_suggests_quota_style_backoff(text: &str) -> bool {
    let compact: String = text.chars().filter(|c| !c.is_whitespace()).collect();
    let lower = compact.to_ascii_lowercase();

    if lower.contains("api_quota_exceeded") || lower.contains("dc_api_quota_exceeded") {
        return true;
    }
    if lower.contains("resource_exhausted") {
        return true;
    }

    let haystack = text.to_ascii_lowercase();
    if haystack.contains("too many requests") || haystack.contains("rate limit") {
        return true;
    }
    if haystack.contains("throttl") {
        return true;
    }
    if haystack.contains("quota") && (haystack.contains("exceed") || haystack.contains("limit")) {
        return true;
    }
    if haystack.contains("daily") && haystack.contains("limit") {
        return true;
    }

    // Warehouse / serverless capacity (400 BAD_REQUEST); retrying every few seconds wastes API calls.
    if haystack.contains("no longer eligible") && haystack.contains("serverless") {
        return true;
    }

    false
}

fn retry_delay_ms(attempt: usize, error: &DatabricksSqlError) -> Option<u64> {
    if should_use_quota_style_backoff(error) {
        if attempt >= INGEST_QUOTA_RETRY_DELAYS_MS.len() {
            return None;
        }
        return Some(INGEST_QUOTA_RETRY_DELAYS_MS[attempt]);
    }

    if attempt >= INGEST_RETRY_DELAYS_MS.len() {
        return None;
    }

    match error {
        DatabricksSqlError::Http(_)
        | DatabricksSqlError::ApiStatus { .. }
        | DatabricksSqlError::StatementTimedOut
        | DatabricksSqlError::StatementFailed { .. } => Some(INGEST_RETRY_DELAYS_MS[attempt]),
        DatabricksSqlError::Serialization(_)
        | DatabricksSqlError::MissingStatementId
        | DatabricksSqlError::QueueFull
        | DatabricksSqlError::QueueClosed
        | DatabricksSqlError::PermanentlyDisabled => None,
    }
}

fn is_permanent_configuration_error(error: &DatabricksSqlError) -> bool {
    match error {
        DatabricksSqlError::ApiStatus { status, body } => {
            if *status != reqwest::StatusCode::BAD_REQUEST {
                return false;
            }
            let haystack = body.to_ascii_lowercase();
            haystack.contains("cannot start warehouse")
                && haystack.contains("serverless compute")
                && (haystack.contains("disabled in global warehouse config")
                    || haystack.contains("contact your administrator"))
        }
        DatabricksSqlError::StatementFailed { message, .. } => {
            let haystack = message.to_ascii_lowercase();
            haystack.contains("cannot start warehouse")
                && haystack.contains("serverless compute")
                && (haystack.contains("disabled in global warehouse config")
                    || haystack.contains("contact your administrator"))
        }
        _ => false,
    }
}

#[cfg(test)]
mod tests {
    use serde_json::Value;

    use reqwest::StatusCode;

    use super::{
        MAX_NAMED_PARAMETERS_PER_STATEMENT, build_insert_parameters, build_insert_rows,
        build_insert_statement, is_permanent_configuration_error, retry_delay_ms,
        row_named_param_count, stable_event_id,
    };

    #[test]
    fn retry_delay_uses_long_backoff_on_http_429() {
        let err = super::DatabricksSqlError::ApiStatus {
            status: StatusCode::TOO_MANY_REQUESTS,
            body: "{}".to_string(),
        };
        assert_eq!(retry_delay_ms(0, &err), Some(900_000));
        assert_eq!(retry_delay_ms(5, &err), Some(28_800_000));
        assert_eq!(retry_delay_ms(6, &err), None);
    }

    #[test]
    fn retry_delay_uses_long_backoff_on_api_quota_body() {
        let err = super::DatabricksSqlError::ApiStatus {
            status: StatusCode::BAD_REQUEST,
            body: r#"{"error_code":"API_QUOTA_EXCEEDED","message":"limit"}"#.to_string(),
        };
        assert_eq!(retry_delay_ms(0, &err), Some(900_000));
    }

    #[test]
    fn retry_delay_uses_long_backoff_on_serverless_ineligible_message() {
        let err = super::DatabricksSqlError::ApiStatus {
            status: StatusCode::BAD_REQUEST,
            body: r#"{"message":"Cannot start warehouse since workspace is no longer eligible for Serverless Compute"}"#
                .to_string(),
        };
        assert_eq!(retry_delay_ms(0, &err), Some(900_000));
    }

    #[test]
    fn retry_delay_uses_short_backoff_on_unclassified_400() {
        let err = super::DatabricksSqlError::ApiStatus {
            status: StatusCode::BAD_REQUEST,
            body: r#"{"error_code":"BAD_REQUEST","message":"Invalid parameter"}"#.to_string(),
        };
        assert_eq!(retry_delay_ms(0, &err), Some(1_000));
    }

    #[test]
    fn classifies_serverless_disabled_error_as_permanent_configuration_issue() {
        let err = super::DatabricksSqlError::ApiStatus {
            status: StatusCode::BAD_REQUEST,
            body: r#"{"error_code":"BAD_REQUEST","message":"Cannot start warehouse 'Serverless Starter Warehouse' with Serverless Compute since it is disabled in global warehouse config. To use the warehouse, please contact your administrator."}"#.to_string(),
        };
        assert!(is_permanent_configuration_error(&err));
    }

    #[test]
    fn build_insert_rows_derives_stable_event_id_when_missing() {
        let rows = build_insert_rows(&[serde_json::json!({
            "event": "summary_opened",
            "ts": "2026-03-24T10:11:12Z",
            "session_id": "session-1",
            "video_id": "video-1",
            "channel_id": "channel-1",
            "summary_id": "summary-1"
        })])
        .expect("rows");

        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0].event_id, stable_event_id(&rows[0].raw_json));
    }

    #[test]
    fn build_insert_statement_uses_null_for_missing_optional_fields() {
        let rows = build_insert_rows(&[serde_json::json!({
            "event": "channel_snapshot_loaded"
        })])
        .expect("rows");

        let statement = build_insert_statement("`workspace`.`sandbox`.`bronze_app_events`", &rows);
        assert!(statement.contains("INSERT INTO `workspace`.`sandbox`.`bronze_app_events`"));
        assert!(statement.contains("NULL"));
    }

    #[test]
    fn build_insert_parameters_skips_null_optionals() {
        let rows = build_insert_rows(&[serde_json::json!({
            "event": "video_opened",
            "session_id": "session-1",
            "video_id": "video-1",
            "channel_id": "channel-1"
        })])
        .expect("rows");

        let params = build_insert_parameters(&rows);
        assert!(params.iter().any(|param| param.name == "video_id_0"));
        assert!(!params.iter().any(|param| param.name == "summary_id_0"));
    }

    #[test]
    fn row_named_param_count_matches_insert_parameters_len() {
        let rows = build_insert_rows(&[serde_json::json!({
            "event": "summary_opened",
            "ts": "2026-03-24T10:11:12Z",
            "session_id": "session-1",
            "video_id": "video-1",
            "channel_id": "channel-1",
            "summary_id": "summary-1"
        })])
        .expect("rows");
        assert_eq!(
            row_named_param_count(&rows[0]),
            build_insert_parameters(&rows).len()
        );
    }

    #[test]
    fn insert_chunking_keeps_each_statement_under_named_parameter_cap() {
        let events: Vec<Value> = (0..40)
            .map(|i| {
                serde_json::json!({
                    "event": "video_opened",
                    "ts": "2026-01-01T00:00:00Z",
                    "session_id": format!("session-{i}"),
                    "channel_id": "channel-1",
                    "video_id": "video-1",
                    "summary_id": "summary-1"
                })
            })
            .collect();
        let rows = build_insert_rows(&events).expect("rows");

        let mut start = 0usize;
        while start < rows.len() {
            let mut param_count = 0usize;
            let mut end = start;
            while end < rows.len() {
                let row_params = row_named_param_count(&rows[end]);
                if param_count + row_params > MAX_NAMED_PARAMETERS_PER_STATEMENT {
                    break;
                }
                param_count += row_params;
                end += 1;
            }
            if end == start {
                end = start + 1;
            }
            let chunk = &rows[start..end];
            let n = build_insert_parameters(chunk).len();
            assert!(
                n <= MAX_NAMED_PARAMETERS_PER_STATEMENT,
                "chunk would exceed Databricks cap: {n} > {MAX_NAMED_PARAMETERS_PER_STATEMENT}"
            );
            start = end;
        }
    }
}
