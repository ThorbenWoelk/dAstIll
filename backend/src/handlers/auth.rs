use std::time::{Duration, Instant};

use axum::{
    Json,
    extract::{Extension, Path, State},
    http::{HeaderMap, StatusCode, header},
    response::IntoResponse,
};
use rand::RngCore;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use utoipa::ToSchema;

use crate::{
    security::{AccessContext, AuthState, CLIENT_IP_HEADER},
    state::{AppState, MobileAuthHandoff},
};

const MOBILE_AUTH_HANDOFF_TTL: Duration = Duration::from_secs(5 * 60);
const MOBILE_AUTH_HANDOFF_TOKEN_BYTES: usize = 32;
const X_FORWARDED_FOR_HEADER: &str = "x-forwarded-for";

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct CreateMobileAuthHandoffPayload {
    pub status: String,
    pub handoff_id: String,
    pub complete_token: String,
    pub redeem_token: String,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct MobileAuthHandoffStatusPayload {
    pub status: String,
    pub google_id_token: Option<String>,
    pub google_access_token: Option<String>,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct CompleteMobileAuthHandoffPayload {
    pub complete_token: String,
    pub google_id_token: String,
    pub google_access_token: String,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct RedeemMobileAuthHandoffPayload {
    pub redeem_token: String,
}

fn prune_expired_handoffs(state: &mut std::collections::HashMap<String, MobileAuthHandoff>) {
    let now = Instant::now();
    state.retain(|_, handoff| now.duration_since(handoff.created_at) < MOBILE_AUTH_HANDOFF_TTL);
}

fn mint_nonce() -> String {
    let mut bytes = [0_u8; MOBILE_AUTH_HANDOFF_TOKEN_BYTES];
    rand::thread_rng().fill_bytes(&mut bytes);
    bytes.iter().map(|byte| format!("{byte:02x}")).collect()
}

fn hash_value(value: &str) -> String {
    Sha256::digest(value.as_bytes())
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect()
}

fn token_matches(expected_hash: &str, candidate: &str) -> bool {
    hash_value(candidate) == expected_hash
}

fn handoff_not_found() -> (StatusCode, String) {
    (
        StatusCode::NOT_FOUND,
        "Mobile auth handoff not found".to_string(),
    )
}

fn handoff_pending_payload() -> MobileAuthHandoffStatusPayload {
    MobileAuthHandoffStatusPayload {
        status: "pending".to_string(),
        google_id_token: None,
        google_access_token: None,
    }
}

fn extract_client_hint(headers: &HeaderMap) -> Option<&str> {
    headers
        .get(CLIENT_IP_HEADER)
        .and_then(|value| value.to_str().ok())
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .or_else(|| {
            headers
                .get(X_FORWARDED_FOR_HEADER)
                .and_then(|value| value.to_str().ok())
                .and_then(|value| value.split(',').next())
                .map(str::trim)
                .filter(|value| !value.is_empty())
        })
}

fn resolve_creator_binding(headers: &HeaderMap, access_context: &AccessContext) -> String {
    if let Some(authorization) = headers
        .get(header::AUTHORIZATION)
        .and_then(|value| value.to_str().ok())
        .map(str::trim)
        .filter(|value| !value.is_empty())
    {
        return format!("authorization:{}", hash_value(authorization));
    }

    if access_context.auth_state == AuthState::Authenticated {
        if let Some(user_id) = access_context.user_id.as_deref() {
            return format!("user:{user_id}");
        }
    }

    if let Some(client_hint) = extract_client_hint(headers) {
        return format!("client:{}", hash_value(client_hint));
    }

    format!("scope:{}", access_context.cache_scope_key())
}

#[utoipa::path(
    post,
    path = "/api/auth/mobile-handoff",
    responses(
        (status = 201, description = "Created a mobile auth handoff session", body = CreateMobileAuthHandoffPayload)
    )
)]
pub async fn create_mobile_auth_handoff(
    State(state): State<AppState>,
    Extension(access_context): Extension<AccessContext>,
    headers: HeaderMap,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    let handoff_id = mint_nonce();
    let complete_token = mint_nonce();
    let redeem_token = mint_nonce();
    let creator_binding = resolve_creator_binding(&headers, &access_context);

    let mut handoffs = state.mobile_auth_handoffs.lock().await;
    prune_expired_handoffs(&mut handoffs);
    handoffs.insert(
        handoff_id.clone(),
        MobileAuthHandoff {
            created_at: Instant::now(),
            creator_binding_hash: hash_value(&creator_binding),
            complete_token_hash: hash_value(&complete_token),
            redeem_token_hash: hash_value(&redeem_token),
            google_id_token: None,
            google_access_token: None,
        },
    );

    Ok((
        StatusCode::CREATED,
        Json(CreateMobileAuthHandoffPayload {
            status: "pending".to_string(),
            handoff_id,
            complete_token,
            redeem_token,
        }),
    ))
}

#[utoipa::path(
    put,
    path = "/api/auth/mobile-handoff/{id}",
    params(
        ("id" = String, Path, description = "Mobile auth handoff session id")
    ),
    request_body = CompleteMobileAuthHandoffPayload,
    responses(
        (status = 200, description = "Completed handoff session", body = MobileAuthHandoffStatusPayload),
        (status = 400, description = "Request failed", body = String),
        (status = 403, description = "Authenticated handoff completion required", body = String),
        (status = 404, description = "Unknown handoff session", body = String),
        (status = 409, description = "Handoff session already completed", body = String)
    )
)]
pub async fn complete_mobile_auth_handoff(
    State(state): State<AppState>,
    Extension(access_context): Extension<AccessContext>,
    Path(session_id): Path<String>,
    Json(payload): Json<CompleteMobileAuthHandoffPayload>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    if access_context.auth_state != AuthState::Authenticated {
        return Err((
            StatusCode::FORBIDDEN,
            "Authenticated handoff completion required".to_string(),
        ));
    }

    let complete_token = payload.complete_token.trim();
    let google_id_token = payload.google_id_token.trim();
    let google_access_token = payload.google_access_token.trim();
    if complete_token.is_empty() || google_id_token.is_empty() || google_access_token.is_empty() {
        return Err((
            StatusCode::BAD_REQUEST,
            "Complete token and Google auth tokens are required".to_string(),
        ));
    }

    let mut handoffs = state.mobile_auth_handoffs.lock().await;
    prune_expired_handoffs(&mut handoffs);
    let Some(handoff) = handoffs.get_mut(&session_id) else {
        return Err(handoff_not_found());
    };

    if !token_matches(&handoff.complete_token_hash, complete_token) {
        return Err(handoff_not_found());
    }

    if handoff.google_id_token.is_some() || handoff.google_access_token.is_some() {
        return Err((
            StatusCode::CONFLICT,
            "Mobile auth handoff already completed".to_string(),
        ));
    }

    handoff.google_id_token = Some(google_id_token.to_string());
    handoff.google_access_token = Some(google_access_token.to_string());

    Ok(Json(MobileAuthHandoffStatusPayload {
        status: "complete".to_string(),
        google_id_token: None,
        google_access_token: None,
    }))
}

