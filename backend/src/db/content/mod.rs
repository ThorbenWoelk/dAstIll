use crate::models::{
    ContentStatus, Summary, SummaryEvaluationJob, Transcript, TranscriptRenderMode,
};
use crate::search::{SearchSourceKind, hash_search_content};

use super::{Store, StoreError};

fn summary_audio_key(video_id: &str, audio_hash: &str, ext: &str) -> String {
    format!("summary-audio/{video_id}/{audio_hash}.{ext}")
}

fn transcript_key(video_id: &str) -> String {
    format!("transcripts/{video_id}.json")
}

fn summary_key(video_id: &str) -> String {
    format!("summaries/{video_id}.json")
}

fn auto_regen_attempts_key(video_id: &str) -> String {
    format!("meta/auto-regen-attempts/{video_id}")
}

fn evaluation_failures_key(video_id: &str) -> String {
    format!("meta/summary-eval-failures/{video_id}")
}

fn summary_before_regeneration_key(video_id: &str) -> String {
    format!("meta/summary-before-regen/{video_id}")
}

/// `model_used` value for summaries a user wrote or edited by hand.
pub(crate) const MANUAL_SUMMARY_MODEL: &str = "manual";

pub(crate) fn is_manual_summary(summary: &Summary) -> bool {
    summary.model_used.as_deref() == Some(MANUAL_SUMMARY_MODEL)
}

pub async fn get_summary_audio(store: &Store, key: &str) -> Result<Option<Vec<u8>>, StoreError> {
    store.get_bytes(key).await
}

pub async fn summary_audio_exists(store: &Store, key: &str) -> Result<bool, StoreError> {
    store.key_exists(key).await
}

pub async fn put_summary_audio(
    store: &Store,
    key: &str,
    bytes: &[u8],
    content_type: &str,
) -> Result<(), StoreError> {
    store.put_bytes(key, bytes, content_type).await
}

pub fn summary_audio_cache_key(video_id: &str, audio_hash: &str, ext: &str) -> String {
    summary_audio_key(video_id, audio_hash, ext)
}

pub async fn upsert_transcript(store: &Store, transcript: &Transcript) -> Result<(), StoreError> {
    store
        .put_json(&transcript_key(&transcript.video_id), transcript)
        .await
}

pub async fn get_transcript(
    store: &Store,
    video_id: &str,
) -> Result<Option<Transcript>, StoreError> {
    store.get_json(&transcript_key(video_id)).await
}

fn transcript_with_render_mode(
    video_id: &str,
    content: &str,
    render_mode: TranscriptRenderMode,
    existing: Option<Transcript>,
) -> Transcript {
    match render_mode {
        TranscriptRenderMode::PlainText => Transcript {
            video_id: video_id.to_string(),
            raw_text: Some(content.to_string()),
            formatted_markdown: None,
            render_mode,
            timed_text: None,
        },
        TranscriptRenderMode::Markdown => {
            let raw_text = existing
                .as_ref()
                .and_then(|t| t.raw_text.clone())
                .filter(|v| !v.trim().is_empty())
                .or_else(|| {
                    existing
                        .as_ref()
                        .and_then(|t| t.formatted_markdown.clone())
                        .filter(|v| !v.trim().is_empty())
                })
                .or_else(|| Some(content.to_string()));

            Transcript {
                video_id: video_id.to_string(),
                raw_text,
                formatted_markdown: Some(content.to_string()),
                render_mode,
                timed_text: None,
            }
        }
    }
}

pub async fn save_manual_transcript(
    store: &Store,
    video_id: &str,
    content: &str,
    render_mode: TranscriptRenderMode,
) -> Result<Transcript, StoreError> {
    let existing = get_transcript(store, video_id).await?;
    let transcript = transcript_with_render_mode(video_id, content, render_mode, existing);
    upsert_transcript(store, &transcript).await?;
    super::videos::update_video_transcript_status(store, video_id, ContentStatus::Ready).await?;
    Ok(transcript)
}

pub async fn upsert_summary(store: &Store, summary: &Summary) -> Result<(), StoreError> {
    store
        .put_json(&summary_key(&summary.video_id), summary)
        .await
}

pub async fn get_summary(store: &Store, video_id: &str) -> Result<Option<Summary>, StoreError> {
    store.get_json(&summary_key(video_id)).await
}

