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
