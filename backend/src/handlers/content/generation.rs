use super::*;

const COMPLETED_LIVE_TRANSCRIPT_GRACE_SECONDS: i64 = 30 * 60;
const LONG_LIVE_MIN_DURATION_SECONDS: u64 = 30 * 60;
const DESCRIPTION_LIKE_MAX_TRANSCRIPT_WORDS: usize = 1_000;
const DESCRIPTION_LIKE_MIN_WORDS: usize = 40;
const DESCRIPTION_LIKE_OVERLAP_RATIO: f64 = 0.75;

#[utoipa::path(
    put,
    path = "/api/videos/{id}/summary",
    params(
        ("id" = String, Path, description = "Video id")
    ),
    request_body = UpdateContentRequest,
    responses(
        (status = 200, description = "Updated summary", body = Summary),
        (status = 404, description = "Video not found", body = String)
    )
)]
pub async fn update_summary(
    State(state): State<AppState>,
    Path(video_id): Path<String>,
    Json(payload): Json<UpdateContentRequest>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    let summary = save_manual_summary_content(&state, &video_id, &payload.content).await?;
    Ok(Json(summary))
}

/// Returns false for empty transcripts and YouTube site-wide placeholder blurbs that were
/// accidentally stored before the Firecrawl fallback was disabled.
fn is_valid_cached_transcript(transcript: &Transcript) -> bool {
    let text = transcript
        .raw_text
        .as_deref()
        .or(transcript.formatted_markdown.as_deref())
        .unwrap_or("");
    !text.trim().is_empty() && !is_site_wide_placeholder_description(text)
}

fn completed_live_transcript_grace_elapsed(
    actual_end: chrono::DateTime<chrono::Utc>,
    now: chrono::DateTime<chrono::Utc>,
) -> bool {
    now >= actual_end + chrono::Duration::seconds(COMPLETED_LIVE_TRANSCRIPT_GRACE_SECONDS)
}

fn normalized_word_tokens(text: &str) -> Vec<String> {
    text.split(|ch: char| !ch.is_alphanumeric())
        .map(str::trim)
        .filter(|token| !token.is_empty())
        .map(str::to_lowercase)
        .collect()
}

fn token_overlap_ratio(needle_tokens: &[String], haystack_tokens: &[String]) -> f64 {
    if needle_tokens.is_empty() {
        return 0.0;
    }

    let mut counts = std::collections::HashMap::<&str, usize>::new();
    for token in haystack_tokens {
        *counts.entry(token.as_str()).or_default() += 1;
    }

    let mut matches = 0usize;
    for token in needle_tokens {
        let Some(count) = counts.get_mut(token.as_str()) else {
            continue;
        };
        if *count > 0 {
            *count -= 1;
            matches += 1;
        }
    }

    matches as f64 / needle_tokens.len() as f64
}

fn completed_live_transcript_looks_like_description(
    transcript_text: &str,
    description: &str,
    duration_seconds: Option<u64>,
    timed_segment_count: usize,
) -> bool {
    if timed_segment_count > 0 {
        return false;
    }

    if duration_seconds
        .map(|duration| duration < LONG_LIVE_MIN_DURATION_SECONDS)
        .unwrap_or(true)
    {
        return false;
    }

    let transcript_tokens = normalized_word_tokens(transcript_text);
    if transcript_tokens.len() < DESCRIPTION_LIKE_MIN_WORDS
        || transcript_tokens.len() > DESCRIPTION_LIKE_MAX_TRANSCRIPT_WORDS
    {
        return false;
    }

    let description_tokens = normalized_word_tokens(description);
    if description_tokens.len() < DESCRIPTION_LIKE_MIN_WORDS {
        return false;
    }

    token_overlap_ratio(&transcript_tokens, &description_tokens) >= DESCRIPTION_LIKE_OVERLAP_RATIO
}

