use axum::{
    Json,
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
};

use crate::db;
use crate::models::{
    CleanTranscriptResponse, ContentStatus, Summary, Transcript, TranscriptRenderMode,
    UpdateContentRequest,
};
use crate::services::summarizer::{MAX_TRANSCRIPT_FORMAT_ATTEMPTS, SummarizerError};
use crate::state::AppState;

use super::{map_db_err, require_video};

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
    require_video(&state, &video_id).await?;

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
        .clean_transcript_formatting(&payload.content)
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
            Err((StatusCode::INTERNAL_SERVER_ERROR, err.to_string()))
        }
    }
}

pub async fn get_summary(
    State(state): State<AppState>,
    Path(video_id): Path<String>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    tracing::debug!(video_id = %video_id, "summary requested");
    let summary = ensure_summary(&state, &video_id).await?;
    Ok(Json(summary))
}

pub async fn regenerate_summary(
    State(state): State<AppState>,
    Path(video_id): Path<String>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    tracing::info!(video_id = %video_id, "summary regeneration requested");
    require_video(&state, &video_id).await?;
    {
        let conn = state.db.lock().await;
        db::delete_summary(&conn, &video_id)
            .await
            .map_err(map_db_err)?;
    }

    let summary = ensure_summary(&state, &video_id).await?;
    Ok(Json(summary))
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

pub(crate) async fn ensure_transcript(
    state: &AppState,
    video_id: &str,
) -> Result<Transcript, (StatusCode, String)> {
    require_video(state, video_id).await?;
    {
        let conn = state.db.lock().await;
        if let Some(transcript) = db::get_transcript(&conn, video_id)
            .await
            .map_err(map_db_err)?
        {
            let _ = db::update_video_transcript_status(&conn, video_id, ContentStatus::Ready).await;
            tracing::debug!(video_id = %video_id, "transcript cache hit");
            return Ok(transcript);
        }

        db::update_video_transcript_status(&conn, video_id, ContentStatus::Loading)
            .await
            .map_err(map_db_err)?;
        tracing::info!(video_id = %video_id, "transcript queued - status set to loading");
    }

    tracing::info!(video_id = %video_id, "starting transcript download");
    let (raw, formatted) = state
        .transcript
        .extract(video_id)
        .await
        .map_err(|e| map_transcript_err(state, video_id, e))?;
    tracing::info!(
        video_id = %video_id,
        raw_bytes = raw.len(),
        markdown_bytes = formatted.len(),
        "transcript download completed"
    );

    let transcript = Transcript {
        video_id: video_id.to_string(),
        raw_text: Some(raw),
        formatted_markdown: Some(formatted),
        render_mode: TranscriptRenderMode::PlainText,
    };

    let conn = state.db.lock().await;
    db::upsert_transcript(&conn, &transcript)
        .await
        .map_err(map_db_err)?;
    db::update_video_transcript_status(&conn, video_id, ContentStatus::Ready)
        .await
        .map_err(map_db_err)?;
    tracing::info!(video_id = %video_id, "transcript stored - status set to ready");

    Ok(transcript)
}

pub(crate) async fn ensure_summary(
    state: &AppState,
    video_id: &str,
) -> Result<Summary, (StatusCode, String)> {
    let video = require_video(state, video_id).await?;
    {
        let conn = state.db.lock().await;
        if let Some(summary) = db::get_summary(&conn, video_id).await.map_err(map_db_err)? {
            let auto_regen_attempts = db::get_summary_auto_regen_attempts(&conn, video_id)
                .await
                .map_err(map_db_err)?;
            if should_auto_regenerate_summary(
                video.summary_status,
                summary.quality_score,
                auto_regen_attempts,
            ) {
                db::increment_summary_auto_regen_attempts(&conn, video_id)
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
                    db::update_video_summary_status(&conn, video_id, ContentStatus::Ready).await;
                tracing::debug!(video_id = %video_id, "summary cache hit");
                return Ok(summary);
            }
        }

        db::update_video_summary_status(&conn, video_id, ContentStatus::Loading)
            .await
            .map_err(map_db_err)?;
        tracing::info!(video_id = %video_id, "summary queued - status set to loading");
    }

    if !state.summarizer.is_available().await {
        let conn = state.db.lock().await;
        db::update_video_summary_status(&conn, video_id, ContentStatus::Failed)
            .await
            .map_err(map_db_err)?;
        return Err((
            StatusCode::SERVICE_UNAVAILABLE,
            "Ollama not available".to_string(),
        ));
    }

    let transcript = ensure_transcript(state, video_id)
        .await
        .map_err(|(status, message)| {
            let next_status = if status == StatusCode::TOO_MANY_REQUESTS {
                ContentStatus::Pending
            } else {
                ContentStatus::Failed
            };
            spawn_status_update(state, video_id, StatusField::Summary, next_status);
            (status, message)
        })?;
    let transcript_text = transcript
        .raw_text
        .as_deref()
        .or(transcript.formatted_markdown.as_deref())
        .unwrap_or("")
        .trim()
        .to_string();

    if transcript_text.is_empty() {
        let conn = state.db.lock().await;
        db::update_video_summary_status(&conn, video_id, ContentStatus::Failed)
            .await
            .map_err(map_db_err)?;
        return Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            "Transcript content missing".to_string(),
        ));
    }

    let (content, model) = state
        .summarizer
        .summarize(&transcript_text, &video.title)
        .await
        .map_err(|e| {
            let error_msg = e.to_string();
            let status = if error_msg.contains("rate limited") || error_msg.contains("429") {
                StatusCode::TOO_MANY_REQUESTS
            } else {
                StatusCode::INTERNAL_SERVER_ERROR
            };

            let next_status = if status == StatusCode::TOO_MANY_REQUESTS {
                ContentStatus::Pending
            } else {
                ContentStatus::Failed
            };

            spawn_status_update(state, video_id, StatusField::Summary, next_status);
            (status, error_msg)
        })?;
    tracing::info!(video_id = %video_id, "summary generation completed");

    let summary = Summary {
        video_id: video_id.to_string(),
        content,
        model_used: Some(model),
        quality_score: None,
        quality_note: None,
        quality_model_used: None,
    };

    let conn = state.db.lock().await;
    db::upsert_summary(&conn, &summary)
        .await
        .map_err(map_db_err)?;
    db::update_video_summary_status(&conn, video_id, ContentStatus::Ready)
        .await
        .map_err(map_db_err)?;
    tracing::info!(video_id = %video_id, "summary stored - status set to ready");

    Ok(summary)
}

