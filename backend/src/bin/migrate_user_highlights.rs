//! One-shot tool: copy highlight objects from one user namespace to another.
//!
//! Usage (from `backend/`, same env as the server: `S3_DATA_BUCKET`, `GCP_PROJECT_ID`, AWS creds):
//!   cargo run --bin migrate_user_highlights -- --from <source-user-id> --to <target-user-id> [--dry-run]

use anyhow::{Context, Result, bail};
use dastill::config::TursoRuntimeConfig;
use dastill::db::{HighlightMigrationStats, init_store, migrate_user_highlights};
use dastill::local_env::load_dotenv_preserving_existing;

struct Args {
    from_user_id: String,
    to_user_id: String,
    dry_run: bool,
}

fn parse_args() -> Result<Args> {
    let mut from_user_id: Option<String> = None;
    let mut to_user_id: Option<String> = None;
    let mut dry_run = false;
    let mut args = std::env::args().skip(1);

    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--from" => {
                from_user_id = Some(args.next().context("--from requires a source user id")?);
            }
            "--to" => {
                to_user_id = Some(args.next().context("--to requires a target user id")?);
            }
            "--dry-run" => dry_run = true,
            "--help" | "-h" => {
                println!(
                    "Usage: cargo run --bin migrate_user_highlights -- --from <source-user-id> --to <target-user-id> [--dry-run]"
                );
                std::process::exit(0);
            }
            other => bail!("unexpected argument: {other}"),
        }
    }

    let from_user_id = from_user_id.context("missing --from <source-user-id>")?;
    let to_user_id = to_user_id.context("missing --to <target-user-id>")?;

    if from_user_id == to_user_id {
        bail!("source and target user ids must differ");
    }

    Ok(Args {
        from_user_id,
        to_user_id,
        dry_run,
    })
}

fn print_stats(prefix: &str, stats: HighlightMigrationStats) {
    println!(
        "{prefix} scanned={} copied={} skipped_duplicates={} remapped_ids={}",
        stats.scanned, stats.copied, stats.skipped_duplicates, stats.remapped_ids
    );
}

#[tokio::main]
async fn main() -> Result<()> {
    rustls::crypto::aws_lc_rs::default_provider()
        .install_default()
        .expect("failed to install rustls crypto provider");

    load_dotenv_preserving_existing();

    let args = parse_args()?;

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

    let turso_db = libsql::Builder::new_remote_replica(
        std::env::temp_dir().join("dastill-bin.db"),
        turso.db_url,
        turso.auth_token,
    )
    .build()
    .await
    .map_err(|e| anyhow::anyhow!("Turso: {e}"))?;
    turso_db.sync().await.map_err(|e| anyhow::anyhow!("Turso sync: {e}"))?;
    let turso_conn = turso_db.connect().map_err(|e| anyhow::anyhow!("Turso connect: {e}"))?;
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

    let stats = migrate_user_highlights(&store, &args.from_user_id, &args.to_user_id, args.dry_run)
        .await
        .map_err(|e| anyhow::anyhow!("{e}"))?;

    print_stats(if args.dry_run { "dry-run:" } else { "done:" }, stats);
    Ok(())
}