async fn defer_transcript_processing(
    state: &AppState,
    video_id: &str,
    message: &str,
) -> (StatusCode, String) {
    if let Err(err) =
        db::update_video_transcript_status(&state.db, video_id, ContentStatus::Pending).await
    {
        tracing::error!(
            video_id = %video_id,
            error = %err,
            "failed to persist deferred transcript status"
        );
    } else {
        evict_video_scope_cache_by_video_id(state, video_id).await;
    }
    state.transcript_cooldown.activate();
    (StatusCode::TOO_MANY_REQUESTS, message.to_string())
}

pub(crate) async fn ensure_transcript(
    state: &AppState,
    video_id: &str,
) -> Result<Transcript, (StatusCode, String)> {
    let video = require_video(state, video_id).await?;
    if let Some(transcript) = valid_cached_transcript(state, video_id).await? {
        let _ = db::update_video_transcript_status(&state.db, video_id, ContentStatus::Ready).await;
        tracing::debug!(video_id = %video_id, "transcript cache hit");
        return Ok(transcript);
    }

    if is_podcast_video(state, &video).await? {
        return ensure_podcast_audio_transcript(state, &video).await;
    }

    let completed_live = match state
        .youtube
        .fetch_completed_live_transcript_metadata(video_id)
        .await
    {
        Ok(metadata) => metadata,
        Err(crate::services::youtube::YouTubeError::NonCompletedLiveStream {
            state: live_state,
        }) => {
            return Err(
                defer_transcript_processing(
                    state,
                    video_id,
                    &format!(
                        "YouTube live stream is not finished yet ({live_state}); transcript will be retried later"
                    ),
                )
                .await,
            );
        }
        Err(err) => {
            tracing::warn!(
                video_id = %video_id,
                error = %err,
                "failed to fetch completed livestream transcript metadata; continuing without live transcript guard"
            );
            None
        }
    };

    if let Some(metadata) = completed_live.as_ref() {
        if !completed_live_transcript_grace_elapsed(metadata.actual_end, chrono::Utc::now()) {
            return Err(defer_transcript_processing(
                state,
                video_id,
                "Completed livestream transcript is not ready yet; retry later",
            )
            .await);
        }
    }

    {
        db::update_video_transcript_status(&state.db, video_id, ContentStatus::Loading)
            .await
            .map_err(map_db_err)?;
        evict_video_scope_cache_by_video_id(state, video_id).await;
        tracing::info!(video_id = %video_id, "transcript queued - status set to loading");
    }

    tracing::info!(video_id = %video_id, "starting transcript download");
    let (raw, formatted, timed) = match state.transcript.extract(video_id).await {
        Ok(result) => result,
        Err(err) => {
            return Err(apply_transcript_error(state, video_id, err).await);
        }
    };
    tracing::info!(
        video_id = %video_id,
        raw_bytes = raw.len(),
        markdown_bytes = formatted.len(),
        timed_segments = timed.len(),
        "transcript download completed"
    );

    if let Some(metadata) = completed_live.as_ref() {
        let candidate_text = if raw.trim().is_empty() {
            &formatted
        } else {
            &raw
        };
        if let Some(description) = metadata.description.as_deref() {
            if completed_live_transcript_looks_like_description(
                candidate_text,
                description,
                metadata.duration_seconds,
                timed.len(),
            ) {
                tracing::warn!(
                    video_id = %video_id,
                    transcript_words = candidate_text.split_whitespace().count(),
                    duration_seconds = metadata.duration_seconds.unwrap_or_default(),
                    "completed livestream transcript looks like the YouTube description; deferring retry"
                );
                return Err(defer_transcript_processing(
                    state,
                    video_id,
                    "Completed livestream transcript looks like the video description; retry later",
                )
                .await);
            }
        }
    }

    let transcript = Transcript {
        video_id: video_id.to_string(),
        raw_text: Some(raw),
        formatted_markdown: Some(formatted),
        render_mode: TranscriptRenderMode::PlainText,
        timed_text: if timed.is_empty() { None } else { Some(timed) },
    };

    db::upsert_transcript(&state.db, &transcript)
        .await
        .map_err(map_db_err)?;
    db::update_video_transcript_status(&state.db, video_id, ContentStatus::Ready)
        .await
        .map_err(map_db_err)?;
    sync_search_source(
        state,
        video_id,
        SearchSourceKind::Transcript,
        transcript_text(&transcript),
    )
    .await
    .map_err(map_db_err)?;
    evict_video_scope_cache(state, &video.channel_id).await?;
    tracing::info!(video_id = %video_id, "transcript stored - status set to ready");

    Ok(transcript)
}