#[utoipa::path(
    post,
    path = "/api/auth/mobile-handoff/{id}/redeem",
    params(
        ("id" = String, Path, description = "Mobile auth handoff session id")
    ),
    request_body = RedeemMobileAuthHandoffPayload,
    responses(
        (status = 200, description = "Redeemed completed handoff session", body = MobileAuthHandoffStatusPayload),
        (status = 202, description = "Handoff session exists but browser login is still pending", body = MobileAuthHandoffStatusPayload),
        (status = 400, description = "Request failed", body = String),
        (status = 404, description = "Unknown handoff session", body = String)
    )
)]
pub async fn redeem_mobile_auth_handoff(
    State(state): State<AppState>,
    Extension(access_context): Extension<AccessContext>,
    headers: HeaderMap,
    Path(session_id): Path<String>,
    Json(payload): Json<RedeemMobileAuthHandoffPayload>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    let redeem_token = payload.redeem_token.trim();
    if redeem_token.is_empty() {
        return Err((
            StatusCode::BAD_REQUEST,
            "Redeem token is required".to_string(),
        ));
    }

    let creator_binding = resolve_creator_binding(&headers, &access_context);
    let creator_binding_hash = hash_value(&creator_binding);

    let mut handoffs = state.mobile_auth_handoffs.lock().await;
    prune_expired_handoffs(&mut handoffs);
    let Some(handoff) = handoffs.get(&session_id) else {
        return Err(handoff_not_found());
    };

    if handoff.creator_binding_hash != creator_binding_hash
        || !token_matches(&handoff.redeem_token_hash, redeem_token)
    {
        return Err(handoff_not_found());
    }

    if handoff.google_id_token.is_none() || handoff.google_access_token.is_none() {
        return Ok((StatusCode::ACCEPTED, Json(handoff_pending_payload())));
    }

    let handoff = handoffs.remove(&session_id).ok_or_else(handoff_not_found)?;

    Ok((
        StatusCode::OK,
        Json(MobileAuthHandoffStatusPayload {
            status: "complete".to_string(),
            google_id_token: handoff.google_id_token,
            google_access_token: handoff.google_access_token,
        }),
    ))
}

