use std::path::{Path, PathBuf};
use std::time::Duration;

use crate::read_cache::ReadCache;
use crate::runtime_paths::local_libsql_dir;

use super::{
    LibsqlSnapshotPublisher, LibsqlSourceGenerationTracker, Store, init_store,
    publish_libsql_snapshot, reconcile_sql_cache_with_store, replay_libsql_snapshot_deltas,
    reset_local_libsql_cache, restore_libsql_snapshot,
};

pub struct LocalLibsqlStore {
    pub database: libsql::Database,
    pub store: Store,
    pub read_cache: ReadCache,
    pub fts_dir: PathBuf,
    pub shared_db_path: Option<PathBuf>,
}

pub async fn initialize_local_libsql(
    db_path: &Path,
) -> anyhow::Result<(libsql::Database, libsql::Connection, Option<PathBuf>)> {
    tracing::info!(path = %db_path.display(), "Initializing local libSQL database...");
    let db = libsql::Builder::new_local(db_path)
        .build()
        .await
        .map_err(|err| anyhow::anyhow!("failed to build local libSQL database: {err}"))?;
    let conn = db
        .connect()
        .map_err(|err| anyhow::anyhow!("failed to connect to local libSQL: {err}"))?;
    conn.busy_timeout(Duration::from_secs(5))
        .map_err(|err| anyhow::anyhow!("failed to set local libSQL busy timeout: {err}"))?;
    super::sql_schema::initialize_sql_schema(&conn)
        .await
        .map_err(|err| anyhow::anyhow!(err))?;
    Ok((db, conn, Some(db_path.to_path_buf())))
}

pub async fn initialize_local_libsql_store(
    s3_client: aws_sdk_s3::Client,
    s3v_client: aws_sdk_s3vectors::Client,
    data_bucket: String,
    vector_bucket: String,
    vector_index: String,
    port: u16,
) -> anyhow::Result<LocalLibsqlStore> {
    let fts_dir = local_libsql_dir(&std::env::temp_dir(), port);
    std::fs::create_dir_all(&fts_dir)?;
    let db_path = fts_dir.join("search-fts.db");
    let snapshot_restore = restore_libsql_snapshot(&s3_client, &data_bucket, &db_path).await;
    let (database, conn, shared_db_path) = initialize_local_libsql(&db_path).await?;
    let snapshot_conn = conn.clone();
    let mut replayed_delta_generations = 0usize;
    let mut snapshot_replay_failed = false;

    if let Some((base_generation, target_generation)) = snapshot_restore.replay_range() {
        match replay_libsql_snapshot_deltas(
            &s3_client,
            &data_bucket,
            &snapshot_conn,
            base_generation,
            target_generation,
        )
        .await
        {
            Ok(applied) => {
                replayed_delta_generations = applied;
                tracing::info!(
                    base_generation,
                    target_generation,
                    replayed_delta_generations,
                    "libSQL snapshot delta replay complete"
                );
            }
            Err(err) => {
                snapshot_replay_failed = true;
                tracing::warn!(
                    error = %err,
                    base_generation,
                    target_generation,
                    "libSQL snapshot delta replay failed - clearing local SQL cache for S3 rebuild"
                );
                reset_local_libsql_cache(&snapshot_conn)
                    .await
                    .map_err(|reset_err| anyhow::anyhow!(reset_err))?;
            }
        }
    }

    let source_generation_tracker =
        LibsqlSourceGenerationTracker::new(s3_client.clone(), data_bucket.clone())
            .await
            .map_err(|err| anyhow::anyhow!(err))?;
    let snapshot_publisher = LibsqlSnapshotPublisher::new(
        s3_client.clone(),
        data_bucket.clone(),
        snapshot_conn.clone(),
        db_path.clone(),
    );

    let read_cache = ReadCache::default();
    let store = init_store(
        s3_client.clone(),
        s3v_client,
        conn,
        data_bucket.clone(),
        vector_bucket,
        vector_index,
        read_cache.clone(),
        Some(source_generation_tracker),
        Some(snapshot_publisher),
    )
    .await
    .map_err(|err| anyhow::anyhow!(err))?;

    let cache_reconcile = reconcile_sql_cache_with_store(&store)
        .await
        .map_err(|err| anyhow::anyhow!("failed to reconcile SQL cache from S3: {err}"))?;
    tracing::info!(
        bootstrapped_videos = cache_reconcile.bootstrapped_videos,
        exported_videos = cache_reconcile.exported_videos,
        reconciled_videos = cache_reconcile.reconciled_videos,
        pruned_videos = cache_reconcile.pruned_videos,
        bootstrapped_preferences = cache_reconcile.bootstrapped_preferences,
        exported_preferences = cache_reconcile.exported_preferences,
        reconciled_preferences = cache_reconcile.reconciled_preferences,
        pruned_preferences = cache_reconcile.pruned_preferences,
        bootstrapped_tts_stats = cache_reconcile.bootstrapped_tts_stats,
        exported_tts_stats = cache_reconcile.exported_tts_stats,
        snapshot_restored = snapshot_restore.restored(),
        replayed_delta_generations,
        snapshot_replay_failed,
        "SQL cache reconciliation complete"
    );
    let cache_reconcile_changed = cache_reconcile.bootstrapped_videos > 0
        || cache_reconcile.exported_videos > 0
        || cache_reconcile.reconciled_videos > 0
        || cache_reconcile.pruned_videos > 0
        || cache_reconcile.bootstrapped_preferences > 0
        || cache_reconcile.exported_preferences > 0
        || cache_reconcile.reconciled_preferences > 0
        || cache_reconcile.pruned_preferences > 0
        || cache_reconcile.bootstrapped_tts_stats
        || cache_reconcile.exported_tts_stats;

    if !snapshot_restore.restored() || cache_reconcile_changed || replayed_delta_generations > 0 {
        if let Err(err) = publish_libsql_snapshot(
            &s3_client,
            &data_bucket,
            &snapshot_conn,
            &db_path,
            env!("CARGO_PKG_VERSION"),
        )
        .await
        {
            tracing::warn!(error = %err, "failed to publish libSQL snapshot");
        }
    } else {
        tracing::info!(
            "libSQL snapshot publish skipped - restored snapshot required no reconciliation changes"
        );
    }

    Ok(LocalLibsqlStore {
        database,
        store,
        read_cache,
        fts_dir,
        shared_db_path,
    })
}
