//! One-shot tool: find or reset poisoned transcripts that contain the known
//! summarize HTML-cache fallback blurb.
//!
//! Usage (from `backend/`, same env as the server):
//!   cargo run --bin purge_bad_transcripts -- --video-id nDU7Mn-XRWI --video-id nsqGI1VAYbU

use anyhow::{Context, Result, bail};
use dastill::config::TursoRuntimeConfig;
use dastill::db::{self, init_store};
use dastill::local_env::load_dotenv_preserving_existing;
use dastill::models::ContentStatus;

const BAD_SNIPPET: &str = "Sup nerds we got things to discuss.";

#[tokio::main]
async fn main() -> Result<()> {
    rustls::crypto::ring::default_provider()
        .install_default()
        .expect("failed to install rustls crypto provider");

    load_dotenv_preserving_existing();

    let args: Vec<String> = std::env::args().skip(1).collect();
    if args.is_empty() {
        bail!("pass --scan or one or more --video-id <id> arguments");
    }

    let dry_run = args.iter().any(|arg| arg == "--dry-run");
    let video_ids = parse_video_ids(&args)?;

    if video_ids.is_empty() {
        bail!("no video IDs supplied");
    }

    let store = connect_store().await?;

    for video_id in video_ids {
        let transcript = db::get_transcript(&store, &video_id)
            .await
            .with_context(|| format!("load transcript for {video_id}"))?;
        let is_bad = transcript
            .as_ref()
            .is_some_and(transcript_contains_bad_snippet);

        if !is_bad {
            println!("skip: {video_id} does not currently contain the poisoned transcript snippet");
            continue;
        }

        if dry_run {
            println!("dry-run: would reset {video_id}");
            continue;
        }

        db::delete_transcript(&store, &video_id)
            .await
            .with_context(|| format!("delete transcript for {video_id}"))?;
        db::delete_summary(&store, &video_id)
            .await
            .with_context(|| format!("delete summary for {video_id}"))?;
        db::reset_summary_auto_regen_attempts(&store, &video_id)
            .await
            .with_context(|| format!("reset auto-regeneration metadata for {video_id}"))?;
        db::update_video_transcript_status(&store, &video_id, ContentStatus::Pending)
            .await
            .with_context(|| format!("set transcript status pending for {video_id}"))?;
        db::update_video_summary_status(&store, &video_id, ContentStatus::Pending)
            .await
            .with_context(|| format!("set summary status pending for {video_id}"))?;

        println!("reset: {video_id}");
    }

    Ok(())
}

fn parse_video_ids(args: &[String]) -> Result<Vec<String>> {
    let mut video_ids = Vec::new();
    let mut i = 0usize;
    while i < args.len() {
        if args[i] == "--video-id" {
            let Some(video_id) = args.get(i + 1) else {
                bail!("missing value after --video-id");
            };
            video_ids.push(video_id.clone());
            i += 2;
            continue;
        }
        i += 1;
    }
    Ok(video_ids)
}

async fn connect_store() -> Result<db::Store> {
    let data_bucket = std::env::var("S3_DATA_BUCKET").context("S3_DATA_BUCKET must be set")?;
    let vector_bucket =
        std::env::var("S3_VECTOR_BUCKET").context("S3_VECTOR_BUCKET must be set")?;
    let vector_index =
        std::env::var("S3_VECTOR_INDEX").unwrap_or_else(|_| "search-chunks".to_string());
    let aws_region = std::env::var("AWS_REGION").unwrap_or_else(|_| "eu-central-1".to_string());
    let turso = TursoRuntimeConfig::from_env()
        .map_err(|e| anyhow::anyhow!(e))?
        .context("TURSO_DB_URL and TURSO_AUTH_TOKEN must be set")?;

    let aws_config = dastill::aws_auth::load_aws_sdk_config(aws_region)
        .await
        .map_err(|err| anyhow::anyhow!(err))?;

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

    init_store(
        s3_client,
        s3v_client,
        turso_conn,
        data_bucket,
        vector_bucket,
        vector_index,
        dastill::read_cache::ReadCache::default(),
    )
    .await
    .map_err(|err| anyhow::anyhow!(err))
}
fn transcript_contains_bad_snippet(transcript: &dastill::models::Transcript) -> bool {
    transcript
        .raw_text
        .as_deref()
        .is_some_and(|text| text.contains(BAD_SNIPPET))
        || transcript
            .formatted_markdown
            .as_deref()
            .is_some_and(|text| text.contains(BAD_SNIPPET))
}
