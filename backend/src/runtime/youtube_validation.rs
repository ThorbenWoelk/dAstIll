use crate::services::YouTubeService;

pub(super) async fn validate_youtube_api_key(youtube: &YouTubeService) {
    match youtube.validate_data_api_key().await {
        Ok(crate::services::DataApiKeyValidation::Valid) => {
            tracing::info!("YOUTUBE_API_KEY is configured and valid")
        }
        Ok(crate::services::DataApiKeyValidation::QuotaExceeded { message }) => {
            tracing::warn!(
                message = message.as_deref().unwrap_or("unknown"),
                "YOUTUBE_API_KEY is configured but YouTube Data API quota is currently exceeded"
            )
        }
        Ok(crate::services::DataApiKeyValidation::ServiceDisabled { reason, message }) => {
            tracing::warn!(
                reason = reason.as_deref().unwrap_or("unknown"),
                message = message.as_deref().unwrap_or("unknown"),
                "YOUTUBE_API_KEY is configured but YouTube Data API v3 is disabled for the active GCP project or the key belongs to a different project"
            )
        }
        Ok(crate::services::DataApiKeyValidation::Rejected { reason, message }) => {
            tracing::warn!(
                reason = reason.as_deref().unwrap_or("unknown"),
                message = message.as_deref().unwrap_or("unknown"),
                "YOUTUBE_API_KEY is configured but rejected by YouTube Data API"
            )
        }
        Ok(crate::services::DataApiKeyValidation::NotConfigured) => {
            tracing::info!("YOUTUBE_API_KEY is not configured - using fallback sources")
        }
        Err(err) => tracing::warn!(error = %err, "could not validate YOUTUBE_API_KEY on startup"),
    }
}