#[cfg(test)]
mod tests {
    use std::{collections::HashMap, sync::Arc};

    use axum::{
        Extension, Json,
        body::to_bytes,
        extract::{Path, State},
        http::{HeaderMap, HeaderValue, StatusCode, header},
        response::IntoResponse,
    };
    use reqwest::Client;

    use super::{
        CompleteMobileAuthHandoffPayload, CreateMobileAuthHandoffPayload,
        MobileAuthHandoffStatusPayload, RedeemMobileAuthHandoffPayload,
        complete_mobile_auth_handoff, create_mobile_auth_handoff, redeem_mobile_auth_handoff,
    };
    use crate::{
        config::SecurityRuntimeConfig,
        db::Store,
        read_cache::ReadCache,
        search_progress::SearchProgress,
        security::{AccessContext, AccessRole, AuthState, RequestRateLimiter},
        services::{
            ChatService, CloudCooldown, FtsIndex, InputGuardrailService, OllamaCore,
            OpenAlexPlannerService, OpenAlexService, PodcastFeedService, SearchService,
            SummarizerService, SummaryEvaluatorService, TranscriptCooldown, TranscriptService,
            UserActivity, WebsiteService, YouTubeQuotaCooldown, YouTubeService,
        },
        state::AppState,
    };

    fn anonymous_access_context() -> AccessContext {
        AccessContext {
            user_id: None,
            auth_state: AuthState::Anonymous,
            access_role: AccessRole::Anonymous,
            allowed_channel_ids: Vec::new(),
            allowed_other_video_ids: Vec::new(),
        }
    }

    fn authenticated_access_context(user_id: &str) -> AccessContext {
        AccessContext {
            user_id: Some(user_id.to_string()),
            auth_state: AuthState::Authenticated,
            access_role: AccessRole::User,
            allowed_channel_ids: Vec::new(),
            allowed_other_video_ids: Vec::new(),
        }
    }

    fn authorization_headers(token: &str) -> HeaderMap {
        let mut headers = HeaderMap::new();
        headers.insert(
            header::AUTHORIZATION,
            HeaderValue::from_str(token).expect("valid auth header"),
        );
        headers
    }

    fn test_security_config() -> SecurityRuntimeConfig {
        SecurityRuntimeConfig {
            proxy_token: "proxy".to_string(),
            firebase_project_id: "demo-dastill".to_string(),
            allowed_origins: vec![],
            operator_email_allowlist: vec![],
            default_seeded_channel_id: "seeded-channel".to_string(),
            baseline_rate_limit_per_minute: 600,
            expensive_rate_limit_per_minute: 120,
            anonymous_chat_quota: 30,
        }
    }

