use axum::{
    Json,
    extract::{Path, State},
    http::{StatusCode, header},
    response::IntoResponse,
};

use crate::db;
use crate::models::{
    CleanTranscriptResponse, ContentStatus, Summary, Transcript, TranscriptRenderMode,
    UpdateContentRequest,
};
use crate::services::SearchSourceKind;
use crate::services::search::hash_search_content;
use crate::services::summarizer::{MAX_TRANSCRIPT_FORMAT_ATTEMPTS, SummarizerError};
use crate::services::youtube::placeholder::is_site_wide_placeholder_description;
use crate::state::AppState;

use super::{evict_video_scope_cache, map_db_err, require_present, require_video};

pub(crate) const MIN_SUMMARY_QUALITY_SCORE_FOR_ACCEPTANCE: u8 = 7;
pub(crate) const MAX_SUMMARY_AUTO_REGEN_ATTEMPTS: u8 = 2;

pub(crate) fn should_auto_regenerate_summary(
    summary_status: ContentStatus,
    quality_score: Option<u8>,
    auto_regen_attempts: u8,
) -> bool {
    matches!(
        summary_status,
        ContentStatus::Pending | ContentStatus::Loading
    ) && quality_score
        .map(|score| score < MIN_SUMMARY_QUALITY_SCORE_FOR_ACCEPTANCE)
        .unwrap_or(false)
        && auto_regen_attempts < MAX_SUMMARY_AUTO_REGEN_ATTEMPTS
}

pub async fn get_transcript(
    State(state): State<AppState>,
    Path(video_id): Path<String>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    tracing::debug!(video_id = %video_id, "transcript requested");
    require_video(&state, &video_id).await?;
    let transcript = db::get_transcript(&state.db, &video_id)
        .await
        .map_err(map_db_err)
        .and_then(|opt| require_present(opt, "Transcript not found"))?;
    Ok(Json(transcript))
}

pub async fn generate_transcript(
    State(state): State<AppState>,
    Path(video_id): Path<String>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    tracing::info!(video_id = %video_id, "transcript generation requested");
    let transcript = ensure_transcript(&state, &video_id).await?;
    Ok(Json(transcript))
}

pub async fn update_transcript(
    State(state): State<AppState>,
    Path(video_id): Path<String>,
    Json(payload): Json<UpdateContentRequest>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    let transcript =
        save_manual_transcript_content(&state, &video_id, &payload.content, payload.render_mode)
            .await?;
    Ok(Json(transcript))
}

pub async fn clean_transcript_formatting(
    State(state): State<AppState>,
    Path(video_id): Path<String>,
    Json(payload): Json<UpdateContentRequest>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    tracing::info!(
        video_id = %video_id,
        model = %state.summarizer.model(),
        input_chars = payload.content.len(),
        "transcript clean formatting requested"
    );
    let video = require_video(&state, &video_id).await?;

    if payload.content.trim().is_empty() {
        tracing::info!(video_id = %video_id, "transcript clean skipped for empty input");
        return Ok(Json(CleanTranscriptResponse {
            content: payload.content,
            preserved_text: true,
            attempts_used: 0,
            max_attempts: MAX_TRANSCRIPT_FORMAT_ATTEMPTS as u8,
            timed_out: false,
        }));
    }

    if !state.summarizer.is_available().await {
        tracing::warn!(video_id = %video_id, "transcript clean failed - ollama unavailable");
        return Err((
            StatusCode::SERVICE_UNAVAILABLE,
            "Ollama not available".to_string(),
        ));
    }

    match state
        .summarizer
        .clean_transcript_formatting(&payload.content, &video_id, &video.channel_id)
        .await
    {
        Ok(result) => Ok(Json(CleanTranscriptResponse {
            content: result.content,
            preserved_text: true,
            attempts_used: result.attempts_used as u8,
            max_attempts: result.max_attempts as u8,
            timed_out: false,
        })),
        Err(SummarizerError::TextChanged {
            attempts_used,
            max_attempts,
        }) => {
            tracing::warn!(
                video_id = %video_id,
                "transcript clean output modified wording - returning original input"
            );
            Ok(Json(CleanTranscriptResponse {
                content: payload.content,
                preserved_text: false,
                attempts_used: attempts_used as u8,
                max_attempts: max_attempts as u8,
                timed_out: false,
            }))
        }
        Err(SummarizerError::TimedOut {
            attempts_used,
            max_attempts,
            timeout_secs,
        }) => {
            tracing::warn!(
                video_id = %video_id,
                attempts_used = attempts_used,
                max_attempts = max_attempts,
                timeout_secs = timeout_secs,
                "transcript clean timed out - returning original input"
            );
            Ok(Json(CleanTranscriptResponse {
                content: payload.content,
                preserved_text: true,
                attempts_used: attempts_used as u8,
                max_attempts: max_attempts as u8,
                timed_out: true,
            }))
        }
        Err(err) => {
            tracing::error!(video_id = %video_id, error = %err, "transcript clean failed");
            Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                "Transcript clean failed".to_string(),
            ))
        }
    }
}

