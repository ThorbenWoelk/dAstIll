//! One-shot tool: set `earliest_sync_date` to 2026-01-01 UTC and `earliest_sync_date_user_set`
//! on every user-channel subscription in the data bucket (see `Store::set_all_user_channel_earliest_sync_dates`).
//!
//! Usage (from `backend/`, same env as the server: `S3_DATA_BUCKET`, `GCP_PROJECT_ID`, AWS creds):
//!   cargo run --bin set_earliest_sync_dates -- [--dry-run]

use anyhow::{Context, Result};
use chrono::{TimeZone, Utc};
use dastill::config::TursoRuntimeConfig;
use dastill::db::init_store;
use dastill::local_env::load_dotenv_preserving_existing;

#[tokio::main]
async fn main() -> Result<()> {
    rustls::crypto::ring::default_provider()
        .install_default()
        .expect("failed to install rustls crypto provider");

    load_dotenv_preserving_existing();

    let dry_run = std::env::args().any(|a| a == "--dry-run");

    let data_bucket = std::env::var("S3_DATA_BUCKET").context("S3_DATA_BUCKET must be set")?;
    let vector_bucket =
        std::env::var("S3_VECTOR_BUCKET").context("S3_VECTOR_BUCKET must be set")?;
    let vector_index =
        std::env::var("S3_VECTOR_INDEX").unwrap_or_else(|_| "search-chunks".to_string());
    let aws_region = std::env::var("AWS_REGION").unwrap_or_else(|_| "eu-central-1".to_string());
    let turso = TursoRuntimeConfig::from_env()
        .map_err(|e| anyhow::anyhow!(e))?
        .context("TURSO_DB_URL and TURSO_AUTH_TOKEN must be set")?;

    let aws_config = dastill::aws_auth::load_aws_sdk_config(aws_region.clone())
        .await
        .map_err(|e| anyhow::anyhow!(e))?;

    let mut s3_config_builder = aws_sdk_s3::config::Builder::from(&aws_config);
    if let Ok(endpoint) = std::env::var("S3_ENDPOINT_URL") {
        s3_config_builder = s3_config_builder
            .endpoint_url(endpoint)
            .force_path_style(true);
    }
    let s3_client = aws_sdk_s3::Client::from_conf(s3_config_builder.build());

    let mut s3v_config_builder = aws_sdk_s3vectors::config::Builder::from(&aws_config);
    if let Ok(endpoint) = std::env::var("S3_VECTOR_ENDPOINT_URL") {
        s3v_config_builder = s3v_config_builder.endpoint_url(endpoint);
    }
    let s3v_client = aws_sdk_s3vectors::Client::from_conf(s3v_config_builder.build());

    let turso_db = libsql::Builder::new_remote(turso.db_url, turso.auth_token)
        .build()
        .await
        .map_err(|e| anyhow::anyhow!("Turso: {e}"))?;
    let turso_conn = turso_db
        .connect()
        .map_err(|e| anyhow::anyhow!("Turso connect: {e}"))?;
    dastill::db::turso_schema::initialize_turso_schema(&turso_conn)
        .await
        .map_err(|e| anyhow::anyhow!(e))?;

    let store = init_store(
        s3_client,
        s3v_client,
        turso_conn,
        data_bucket,
        vector_bucket,
        vector_index,
        dastill::read_cache::ReadCache::default(),
    )
    .await
    .map_err(|e| anyhow::anyhow!(e))?;

    let target = Utc
        .with_ymd_and_hms(2026, 1, 1, 0, 0, 0)
        .single()
        .context("invalid target date")?;

    let n = store
        .set_all_user_channel_earliest_sync_dates(target, dry_run)
        .await
        .map_err(|e| anyhow::anyhow!("{e}"))?;

    println!(
        "{} {} subscription(s)",
        if dry_run { "dry-run:" } else { "done:" },
        n
    );
    Ok(())
}