async fn valid_cached_transcript(
    state: &AppState,
    video_id: &str,
) -> Result<Option<Transcript>, (StatusCode, String)> {
    if let Some(transcript) = db::get_transcript(&state.db, video_id)
        .await
        .map_err(map_db_err)?
    {
        if is_valid_cached_transcript(&transcript) {
            return Ok(Some(transcript));
        }
        tracing::warn!(
            video_id = %video_id,
            "cached transcript is invalid (site-wide blurb or empty) - discarding and re-fetching"
        );
    }
    Ok(None)
}

async fn is_podcast_video(
    state: &AppState,
    video: &crate::models::Video,
) -> Result<bool, (StatusCode, String)> {
    let source_profile = db::get_source_profile(&state.db, &video.channel_id)
        .await
        .map_err(map_db_err)?;
    Ok(source_profile
        .map(|profile| profile.source.provider == crate::models::ProviderKind::PodcastRss)
        .unwrap_or(false))
}

async fn ensure_podcast_audio_transcript(
    state: &AppState,
    video: &crate::models::Video,
) -> Result<Transcript, (StatusCode, String)> {
    db::update_video_transcript_status(&state.db, &video.id, ContentStatus::Loading)
        .await
        .map_err(map_db_err)?;
    evict_video_scope_cache_by_video_id(state, &video.id).await;
    tracing::info!(video_id = %video.id, "podcast transcript queued - status set to loading");

    let audio_asset = match db::get_source_audio_asset(&state.db, &video.id)
        .await
        .map_err(map_db_err)?
    {
        Some(asset) => asset,
        None => {
            db::update_video_transcript_status(&state.db, &video.id, ContentStatus::Failed)
                .await
                .map_err(map_db_err)?;
            return Err((
                StatusCode::NOT_FOUND,
                "Podcast episode has no source audio enclosure".to_string(),
            ));
        }
    };
    let Some(audio_url) = audio_asset.url.as_deref() else {
        db::update_video_transcript_status(&state.db, &video.id, ContentStatus::Failed)
            .await
            .map_err(map_db_err)?;
        return Err((
            StatusCode::NOT_FOUND,
            "Podcast episode source audio URL missing".to_string(),
        ));
    };

    let (raw, formatted, timed) = match state
        .transcript
        .extract_podcast_audio(&video.id, audio_url, audio_asset.mime_type.as_deref())
        .await
    {
        Ok(result) => result,
        Err(err) => {
            return Err(apply_transcript_error(state, &video.id, err).await);
        }
    };

    let transcript = Transcript {
        video_id: video.id.clone(),
        raw_text: Some(raw),
        formatted_markdown: Some(formatted),
        render_mode: TranscriptRenderMode::PlainText,
        timed_text: if timed.is_empty() { None } else { Some(timed) },
    };

    db::upsert_transcript(&state.db, &transcript)
        .await
        .map_err(map_db_err)?;
    db::update_video_transcript_status(&state.db, &video.id, ContentStatus::Ready)
        .await
        .map_err(map_db_err)?;
    sync_search_source(
        state,
        &video.id,
        SearchSourceKind::Transcript,
        transcript_text(&transcript),
    )
    .await
    .map_err(map_db_err)?;
    evict_video_scope_cache(state, &video.channel_id).await?;
    tracing::info!(video_id = %video.id, "podcast transcript stored - status set to ready");

    Ok(transcript)
}

