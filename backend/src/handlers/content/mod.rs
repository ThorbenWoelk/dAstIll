use axum::{
    Json,
    extract::{Path, State},
    http::{StatusCode, header},
    response::IntoResponse,
};

use crate::audit;
use crate::db;
use crate::models::{
    CleanTranscriptResponse, ContentStatus, Summary, Transcript, TranscriptRenderMode,
    UpdateContentRequest,
};
use crate::services::SearchSourceKind;
use crate::services::search::hash_search_content;
use crate::services::summarizer::{
    MAX_TRANSCRIPT_FORMAT_ATTEMPTS, SummarizerError, apply_vocabulary_replacements,
};
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
    audit::log_video_reset(&video_id, &video.channel_id);

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

include!("frag_02.rs");