fn apply_summary_quality_update(
    summary: &mut Summary,
    quality_score: Option<u8>,
    quality_note: Option<&str>,
    quality_model_used: Option<&str>,
    summary_tags: Option<&[String]>,
) {
    summary.quality_score = quality_score;
    summary.quality_note = quality_note.map(ToOwned::to_owned);
    summary.quality_model_used = quality_model_used.map(ToOwned::to_owned);
    if let Some(tags) = summary_tags {
        summary.summary_tags = tags.to_vec();
        summary.summary_tags_evaluated = true;
    }
}

pub(crate) fn summary_needs_quality_eval(summary: &Summary) -> bool {
    let has_quality = summary.quality_score.is_some() || summary.quality_note.is_some();
    !has_quality || !summary.summary_tags_evaluated
}

pub async fn save_manual_summary(
    store: &Store,
    video_id: &str,
    content: &str,
    model_used: Option<&str>,
) -> Result<Summary, StoreError> {
    let summary = Summary {
        video_id: video_id.to_string(),
        content: content.to_string(),
        model_used: model_used.map(ToOwned::to_owned),
        quality_score: None,
        quality_note: None,
        quality_model_used: None,
        summary_tags: Vec::new(),
        summary_tags_evaluated: false,
    };
    store.put_json(&summary_key(video_id), &summary).await?;
    // A hand-written summary replaces any automatic regeneration in progress.
    reset_summary_auto_regen_attempts(store, video_id).await?;
    clear_summary_evaluation_failures(store, video_id).await?;
    discard_summary_before_regeneration(store, video_id).await?;
    super::videos::update_video_summary_status(store, video_id, ContentStatus::Ready).await?;
    Ok(summary)
}

/// Stores an evaluation on the summary, but only when the stored summary is still
/// the text that was evaluated. Returns `false` (and changes nothing) when the
/// summary is gone or was replaced while the evaluator was running.
pub async fn update_summary_quality(
    store: &Store,
    video_id: &str,
    evaluated_content: &str,
    quality_score: Option<u8>,
    quality_note: Option<&str>,
    quality_model_used: Option<&str>,
    summary_tags: Option<&[String]>,
) -> Result<bool, StoreError> {
    let key = summary_key(video_id);
    let Some(mut summary) = store.get_json::<Summary>(&key).await? else {
        return Ok(false);
    };
    if summary.content != evaluated_content {
        return Ok(false);
    }
    apply_summary_quality_update(
        &mut summary,
        quality_score,
        quality_note,
        quality_model_used,
        summary_tags,
    );
    store.put_json(&key, &summary).await?;
    Ok(true)
}

const EVALUATION_FAILURES_PREFIX: &str = "meta/summary-eval-failures/";

#[derive(serde::Serialize, serde::Deserialize, Default)]
struct EvaluationFailures {
    #[serde(default)]
    video_id: String,
    /// Hash of the summary text that failed. A new summary starts a new count.
    summary_hash: String,
    /// Every failed attempt. Used to put failing summaries behind the others.
    #[serde(default)]
    failed_attempts: u8,
    /// Attempts where the model answered but the answer was unusable. These
    /// count toward the limit after which the summary is marked unscorable.
    #[serde(default)]
    unusable_answers: u8,
}

/// What happened after an evaluation attempt failed.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SummaryEvaluationFailureOutcome {
    /// The evaluation will be tried again on a later scan.
    WillRetry { failed_attempts: u8 },
    /// The limit was reached. The summary now carries an "unscorable" note so the
    /// scan stops picking it.
    GaveUp { unusable_answers: u8 },
    /// The summary changed while the evaluator ran. Nothing was counted.
    SummaryChanged,
}

/// How the evaluation attempt failed.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SummaryEvaluationFailureKind {
    /// The model answered, but the answer could not be used. Counts toward the limit.
    UnusableAnswer,
    /// The request itself failed (timeout, HTTP error). Retried without a limit.
    RequestFailed,
}

async fn get_evaluation_failures(
    store: &Store,
    video_id: &str,
    summary_content: &str,
) -> Result<EvaluationFailures, StoreError> {
    let meta: Option<EvaluationFailures> =
        store.get_json(&evaluation_failures_key(video_id)).await?;
    Ok(meta
        .filter(|meta| meta.summary_hash == hash_search_content(summary_content))
        .unwrap_or_default())
}

/// Failed evaluation attempts for the summary text currently stored.
pub async fn get_summary_evaluation_failures(
    store: &Store,
    video_id: &str,
    summary_content: &str,
) -> Result<u8, StoreError> {
    Ok(get_evaluation_failures(store, video_id, summary_content)
        .await?
        .failed_attempts)
}