pub(crate) async fn ensure_summary(
    state: &AppState,
    video_id: &str,
) -> Result<Summary, (StatusCode, String)> {
    ensure_summary_internal(state, video_id, false).await
}

pub(crate) async fn ensure_summary_for_queue(
    state: &AppState,
    video_id: &str,
) -> Result<Summary, (StatusCode, String)> {
    ensure_summary_internal(state, video_id, true).await
}

async fn ensure_summary_internal(
    state: &AppState,
    video_id: &str,
    allow_cached_auto_regen: bool,
) -> Result<Summary, (StatusCode, String)> {
    let video = require_video(state, video_id).await?;
    {
        if let Some(summary) = db::get_summary(&state.db, video_id)
            .await
            .map_err(map_db_err)?
        {
            if allow_cached_auto_regen {
                let auto_regen_attempts = db::get_summary_auto_regen_attempts(&state.db, video_id)
                    .await
                    .map_err(map_db_err)?;
                if should_auto_regenerate_summary(
                    video.summary_status,
                    summary.quality_score,
                    auto_regen_attempts,
                ) {
                    db::increment_summary_auto_regen_attempts(&state.db, video_id)
                        .await
                        .map_err(map_db_err)?;
                    tracing::info!(
                        video_id = %video_id,
                        score = summary.quality_score.unwrap_or_default(),
                        attempts_before = auto_regen_attempts,
                        max_attempts = MAX_SUMMARY_AUTO_REGEN_ATTEMPTS,
                        "summary auto-regeneration requested"
                    );
                } else {
                    let _ =
                        db::update_video_summary_status(&state.db, video_id, ContentStatus::Ready)
                            .await;
                    tracing::debug!(video_id = %video_id, "summary cache hit");
                    return Ok(summary);
                }
            } else {
                tracing::debug!(
                    video_id = %video_id,
                    summary_status = ?video.summary_status,
                    "summary cache hit (user read path)"
                );
                return Ok(summary);
            }
        }

        set_summary_status_and_evict(state, video_id, ContentStatus::Loading).await?;
        tracing::info!(video_id = %video_id, "summary queued - status set to loading");
    }

    if !state.summarizer.is_available().await {
        set_summary_status_and_evict(state, video_id, ContentStatus::Pending).await?;
        return Err((
            StatusCode::SERVICE_UNAVAILABLE,
            "Ollama not available".to_string(),
        ));
    }

    let transcript = match ensure_transcript(state, video_id).await {
        Ok(t) => t,
        Err((status, message)) => {
            let content_status = if matches!(
                status,
                StatusCode::TOO_MANY_REQUESTS | StatusCode::SERVICE_UNAVAILABLE
            ) {
                ContentStatus::Pending
            } else {
                ContentStatus::Failed
            };
            set_summary_status_and_evict(state, video_id, content_status).await?;
            return Err((status, message));
        }
    };
    let transcript_text = transcript_text(&transcript)
        .unwrap_or("")
        .trim()
        .to_string();

    if transcript_text.is_empty() {
        set_summary_status_and_evict(state, video_id, ContentStatus::Failed).await?;
        return Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            "Transcript content missing".to_string(),
        ));
    }

    let vocabulary_replacements = db::get_preferences(&state.db)
        .await
        .map_err(map_db_err)?
        .vocabulary_replacements;
    let normalized_transcript =
        apply_vocabulary_replacements(&transcript_text, &vocabulary_replacements);

    let summarize_result = state
        .summarizer
        .summarize(
            &normalized_transcript,
            &video.title,
            video_id,
            &video.channel_id,
            &vocabulary_replacements,
        )
        .await;
    let (content, model) = match summarize_result {
        Ok(pair) => pair,
        Err(e) => {
            let (http_status, content_status) = summarizer_error_statuses(&e);
            set_summary_status_and_evict(state, video_id, content_status).await?;
            let message = if content_status == ContentStatus::Pending {
                "AI generation is temporarily unavailable. The summary will retry when capacity returns."
                    .to_string()
            } else {
                e.to_string()
            };
            return Err((http_status, message));
        }
    };
    tracing::info!(video_id = %video_id, "summary generation completed");

    let summary = Summary {
        video_id: video_id.to_string(),
        content,
        model_used: Some(model),
        quality_score: None,
        quality_note: None,
        quality_model_used: None,
        summary_tags: Vec::new(),
        summary_tags_evaluated: false,
    };

    db::upsert_summary(&state.db, &summary)
        .await
        .map_err(map_db_err)?;
    db::update_video_summary_status(&state.db, video_id, ContentStatus::Ready)
        .await
        .map_err(map_db_err)?;
    sync_search_source(
        state,
        video_id,
        SearchSourceKind::Summary,
        Some(summary.content.as_str()),
    )
    .await
    .map_err(map_db_err)?;
    evict_video_scope_cache(state, &video.channel_id).await?;
    tracing::info!(video_id = %video_id, "summary stored - status set to ready");

    Ok(summary)
}