pub async fn get_summary(
    State(state): State<AppState>,
    Path(video_id): Path<String>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    tracing::debug!(video_id = %video_id, "summary requested");
    require_video(&state, &video_id).await?;
    let summary = db::get_summary(&state.db, &video_id)
        .await
        .map_err(map_db_err)
        .and_then(|opt| require_present(opt, "Summary not found"))?;
    Ok(Json(summary))
}

async fn get_summary_audio_cache_info(
    tts: &crate::services::PollyTtsService,
    video_id: &str,
    summary_content: &str,
) -> Result<(String, String, String), (StatusCode, String)> {
    let voice_id = tts
        .resolve_voice_id_for_cache_key()
        .await
        .map_err(|err| (StatusCode::BAD_GATEWAY, err.to_string()))?;

    let output_format = tts.output_format();
    let is_wav = output_format.starts_with("wav");
    let ext = if is_wav { "wav" } else { "mp3" };

    let tts_text = crate::services::tts::sanitize_markdown_for_tts(summary_content);
    let audio_hash = hash_search_content(&format!(
        "{}|voice_id={voice_id}|model_id={}|output_format={}",
        tts_text.trim(),
        tts.model_id(),
        output_format
    ));

    let key = db::summary_audio_cache_key(video_id, &audio_hash, ext);
    let content_type = if is_wav { "audio/wav" } else { "audio/mpeg" };

    Ok((key, content_type.to_string(), tts_text))
}

pub async fn get_summary_audio(
    State(state): State<AppState>,
    Path(video_id): Path<String>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    tracing::info!(video_id = %video_id, "summary audio requested");
    require_video(&state, &video_id).await?;
    let summary = db::get_summary(&state.db, &video_id)
        .await
        .map_err(map_db_err)
        .and_then(|opt| require_present(opt, "Summary not found"))?;

    let tts = state.tts.as_ref().ok_or((
        StatusCode::SERVICE_UNAVAILABLE,
        "Polly TTS is not configured".to_string(),
    ))?;

    let (key, content_type, _) =
        get_summary_audio_cache_info(tts, &video_id, &summary.content).await?;

    match db::get_summary_audio(&state.db, &key).await {
        Ok(Some(audio)) => {
            let effective_content_type = if audio.starts_with(b"RIFF") {
                "audio/wav"
            } else {
                &content_type
            };

            Ok((
                [
                    (header::CONTENT_TYPE, effective_content_type.to_string()),
                    (
                        header::CACHE_CONTROL,
                        "public, max-age=31536000, immutable".to_string(),
                    ),
                ],
                audio,
            ))
        }
        Ok(None) => Err((
            StatusCode::NOT_FOUND,
            "Summary audio not yet generated or has expired.".to_string(),
        )),
        Err(err) => Err(map_db_err(err)),
    }
}

