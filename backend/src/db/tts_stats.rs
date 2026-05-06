use libsql::{Value, params};
use serde::{Deserialize, Serialize};

use super::{Store, StoreError};

const GLOBAL_DOC_ID: &str = "global";

fn tts_stats_storage_key() -> &'static str {
    "tts-stats/global.json"
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TtsGenerationStats {
    pub sample_count: u32,
    pub total_words: u64,
    pub total_duration_secs: f64,
}

impl TtsGenerationStats {
    fn words_per_sec(&self) -> Option<f64> {
        if self.total_duration_secs > 0.0 && self.total_words > 0 {
            Some(self.total_words as f64 / self.total_duration_secs)
        } else {
            None
        }
    }

    /// Estimated synthesis duration in seconds for the given word count.
    pub fn estimate_secs(&self, word_count: u32) -> Option<f32> {
        self.words_per_sec()
            .map(|wps| (word_count as f64 / wps) as f32)
    }
}

pub async fn get_tts_stats(store: &Store) -> Result<Option<TtsGenerationStats>, StoreError> {
    let mut rows = store
        .sql
        .query(
            "SELECT sample_count, total_words, total_duration_secs FROM tts_stats WHERE id = ?1",
            params![GLOBAL_DOC_ID],
        )
        .await?;

    match rows.next().await? {
        Some(row) => {
            let sample_count: i64 = row.get(0)?;
            let total_words: i64 = row.get(1)?;
            let total_duration_secs: f64 = match row.get_value(2)? {
                Value::Real(v) => v,
                Value::Integer(v) => v as f64,
                _ => 0.0,
            };
            Ok(Some(TtsGenerationStats {
                sample_count: sample_count.max(0) as u32,
                total_words: total_words.max(0) as u64,
                total_duration_secs,
            }))
        }
        None => Ok(None),
    }
}

pub async fn has_sql_tts_stats(store: &Store) -> Result<bool, StoreError> {
    Ok(get_tts_stats(store).await?.is_some())
}

pub async fn has_snapshot_tts_stats(store: &Store) -> Result<bool, StoreError> {
    store.key_exists(tts_stats_storage_key()).await
}

/// Append a completed generation sample to the running aggregate.
/// Atomic upsert — no read-then-write needed with SQL.
pub async fn record_tts_generation(
    store: &Store,
    word_count: u32,
    duration_secs: f64,
) -> Result<(), StoreError> {
    store
        .sql
        .execute(
            r#"INSERT INTO tts_stats (id, sample_count, total_words, total_duration_secs)
               VALUES (?1, 1, ?2, ?3)
               ON CONFLICT(id) DO UPDATE SET
                 sample_count = tts_stats.sample_count + 1,
                 total_words = tts_stats.total_words + excluded.total_words,
                 total_duration_secs = tts_stats.total_duration_secs + excluded.total_duration_secs"#,
            params![GLOBAL_DOC_ID, word_count as i64, duration_secs],
        )
        .await?;
    if let Some(stats) = get_tts_stats(store).await? {
        store.put_json(tts_stats_storage_key(), &stats).await?;
        store
            .record_libsql_snapshot_delta(vec![super::LibsqlSnapshotDeltaOperation::PutTtsStats {
                stats,
            }])
            .await?;
    }
    Ok(())
}

pub async fn bootstrap_sql_tts_stats_from_store(store: &Store) -> Result<bool, StoreError> {
    let Some(stats) = store
        .get_json::<TtsGenerationStats>(tts_stats_storage_key())
        .await?
    else {
        return Ok(false);
    };

    store
        .sql
        .execute(
            r#"INSERT INTO tts_stats (id, sample_count, total_words, total_duration_secs)
               VALUES (?1, ?2, ?3, ?4)
               ON CONFLICT(id) DO UPDATE SET
                 sample_count = excluded.sample_count,
                 total_words = excluded.total_words,
                 total_duration_secs = excluded.total_duration_secs"#,
            params![
                GLOBAL_DOC_ID,
                stats.sample_count as i64,
                stats.total_words as i64,
                stats.total_duration_secs
            ],
        )
        .await?;
    Ok(true)
}

pub async fn export_sql_tts_stats_to_store(store: &Store) -> Result<bool, StoreError> {
    let Some(stats) = get_tts_stats(store).await? else {
        return Ok(false);
    };
    store.put_json(tts_stats_storage_key(), &stats).await?;
    store
        .record_libsql_snapshot_delta(vec![super::LibsqlSnapshotDeltaOperation::PutTtsStats {
            stats,
        }])
        .await?;
    Ok(true)
}