pub(super) async fn save_manual_transcript_content(
    state: &AppState,
    video_id: &str,
    content: &str,
    render_mode: Option<TranscriptRenderMode>,
) -> Result<Transcript, (StatusCode, String)> {
    let video = require_video(state, video_id).await?;
    let existing_render_mode = db::get_transcript(&state.db, video_id)
        .await
        .map_err(map_db_err)?
        .map(|transcript| transcript.render_mode);
    let effective_render_mode = render_mode
        .or(existing_render_mode)
        .unwrap_or(TranscriptRenderMode::PlainText);
    let transcript =
        db::save_manual_transcript(&state.db, video_id, content, effective_render_mode)
            .await
            .map_err(map_db_err)?;
    sync_search_source(
        state,
        video_id,
        SearchSourceKind::Transcript,
        transcript_text(&transcript),
    )
    .await
    .map_err(map_db_err)?;
    evict_video_scope_cache(state, &video.channel_id).await?;
    Ok(transcript)
}

async fn save_manual_summary_content(
    state: &AppState,
    video_id: &str,
    content: &str,
) -> Result<Summary, (StatusCode, String)> {
    let video = require_video(state, video_id).await?;
    let summary = db::save_manual_summary(&state.db, video_id, content, Some("manual"))
        .await
        .map_err(map_db_err)?;
    sync_search_source(
        state,
        video_id,
        SearchSourceKind::Summary,
        Some(summary.content.as_str()),
    )
    .await
    .map_err(map_db_err)?;
    evict_video_scope_cache(state, &video.channel_id).await?;
    Ok(summary)
}

fn transcript_text(transcript: &Transcript) -> Option<&str> {
    [
        transcript.raw_text.as_deref(),
        transcript.formatted_markdown.as_deref(),
    ]
    .into_iter()
    .flatten()
    .find(|content| !content.trim().is_empty())
}

async fn sync_search_source(
    state: &AppState,
    video_id: &str,
    source_kind: SearchSourceKind,
    content: Option<&str>,
) -> Result<(), crate::db::StoreError> {
    match content.map(str::trim) {
        Some(content) if !content.is_empty() => {
            let content_hash = hash_search_content(content);
            let current = db::get_search_source_state(&state.db, video_id, source_kind).await?;
            if db::should_refresh_search_source(
                current.as_ref(),
                &content_hash,
                state.search.semantic_enabled(),
                state.search.model(),
            ) {
                db::mark_search_source_pending(&state.db, video_id, source_kind, &content_hash)
                    .await
            } else {
                Ok(())
            }
        }
        _ => {
            state
                .fts
                .delete_source(video_id, source_kind)
                .await
                .map_err(crate::db::StoreError::Other)?;
            db::clear_search_source(&state.db, video_id, source_kind).await
        }
    }
}