    async fn test_app_state() -> AppState {
        let store = Store::for_test().await;
        let cooldown = Arc::new(CloudCooldown::cloud());
        let security = Arc::new(test_security_config());
        let search = Arc::new(SearchService::with_config(
            "://invalid-url",
            None,
            crate::services::search::SEARCH_EMBEDDING_DIMENSIONS,
            false,
        ));

        AppState {
            db: store,
            read_cache: Arc::new(ReadCache::default()),
            security: security.clone(),
            request_rate_limiter: Arc::new(RequestRateLimiter::new(security.as_ref())),
            search_auto_create_vector_index: false,
            search_projection_lock: Arc::new(tokio::sync::RwLock::new(())),
            search_progress: Arc::new(SearchProgress::new(
                search.model(),
                search.dimensions(),
                search.semantic_enabled(),
            )),
            fts: Arc::new(FtsIndex::new().await.expect("fts index")),
            youtube: Arc::new(YouTubeService::with_client(Client::new())),
            openalex_planner: Arc::new(OpenAlexPlannerService::new(
                OllamaCore::new("://invalid-url", "qwen3:8b").with_cloud_cooldown(cooldown.clone()),
            )),
            openalex: Arc::new(OpenAlexService::with_client(Client::new())),
            podcast_feed: Arc::new(PodcastFeedService::with_client(Client::new())),
            website: Arc::new(WebsiteService::with_client(Client::new())),
            transcript: Arc::new(TranscriptService::with_path("/usr/bin/false")),
            tts: None,
            summarizer: Arc::new(SummarizerService::new(
                OllamaCore::new("://invalid-url", "qwen3:8b").with_cloud_cooldown(cooldown.clone()),
            )),
            summary_evaluator: Arc::new(SummaryEvaluatorService::new(
                OllamaCore::new("://invalid-url", "qwen3.5:397b-cloud")
                    .with_cloud_cooldown(cooldown.clone()),
            )),
            search,
            chat: Arc::new(ChatService::new(
                OllamaCore::new("://invalid-url", "qwen3:8b").with_cloud_cooldown(cooldown.clone()),
            )),
            input_guardrails: Arc::new(InputGuardrailService::new(
                OllamaCore::new("://invalid-url", "qwen3:8b").with_cloud_cooldown(cooldown.clone()),
                Vec::new(),
                Vec::new(),
            )),
            analytics: None,
            active_replies: Arc::new(tokio::sync::Mutex::new(HashMap::new())),
            conversation_store_lock: Arc::new(tokio::sync::Mutex::new(())),
            anonymous_chat_quota_lock: Arc::new(tokio::sync::Mutex::new(())),
            mobile_auth_handoffs: Arc::new(tokio::sync::Mutex::new(HashMap::new())),
            cloud_cooldown: cooldown,
            youtube_quota_cooldown: Arc::new(YouTubeQuotaCooldown::youtube_quota()),
            transcript_cooldown: Arc::new(TranscriptCooldown::transcript()),
            user_activity: Arc::new(UserActivity::from_env()),
        }
    }

    async fn json_body<T: serde::de::DeserializeOwned>(response: impl IntoResponse) -> T {
        let response = response.into_response();
        let body = to_bytes(response.into_body(), usize::MAX)
            .await
            .expect("response body should read");
        serde_json::from_slice(&body).expect("payload should deserialize")
    }