pub async fn generate_summary_audio(
    State(state): State<AppState>,
    Path(video_id): Path<String>,
) -> Result<StatusCode, (StatusCode, String)> {
    tracing::info!(video_id = %video_id, "summary audio generation requested");
    require_video(&state, &video_id).await?;
    let summary = db::get_summary(&state.db, &video_id)
        .await
        .map_err(map_db_err)
        .and_then(|opt| require_present(opt, "Summary not found"))?;

    let tts = state.tts.as_ref().ok_or((
        StatusCode::SERVICE_UNAVAILABLE,
        "Polly TTS is not configured".to_string(),
    ))?;

    let (key, content_type, tts_text) =
        get_summary_audio_cache_info(tts, &video_id, &summary.content).await?;

    // Check if it already exists to avoid redundant synthesis
    if let Ok(Some(_)) = db::get_summary_audio(&state.db, &key).await {
        return Ok(StatusCode::OK);
    }

    let word_count = tts_text.split_whitespace().count() as u32;
    let started_at = std::time::Instant::now();

    let audio = tts.synthesize_summary(&tts_text).await.map_err(|err| {
        tracing::error!(video_id = %video_id, error = %err, "summary audio synthesis failed");
        (StatusCode::BAD_GATEWAY, err.to_string())
    })?;

    let duration_secs = started_at.elapsed().as_secs_f64();

    let effective_content_type = if audio.starts_with(b"RIFF") {
        "audio/wav"
    } else {
        &content_type
    };

    db::put_summary_audio(&state.db, &key, &audio, effective_content_type)
        .await
        .map_err(map_db_err)?;

    if let Err(err) = db::record_tts_generation(&state.db, word_count, duration_secs).await {
        tracing::warn!(error = %err, "failed to record TTS generation stats");
    }

    Ok(StatusCode::OK)
}

#[derive(serde::Serialize)]
pub struct SummaryAudioDebugResponse {
    ok: bool,
    cache_hit: bool,
    error: Option<String>,
    word_count: Option<u32>,
    estimated_secs: Option<f32>,
}

pub async fn get_summary_audio_debug(
    State(state): State<AppState>,
    Path(video_id): Path<String>,
) -> Result<Json<SummaryAudioDebugResponse>, (StatusCode, String)> {
    require_video(&state, &video_id).await?;
    let summary = db::get_summary(&state.db, &video_id)
        .await
        .map_err(map_db_err)
        .and_then(|opt| require_present(opt, "Summary not found"))?;

    let tts = state.tts.as_ref().ok_or((
        StatusCode::SERVICE_UNAVAILABLE,
        "Polly TTS is not configured".to_string(),
    ))?;

    let (key, _, tts_text) = get_summary_audio_cache_info(tts, &video_id, &summary.content).await?;
    let word_count = tts_text.split_whitespace().count() as u32;

    let stats = db::get_tts_stats(&state.db).await.ok().flatten();
    let estimated_secs = stats.as_ref().and_then(|s| s.estimate_secs(word_count));

    match db::summary_audio_exists(&state.db, &key).await {
        Ok(true) => Ok(Json(SummaryAudioDebugResponse {
            ok: true,
            cache_hit: true,
            error: None,
            word_count: Some(word_count),
            estimated_secs,
        })),
        Ok(false) => Ok(Json(SummaryAudioDebugResponse {
            ok: true,
            cache_hit: false,
            error: None,
            word_count: Some(word_count),
            estimated_secs,
        })),
        Err(err) => Ok(Json(SummaryAudioDebugResponse {
            ok: false,
            cache_hit: false,
            error: Some(format!("{err}")),
            word_count: None,
            estimated_secs: None,
        })),
    }
}

pub async fn generate_summary(
    State(state): State<AppState>,
    Path(video_id): Path<String>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    tracing::info!(video_id = %video_id, "summary generation requested");
    let summary = ensure_summary(&state, &video_id).await?;
    Ok(Json(summary))
}

pub async fn regenerate_summary(
    State(state): State<AppState>,
    Path(video_id): Path<String>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    tracing::info!(video_id = %video_id, "summary regeneration requested");
    let video = require_video(&state, &video_id).await?;
    {
        db::delete_summary(&state.db, &video_id)
            .await
            .map_err(map_db_err)?;
    }
    evict_video_scope_cache(&state, &video.channel_id).await?;

    let summary = ensure_summary(&state, &video_id).await?;
    Ok(Json(summary))
}