/// Persist transcript status after extraction failure **before** returning to callers
/// (e.g. the queue worker) that increment `retry_count`. A previous `tokio::spawn` here
/// raced S3 writes and left rows stuck in `loading` with `retry_count >= MAX`, which
/// `next_queue_task` then skips forever.
async fn apply_transcript_error(
    state: &AppState,
    video_id: &str,
    err: crate::services::transcript::TranscriptError,
) -> (StatusCode, String) {
    match &err {
        crate::services::transcript::TranscriptError::RateLimited => {
            tracing::warn!(video_id = %video_id, error = %err, "transcript download rate limited");
            state.transcript_cooldown.activate();
        }
        crate::services::transcript::TranscriptError::NoTranscript => {
            tracing::warn!(
                video_id = %video_id,
                error = %err,
                "transcript unavailable for video"
            );
        }
        crate::services::transcript::TranscriptError::AsrUnavailable => {
            tracing::warn!(
                video_id = %video_id,
                error = %err,
                "podcast transcript deferred because local ASR is unavailable"
            );
        }
        crate::services::transcript::TranscriptError::AsrTemporarilyUnavailable(_) => {
            tracing::warn!(
                video_id = %video_id,
                error = %err,
                "podcast transcript deferred because local ASR is temporarily unavailable"
            );
        }
        _ => {
            tracing::error!(video_id = %video_id, error = %err, "transcript download failed");
        }
    }

    let status = match err {
        crate::services::transcript::TranscriptError::RateLimited => StatusCode::TOO_MANY_REQUESTS,
        crate::services::transcript::TranscriptError::NoTranscript => StatusCode::NOT_FOUND,
        crate::services::transcript::TranscriptError::AsrUnavailable => {
            StatusCode::SERVICE_UNAVAILABLE
        }
        crate::services::transcript::TranscriptError::AsrTemporarilyUnavailable(_) => {
            StatusCode::SERVICE_UNAVAILABLE
        }
        _ => StatusCode::INTERNAL_SERVER_ERROR,
    };

    let next_status = match err {
        crate::services::transcript::TranscriptError::RateLimited
        | crate::services::transcript::TranscriptError::AsrUnavailable
        | crate::services::transcript::TranscriptError::AsrTemporarilyUnavailable(_) => {
            ContentStatus::Pending
        }
        _ => ContentStatus::Failed,
    };

    if let Err(e) = db::update_video_transcript_status(&state.db, video_id, next_status).await {
        tracing::error!(
            video_id = %video_id,
            error = %e,
            "failed to persist transcript status after extraction error"
        );
    } else {
        evict_video_scope_cache_by_video_id(state, video_id).await;
    }

    (status, err.to_string())
}

/// Updates the summary status and immediately evicts the cache for that video's scope.
async fn set_summary_status_and_evict(
    state: &AppState,
    video_id: &str,
    status: ContentStatus,
) -> Result<(), (StatusCode, String)> {
    db::update_video_summary_status(&state.db, video_id, status)
        .await
        .map_err(map_db_err)?;
    evict_video_scope_cache_by_video_id(state, video_id).await;
    Ok(())
}

/// Maps a summarizer error to the HTTP status and content status to persist.
fn summarizer_error_statuses(e: &SummarizerError) -> (StatusCode, ContentStatus) {
    if e.is_rate_limited() {
        (StatusCode::TOO_MANY_REQUESTS, ContentStatus::Pending)
    } else if matches!(e, SummarizerError::NotAvailable) {
        (StatusCode::SERVICE_UNAVAILABLE, ContentStatus::Pending)
    } else {
        (StatusCode::INTERNAL_SERVER_ERROR, ContentStatus::Failed)
    }
}

async fn evict_video_scope_cache_by_video_id(state: &AppState, video_id: &str) {
    let Ok(Some(video)) = db::get_video(&state.db, video_id, false).await else {
        return;
    };
    let _ = evict_video_scope_cache(state, &video.channel_id).await;
}

