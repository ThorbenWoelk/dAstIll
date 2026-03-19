use crate::models::{
    ContentStatus, Summary, SummaryEvaluationJob, Transcript, TranscriptRenderMode,
};
use crate::services::search::SearchSourceKind;

use super::{Store, StoreError};

fn transcript_key(video_id: &str) -> String {
    format!("transcripts/{video_id}.json")
}

fn summary_key(video_id: &str) -> String {
    format!("summaries/{video_id}.json")
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
    };
    store.put_json(&summary_key(video_id), &summary).await?;
    // Reset auto_regen_attempts
    store
        .delete_key(&format!("meta/auto-regen-attempts/{video_id}"))
        .await
        .ok();
    super::videos::update_video_summary_status(store, video_id, ContentStatus::Ready).await?;
    Ok(summary)
}

pub async fn update_summary_quality(
    store: &Store,
    video_id: &str,
    quality_score: Option<u8>,
    quality_note: Option<&str>,
    quality_model_used: Option<&str>,
) -> Result<(), StoreError> {
    let key = summary_key(video_id);
    if let Some(mut summary) = store.get_json::<Summary>(&key).await? {
        summary.quality_score = quality_score;
        summary.quality_note = quality_note.map(ToOwned::to_owned);
        summary.quality_model_used = quality_model_used.map(ToOwned::to_owned);
        store.put_json(&key, &summary).await?;
    }
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
    let meta: Option<AutoRegenMeta> = store
        .get_json(&format!("meta/auto-regen-attempts/{video_id}"))
        .await?;
    Ok(meta.map(|m| m.attempts).unwrap_or(0))
}

pub async fn increment_summary_auto_regen_attempts(
    store: &Store,
    video_id: &str,
) -> Result<(), StoreError> {
    let current = get_summary_auto_regen_attempts(store, video_id).await?;
    store
        .put_json(
            &format!("meta/auto-regen-attempts/{video_id}"),
            &AutoRegenMeta {
                attempts: current.saturating_add(1),
            },
        )
        .await
}

pub async fn delete_summary(store: &Store, video_id: &str) -> Result<bool, StoreError> {
    super::search::clear_search_source(store, video_id, SearchSourceKind::Summary).await?;
    let key = summary_key(video_id);
    let exists = store.key_exists(&key).await?;
    if exists {
        store.delete_key(&key).await?;
    }
    Ok(exists)
}

pub async fn list_summaries_pending_quality_eval(
    store: &Store,
    limit: usize,
) -> Result<Vec<SummaryEvaluationJob>, StoreError> {
    let summaries: Vec<Summary> = store.load_all("summaries/").await?;
    let mut results = Vec::new();

    for summary in summaries {
        if summary.quality_score.is_some() || summary.quality_note.is_some() {
            continue;
        }
        if summary.content.trim().is_empty() {
            continue;
        }

        let video = store
            .get_json::<crate::models::Video>(&format!("videos/{}.json", summary.video_id))
            .await?;
        let Some(video) = video else { continue };
        if video.transcript_status != ContentStatus::Ready
            || video.summary_status != ContentStatus::Ready
        {
            continue;
        }

        let transcript = store
            .get_json::<crate::models::Transcript>(&format!(
                "transcripts/{}.json",
                summary.video_id
            ))
            .await?;
        let transcript_text = transcript
            .and_then(|t| t.raw_text.or(t.formatted_markdown))
            .unwrap_or_default();
        if transcript_text.trim().is_empty() {
            continue;
        }

        results.push(SummaryEvaluationJob {
            video_id: summary.video_id,
            video_title: video.title,
            transcript_text: transcript_text.trim().to_string(),
            summary_content: summary.content,
        });

        if results.len() >= limit {
            break;
        }
    }

    Ok(results)
}