    #[tokio::test]
    async fn mobile_auth_handoff_requires_matching_creator_binding_for_redeem() {
        let state = test_app_state().await;
        let created: CreateMobileAuthHandoffPayload = json_body(
            create_mobile_auth_handoff(
                State(state.clone()),
                Extension(anonymous_access_context()),
                authorization_headers("Bearer anon-a"),
            )
            .await
            .expect("handoff should be created"),
        )
        .await;

        complete_mobile_auth_handoff(
            State(state.clone()),
            Extension(authenticated_access_context("google-user")),
            Path(created.handoff_id.clone()),
            Json(CompleteMobileAuthHandoffPayload {
                complete_token: created.complete_token.clone(),
                google_id_token: "google-id".to_string(),
                google_access_token: "google-access".to_string(),
            }),
        )
        .await
        .expect("handoff should complete");

        let wrong_redeem = redeem_mobile_auth_handoff(
            State(state.clone()),
            Extension(anonymous_access_context()),
            authorization_headers("Bearer anon-b"),
            Path(created.handoff_id.clone()),
            Json(RedeemMobileAuthHandoffPayload {
                redeem_token: created.redeem_token.clone(),
            }),
        )
        .await;
        assert!(matches!(
            wrong_redeem,
            Err((StatusCode::NOT_FOUND, message)) if message == "Mobile auth handoff not found"
        ));

        let redeemed: MobileAuthHandoffStatusPayload = json_body(
            redeem_mobile_auth_handoff(
                State(state.clone()),
                Extension(anonymous_access_context()),
                authorization_headers("Bearer anon-a"),
                Path(created.handoff_id.clone()),
                Json(RedeemMobileAuthHandoffPayload {
                    redeem_token: created.redeem_token.clone(),
                }),
            )
            .await
            .expect("matching creator should redeem"),
        )
        .await;

        assert_eq!(redeemed.status, "complete");
        assert_eq!(redeemed.google_id_token.as_deref(), Some("google-id"));
        assert_eq!(
            redeemed.google_access_token.as_deref(),
            Some("google-access")
        );

        let second_redeem = redeem_mobile_auth_handoff(
            State(state),
            Extension(anonymous_access_context()),
            authorization_headers("Bearer anon-a"),
            Path(created.handoff_id),
            Json(RedeemMobileAuthHandoffPayload {
                redeem_token: created.redeem_token,
            }),
        )
        .await;
        assert!(matches!(
            second_redeem,
            Err((StatusCode::NOT_FOUND, message)) if message == "Mobile auth handoff not found"
        ));
    }

    #[tokio::test]
    async fn mobile_auth_handoff_redeem_stays_pending_until_browser_completes() {
        let state = test_app_state().await;
        let created: CreateMobileAuthHandoffPayload = json_body(
            create_mobile_auth_handoff(
                State(state.clone()),
                Extension(anonymous_access_context()),
                authorization_headers("Bearer anon-pending"),
            )
            .await
            .expect("handoff should be created"),
        )
        .await;

        let response = redeem_mobile_auth_handoff(
            State(state),
            Extension(anonymous_access_context()),
            authorization_headers("Bearer anon-pending"),
            Path(created.handoff_id),
            Json(RedeemMobileAuthHandoffPayload {
                redeem_token: created.redeem_token,
            }),
        )
        .await
        .expect("pending handoff should return status");
        let response = response.into_response();
        assert_eq!(response.status(), StatusCode::ACCEPTED);

        let body = to_bytes(response.into_body(), usize::MAX)
            .await
            .expect("response body should read");
        let payload: MobileAuthHandoffStatusPayload =
            serde_json::from_slice(&body).expect("payload should deserialize");
        assert_eq!(payload.status, "pending");
        assert!(payload.google_id_token.is_none());
        assert!(payload.google_access_token.is_none());
    }

    #[tokio::test]
    async fn mobile_auth_handoff_completion_requires_authenticated_browser_session() {
        let state = test_app_state().await;
        let created: CreateMobileAuthHandoffPayload = json_body(
            create_mobile_auth_handoff(
                State(state.clone()),
                Extension(anonymous_access_context()),
                authorization_headers("Bearer anon-complete"),
            )
            .await
            .expect("handoff should be created"),
        )
        .await;

        let completion = complete_mobile_auth_handoff(
            State(state),
            Extension(anonymous_access_context()),
            Path(created.handoff_id),
            Json(CompleteMobileAuthHandoffPayload {
                complete_token: created.complete_token,
                google_id_token: "google-id".to_string(),
                google_access_token: "google-access".to_string(),
            }),
        )
        .await;

        assert!(matches!(
            completion,
            Err((StatusCode::FORBIDDEN, message))
                if message == "Authenticated handoff completion required"
        ));
    }
}