#[cfg(test)]
mod tests {
    use super::{
        MAX_SUMMARY_AUTO_REGEN_ATTEMPTS, completed_live_transcript_grace_elapsed,
        completed_live_transcript_looks_like_description, is_valid_cached_transcript,
        should_auto_regenerate_summary, summarizer_error_statuses, transcript_text,
    };
    use crate::models::{ContentStatus, Transcript, TranscriptRenderMode};
    use crate::services::summarizer::SummarizerError;
    use axum::http::StatusCode;

    fn make_transcript(raw: Option<&str>, formatted: Option<&str>) -> Transcript {
        Transcript {
            video_id: "vid1".to_string(),
            raw_text: raw.map(ToOwned::to_owned),
            formatted_markdown: formatted.map(ToOwned::to_owned),
            render_mode: TranscriptRenderMode::PlainText,
            timed_text: None,
        }
    }

    #[test]
    fn summarizer_temporary_errors_keep_summary_pending() {
        assert_eq!(
            summarizer_error_statuses(&SummarizerError::GenerationFailed(
                "subscription limit reached".to_string()
            )),
            (StatusCode::TOO_MANY_REQUESTS, ContentStatus::Pending)
        );
        assert_eq!(
            summarizer_error_statuses(&SummarizerError::NotAvailable),
            (StatusCode::SERVICE_UNAVAILABLE, ContentStatus::Pending)
        );
    }

    #[test]
    fn summarizer_non_temporary_errors_mark_summary_failed() {
        assert_eq!(
            summarizer_error_statuses(&SummarizerError::GenerationFailed(
                "malformed model output".to_string()
            )),
            (StatusCode::INTERNAL_SERVER_ERROR, ContentStatus::Failed)
        );
    }

    #[test]
    fn valid_cached_transcript_accepts_real_content() {
        let t = make_transcript(Some("Hello world, this is a transcript."), None);
        assert!(is_valid_cached_transcript(&t));
    }

    #[test]
    fn valid_cached_transcript_rejects_youtube_site_wide_blurb_in_raw_text() {
        let t = make_transcript(
            Some(
                "Enjoy the videos and music you love, upload original content, and share it all with friends, family, and the world on YouTube.\n",
            ),
            None,
        );
        assert!(!is_valid_cached_transcript(&t));
    }

    #[test]
    fn valid_cached_transcript_rejects_youtube_site_wide_blurb_in_formatted_markdown() {
        let t = make_transcript(
            None,
            Some(
                "Enjoy the videos and music you love, upload original content, and share it all with friends, family, and the world on YouTube.\n",
            ),
        );
        assert!(!is_valid_cached_transcript(&t));
    }

    #[test]
    fn valid_cached_transcript_rejects_empty_raw_text() {
        let t = make_transcript(Some("   "), None);
        assert!(!is_valid_cached_transcript(&t));
    }

    #[test]
    fn valid_cached_transcript_rejects_all_none() {
        let t = make_transcript(None, None);
        assert!(!is_valid_cached_transcript(&t));
    }

    #[test]
    fn valid_cached_transcript_falls_back_to_formatted_when_raw_is_none() {
        let t = make_transcript(None, Some("Actual transcript content here."));
        assert!(is_valid_cached_transcript(&t));
    }

    #[test]
    fn completed_live_transcript_grace_waits_after_actual_end() {
        let ended = chrono::DateTime::parse_from_rfc3339("2026-04-18T22:27:42Z")
            .unwrap()
            .with_timezone(&chrono::Utc);

        assert!(!completed_live_transcript_grace_elapsed(
            ended,
            ended + chrono::Duration::minutes(2)
        ));
        assert!(completed_live_transcript_grace_elapsed(
            ended,
            ended + chrono::Duration::minutes(31)
        ));
    }