async fn save_manual_transcript_content(
    state: &AppState,
    video_id: &str,
    content: &str,
    render_mode: Option<TranscriptRenderMode>,
) -> Result<Transcript, (StatusCode, String)> {
    require_video(state, video_id).await?;
    let conn = state.db.lock().await;
    let existing_render_mode = db::get_transcript(&conn, video_id)
        .await
        .map_err(map_db_err)?
        .map(|transcript| transcript.render_mode);
    let effective_render_mode = render_mode
        .or(existing_render_mode)
        .unwrap_or(TranscriptRenderMode::PlainText);
    db::save_manual_transcript(&conn, video_id, content, effective_render_mode)
        .await
        .map_err(map_db_err)
}

async fn save_manual_summary_content(
    state: &AppState,
    video_id: &str,
    content: &str,
) -> Result<Summary, (StatusCode, String)> {
    require_video(state, video_id).await?;
    let conn = state.db.lock().await;
    db::save_manual_summary(&conn, video_id, content, Some("manual"))
        .await
        .map_err(map_db_err)
}

enum StatusField {
    Transcript,
    Summary,
}

fn spawn_status_update(
    state: &AppState,
    video_id: &str,
    field: StatusField,
    status: ContentStatus,
) {
    let state = state.clone();
    let video_id = video_id.to_string();
    tokio::spawn(async move {
        let conn = state.db.lock().await;
        let _ = match field {
            StatusField::Transcript => {
                db::update_video_transcript_status(&conn, &video_id, status).await
            }
            StatusField::Summary => db::update_video_summary_status(&conn, &video_id, status).await,
        };
    });
}

fn map_transcript_err(
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

    spawn_status_update(state, video_id, StatusField::Transcript, next_status);

    (status, err.to_string())
}

#[cfg(test)]
mod tests {
    use super::{MAX_SUMMARY_AUTO_REGEN_ATTEMPTS, should_auto_regenerate_summary};
    use crate::models::ContentStatus;

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
}