/// Failed attempt counts by video id and summary hash, read in one listing.
async fn load_evaluation_failures(
    store: &Store,
) -> Result<std::collections::HashMap<String, EvaluationFailures>, StoreError> {
    let entries: Vec<EvaluationFailures> = store.load_all(EVALUATION_FAILURES_PREFIX).await?;
    Ok(entries
        .into_iter()
        .filter(|entry| !entry.video_id.is_empty())
        .map(|entry| (entry.video_id.clone(), entry))
        .collect())
}

pub async fn record_summary_evaluation_failure(
    store: &Store,
    video_id: &str,
    evaluated_content: &str,
    error: &str,
    kind: SummaryEvaluationFailureKind,
    max_unusable_answers: u8,
) -> Result<SummaryEvaluationFailureOutcome, StoreError> {
    let current = get_summary(store, video_id).await?;
    if current.as_ref().map(|summary| summary.content.as_str()) != Some(evaluated_content) {
        return Ok(SummaryEvaluationFailureOutcome::SummaryChanged);
    }

    let mut meta = get_evaluation_failures(store, video_id, evaluated_content).await?;
    meta.video_id = video_id.to_string();
    meta.summary_hash = hash_search_content(evaluated_content);
    meta.failed_attempts = meta.failed_attempts.saturating_add(1);
    if kind == SummaryEvaluationFailureKind::UnusableAnswer {
        meta.unusable_answers = meta.unusable_answers.saturating_add(1);
    }

    if meta.unusable_answers < max_unusable_answers {
        store
            .put_json(&evaluation_failures_key(video_id), &meta)
            .await?;
        return Ok(SummaryEvaluationFailureOutcome::WillRetry {
            failed_attempts: meta.failed_attempts,
        });
    }

    let note = format!(
        "**Unscorable**:\n- The evaluator gave {} unusable answers, so this summary was not scored. Last error: {}",
        meta.unusable_answers,
        error.trim()
    );
    let stored = update_summary_quality(
        store,
        video_id,
        evaluated_content,
        None,
        Some(&note),
        None,
        Some(&[]),
    )
    .await?;
    clear_summary_evaluation_failures(store, video_id).await?;
    if !stored {
        return Ok(SummaryEvaluationFailureOutcome::SummaryChanged);
    }
    Ok(SummaryEvaluationFailureOutcome::GaveUp {
        unusable_answers: meta.unusable_answers,
    })
}

pub async fn clear_summary_evaluation_failures(
    store: &Store,
    video_id: &str,
) -> Result<(), StoreError> {
    store
        .delete_key(&evaluation_failures_key(video_id))
        .await
        .ok();
    Ok(())
}

/// Keeps a copy of the current summary (with its score) before an automatic
/// regeneration, so a worse replacement can be undone.
pub async fn save_summary_before_regeneration(
    store: &Store,
    summary: &Summary,
) -> Result<(), StoreError> {
    store
        .put_json(&summary_before_regeneration_key(&summary.video_id), summary)
        .await
}

/// Returns and removes the copy saved by [`save_summary_before_regeneration`].
pub async fn take_summary_before_regeneration(
    store: &Store,
    video_id: &str,
) -> Result<Option<Summary>, StoreError> {
    let key = summary_before_regeneration_key(video_id);
    let summary: Option<Summary> = store.get_json(&key).await?;
    if summary.is_some() {
        store.delete_key(&key).await?;
    }
    Ok(summary)
}

pub async fn discard_summary_before_regeneration(
    store: &Store,
    video_id: &str,
) -> Result<(), StoreError> {
    store
        .delete_key(&summary_before_regeneration_key(video_id))
        .await
        .ok();
    Ok(())
}

#[derive(serde::Serialize, serde::Deserialize, Default)]
struct AutoRegenMeta {
    attempts: u8,
}

pub async fn get_summary_auto_regen_attempts(
    store: &Store,
    video_id: &str,
) -> Result<u8, StoreError> {
    let meta: Option<AutoRegenMeta> = store.get_json(&auto_regen_attempts_key(video_id)).await?;
    Ok(meta.map(|m| m.attempts).unwrap_or(0))
}

pub async fn reset_summary_auto_regen_attempts(
    store: &Store,
    video_id: &str,
) -> Result<(), StoreError> {
    store
        .delete_key(&auto_regen_attempts_key(video_id))
        .await
        .ok();
    Ok(())
}

pub async fn increment_summary_auto_regen_attempts(
    store: &Store,
    video_id: &str,
) -> Result<(), StoreError> {
    let current = get_summary_auto_regen_attempts(store, video_id).await?;
    store
        .put_json(
            &auto_regen_attempts_key(video_id),
            &AutoRegenMeta {
                attempts: current.saturating_add(1),
            },
        )
        .await
}