    #[test]
    fn completed_live_transcript_rejects_description_like_text_for_long_stream() {
        let description = "Yesterday I tested Claude Design against v0, Grok, Google Stitch, Cursor, Droid, and ChatGPT Pro on the same blog redesign. Claude Design produced the visual system. Droid turned it into a working prototype with Opus on max thinking. It looked great locally. But a prototype that runs on your laptop is not a shipped product. Today I am closing the loop and getting my blog live on the internet.";
        let transcript = "Yesterday I tested Claude Design against v0, Grok, Google Stitch, Cursor, Droid, and ChatGPT Pro on the same blog redesign. Claude Design produced the visual system. Droid turned it into a working prototype with Opus on max thinking. It looked great locally. But a prototype that runs on your laptop is not a shipped product. Today I am closing the loop and getting my blog live on the internet.";

        assert!(completed_live_transcript_looks_like_description(
            transcript,
            description,
            Some(3 * 60 * 60),
            0
        ));
    }

    #[test]
    fn completed_live_transcript_accepts_real_long_caption_text() {
        let description =
            "Today I am shipping a blog live on the internet after yesterday's prototype.";
        let transcript = (0..1_500)
            .map(|index| format!("caption{index}"))
            .collect::<Vec<_>>()
            .join(" ");

        assert!(!completed_live_transcript_looks_like_description(
            &transcript,
            description,
            Some(3 * 60 * 60),
            0
        ));
    }

    #[test]
    fn completed_live_transcript_accepts_timed_segments_even_when_short() {
        let description = "Yesterday I tested Claude Design against v0, Grok, Google Stitch, Cursor, Droid, and ChatGPT Pro on the same blog redesign. Claude Design produced the visual system. Droid turned it into a working prototype with Opus on max thinking. It looked great locally. But a prototype that runs on your laptop is not a shipped product. Today I am closing the loop and getting my blog live on the internet.";

        assert!(!completed_live_transcript_looks_like_description(
            description,
            description,
            Some(3 * 60 * 60),
            12
        ));
    }

    #[test]
    fn should_auto_regenerate_summary_requires_pending_or_loading_and_low_score() {
        assert!(should_auto_regenerate_summary(
            ContentStatus::Pending,
            Some(6),
            0
        ));
        assert!(should_auto_regenerate_summary(
            ContentStatus::Loading,
            Some(0),
            1
        ));
        assert!(!should_auto_regenerate_summary(
            ContentStatus::Ready,
            Some(2),
            0
        ));
        assert!(!should_auto_regenerate_summary(
            ContentStatus::Pending,
            Some(7),
            0
        ));
        assert!(!should_auto_regenerate_summary(
            ContentStatus::Pending,
            None,
            0
        ));
    }

    #[test]
    fn should_auto_regenerate_summary_respects_max_attempts() {
        assert!(!should_auto_regenerate_summary(
            ContentStatus::Pending,
            Some(1),
            MAX_SUMMARY_AUTO_REGEN_ATTEMPTS
        ));
    }

    #[test]
    fn transcript_text_falls_back_to_formatted_markdown_when_raw_text_is_blank() {
        let transcript = Transcript {
            video_id: "video-123".to_string(),
            raw_text: Some("   ".to_string()),
            formatted_markdown: Some("## Section\nUseful formatted text".to_string()),
            render_mode: TranscriptRenderMode::Markdown,
            timed_text: None,
        };

        assert_eq!(
            transcript_text(&transcript),
            Some("## Section\nUseful formatted text")
        );
    }

    #[test]
    fn transcript_text_prefers_non_empty_raw_text() {
        let transcript = Transcript {
            video_id: "video-123".to_string(),
            raw_text: Some("Raw transcript text".to_string()),
            formatted_markdown: Some("## Section\nFormatted text".to_string()),
            render_mode: TranscriptRenderMode::Markdown,
            timed_text: None,
        };

        assert_eq!(transcript_text(&transcript), Some("Raw transcript text"));
    }
}