/// Wipe transcript, summary, quality metadata, and search vectors for a video,
/// then reset its status fields to `pending` so the queue re-processes it from scratch.
pub async fn reset_video(
    State(state): State<AppState>,
    Path(video_id): Path<String>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    tracing::info!(video_id = %video_id, "video reset requested");
    let video = require_video(&state, &video_id).await?;

    db::delete_transcript(&state.db, &video_id)
        .await
        .map_err(map_db_err)?;
    db::delete_summary(&state.db, &video_id)
        .await
        .map_err(map_db_err)?;
    // Reset regen counter so the queue won't skip re-generation thinking it already tried.
    db::reset_summary_auto_regen_attempts(&state.db, &video_id)
        .await
        .map_err(map_db_err)?;
    db::update_video_transcript_status(&state.db, &video_id, ContentStatus::Pending)
        .await
        .map_err(map_db_err)?;
    db::update_video_summary_status(&state.db, &video_id, ContentStatus::Pending)
        .await
        .map_err(map_db_err)?;

    evict_video_scope_cache(&state, &video.channel_id).await?;
    tracing::info!(video_id = %video_id, "video reset complete - transcript and summary wiped, status pending");

    Ok(StatusCode::NO_CONTENT)
}

pub async fn health_ai(State(state): State<AppState>) -> impl IntoResponse {
    let available = state.summarizer.is_available().await;
    let status = state
        .summarizer
        .indicator_status(state.cloud_cooldown.is_active(), available);
    Json(crate::models::AiHealthPayload { available, status })
}

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

pub(crate) async fn ensure_transcript(
    state: &AppState,
    video_id: &str,
) -> Result<Transcript, (StatusCode, String)> {
    let video = require_video(state, video_id).await?;
    {
        if let Some(transcript) = db::get_transcript(&state.db, video_id)
            .await
            .map_err(map_db_err)?
        {
            if is_valid_cached_transcript(&transcript) {
                let _ =
                    db::update_video_transcript_status(&state.db, video_id, ContentStatus::Ready)
                        .await;
                tracing::debug!(video_id = %video_id, "transcript cache hit");
                return Ok(transcript);
            }
            tracing::warn!(
                video_id = %video_id,
                "cached transcript is invalid (site-wide blurb or empty) - discarding and re-fetching"
            );
        }

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
        &state.db,
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
        set_summary_status_and_evict(state, video_id, ContentStatus::Failed).await?;
        return Err((
            StatusCode::SERVICE_UNAVAILABLE,
            "Ollama not available".to_string(),
        ));
    }

    let transcript = match ensure_transcript(state, video_id).await {
        Ok(t) => t,
        Err((status, message)) => {
            let content_status = if status == StatusCode::TOO_MANY_REQUESTS {
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

    let summarize_result = state
        .summarizer
        .summarize(&transcript_text, &video.title, video_id, &video.channel_id)
        .await;
    let (content, model) = match summarize_result {
        Ok(pair) => pair,
        Err(e) => {
            let (http_status, content_status) = summarizer_error_statuses(&e);
            set_summary_status_and_evict(state, video_id, content_status).await?;
            return Err((http_status, e.to_string()));
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
    };

    db::upsert_summary(&state.db, &summary)
        .await
        .map_err(map_db_err)?;
    db::update_video_summary_status(&state.db, video_id, ContentStatus::Ready)
        .await
        .map_err(map_db_err)?;
    sync_search_source(
        &state.db,
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

async fn save_manual_transcript_content(
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
        &state.db,
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
        &state.db,
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
    conn: &crate::db::Store,
    video_id: &str,
    source_kind: SearchSourceKind,
    content: Option<&str>,
) -> Result<(), crate::db::StoreError> {
    match content.map(str::trim) {
        Some(content) if !content.is_empty() => {
            let content_hash = hash_search_content(content);
            db::mark_search_source_pending(conn, video_id, source_kind, &content_hash).await
        }
        _ => db::clear_search_source(conn, video_id, source_kind).await,
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
        _ => {
            tracing::error!(video_id = %video_id, error = %err, "transcript download failed");
        }
    }

    let status = match err {
        crate::services::transcript::TranscriptError::RateLimited => StatusCode::TOO_MANY_REQUESTS,
        crate::services::transcript::TranscriptError::NoTranscript => StatusCode::NOT_FOUND,
        _ => StatusCode::INTERNAL_SERVER_ERROR,
    };

    let next_status = match err {
        crate::services::transcript::TranscriptError::RateLimited => ContentStatus::Pending,
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
        MAX_SUMMARY_AUTO_REGEN_ATTEMPTS, is_valid_cached_transcript,
        should_auto_regenerate_summary, transcript_text,
    };
    use crate::models::{ContentStatus, Transcript, TranscriptRenderMode};

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