pub async fn delete_summary(store: &Store, video_id: &str) -> Result<bool, StoreError> {
    super::search::clear_search_source(store, video_id, SearchSourceKind::Summary).await?;
    clear_summary_evaluation_failures(store, video_id).await?;
    discard_summary_before_regeneration(store, video_id).await?;
    let key = summary_key(video_id);
    let exists = store.key_exists(&key).await?;
    if exists {
        store.delete_key(&key).await?;
    }
    Ok(exists)
}

pub async fn delete_transcript(store: &Store, video_id: &str) -> Result<bool, StoreError> {
    super::search::clear_search_source(store, video_id, SearchSourceKind::Transcript).await?;
    let key = transcript_key(video_id);
    let exists = store.key_exists(&key).await?;
    if exists {
        store.delete_key(&key).await?;
    }
    Ok(exists)
}

/// Video ids of all summaries that still need an evaluation.
///
/// This reads every stored summary, so call it rarely and work through the returned
/// list with [`load_summary_evaluation_job`].
///
/// Order: summaries with fewer failed evaluation attempts first, then newer videos
/// first. A few summaries that keep failing can no longer block everything else.
pub async fn list_video_ids_pending_quality_eval(store: &Store) -> Result<Vec<String>, StoreError> {
    let summaries: Vec<Summary> = store.load_all("summaries/").await?;
    let failures = load_evaluation_failures(store).await?;
    let mut candidates = Vec::new();

    for summary in summaries {
        if !summary_is_ready_for_quality_eval(&summary) {
            continue;
        }
        let Some(video) = ready_video_for_quality_eval(store, &summary.video_id).await? else {
            continue;
        };

        let failed_attempts = failures
            .get(&summary.video_id)
            .filter(|meta| meta.summary_hash == hash_search_content(&summary.content))
            .map(|meta| meta.failed_attempts)
            .unwrap_or(0);
        candidates.push((failed_attempts, video));
    }

    candidates.sort_by(|(failures_a, video_a), (failures_b, video_b)| {
        failures_a
            .cmp(failures_b)
            .then_with(|| video_b.published_at.cmp(&video_a.published_at))
    });

    Ok(candidates.into_iter().map(|(_, video)| video.id).collect())
}

/// The evaluation job for one video, read fresh from storage.
///
/// Returns `None` when the summary no longer needs an evaluation (for example it was
/// scored or deleted since the list was made) or there is no transcript to compare to.
pub async fn load_summary_evaluation_job(
    store: &Store,
    video_id: &str,
) -> Result<Option<SummaryEvaluationJob>, StoreError> {
    let Some(summary) = get_summary(store, video_id).await? else {
        return Ok(None);
    };
    if !summary_is_ready_for_quality_eval(&summary) {
        return Ok(None);
    }
    let Some(video) = ready_video_for_quality_eval(store, video_id).await? else {
        return Ok(None);
    };

    let transcript_text = get_transcript(store, video_id)
        .await?
        .and_then(|t| {
            [t.raw_text, t.formatted_markdown]
                .into_iter()
                .flatten()
                .find(|text| !text.trim().is_empty())
        })
        .unwrap_or_default();
    if transcript_text.trim().is_empty() {
        return Ok(None);
    }

    Ok(Some(SummaryEvaluationJob {
        video_id: summary.video_id,
        video_title: video.title,
        transcript_text: transcript_text.trim().to_string(),
        summary_content: summary.content,
    }))
}

/// Summaries that still need an evaluation, at most `limit`, in the order of
/// [`list_video_ids_pending_quality_eval`].
pub async fn list_summaries_pending_quality_eval(
    store: &Store,
    limit: usize,
) -> Result<Vec<SummaryEvaluationJob>, StoreError> {
    let mut results = Vec::new();
    for video_id in list_video_ids_pending_quality_eval(store).await? {
        if let Some(job) = load_summary_evaluation_job(store, &video_id).await? {
            results.push(job);
        }
        if results.len() >= limit {
            break;
        }
    }
    Ok(results)
}

fn summary_is_ready_for_quality_eval(summary: &Summary) -> bool {
    summary_needs_quality_eval(summary) && !summary.content.trim().is_empty()
}

async fn ready_video_for_quality_eval(
    store: &Store,
    video_id: &str,
) -> Result<Option<crate::models::Video>, StoreError> {
    let video = super::videos::get_video(store, video_id, false).await?;
    Ok(video.filter(|video| {
        video.transcript_status == ContentStatus::Ready
            && video.summary_status == ContentStatus::Ready
    }))
}

#[cfg(test)]
#[path = "tests.rs"]
mod tests;
