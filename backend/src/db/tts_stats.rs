use serde::{Deserialize, Serialize};

use super::{Store, StoreError};

const COLLECTION: &str = "dastill_tts_stats";
const GLOBAL_DOC_ID: &str = "global";

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
    let stats: Option<TtsGenerationStats> = store
        .firestore
        .fluent()
        .select()
        .by_id_in(COLLECTION)
        .obj()
        .one(GLOBAL_DOC_ID)
        .await?;
    Ok(stats)
}

/// Append a completed generation sample to the running aggregate.
/// Uses read-then-write; safe because TTS generation is low-concurrency.
pub async fn record_tts_generation(
    store: &Store,
    word_count: u32,
    duration_secs: f64,
) -> Result<(), StoreError> {
    let existing = get_tts_stats(store).await?;
    let updated = match existing {
        Some(stats) => TtsGenerationStats {
            sample_count: stats.sample_count.saturating_add(1),
            total_words: stats.total_words.saturating_add(u64::from(word_count)),
            total_duration_secs: stats.total_duration_secs + duration_secs,
        },
        None => TtsGenerationStats {
            sample_count: 1,
            total_words: u64::from(word_count),
            total_duration_secs: duration_secs,
        },
    };
    store
        .firestore
        .fluent()
        .update()
        .in_col(COLLECTION)
        .document_id(GLOBAL_DOC_ID)
        .object(&updated)
        .execute::<TtsGenerationStats>()
        .await?;
    Ok(())
}
