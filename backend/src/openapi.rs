use std::sync::LazyLock;

use axum::Json;
use serde::Serialize;
use utoipa::{
    Modify, OpenApi, ToSchema,
    openapi::{
        Components, Server,
        security::{ApiKey, ApiKeyValue, HttpAuthScheme, HttpBuilder, SecurityScheme},
    },
};

#[derive(Debug, Serialize, ToSchema)]
pub struct VideosAddedResponse {
    pub videos_added: usize,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct ChannelBackfillResponse {
    pub videos_added: usize,
    pub fetched_count: usize,
    pub exhausted: bool,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct VideoInfoBackfillResponse {
    pub requested_limit: usize,
    pub force: bool,
    pub heal_placeholders: bool,
    pub processed: usize,
    pub updated: usize,
    pub failed: usize,
}

struct SecurityAddon;

impl Modify for SecurityAddon {
    fn modify(&self, openapi: &mut utoipa::openapi::OpenApi) {
        let components = openapi.components.get_or_insert_with(Components::new);
        components.add_security_scheme(
            "FirebaseBearer",
            SecurityScheme::Http(
                HttpBuilder::new()
                    .scheme(HttpAuthScheme::Bearer)
                    .bearer_format("JWT")
                    .build(),
            ),
        );
        components.add_security_scheme(
            "ProxyAuth",
            SecurityScheme::ApiKey(ApiKey::Header(ApiKeyValue::new("x-dastill-proxy-auth"))),
        );
        components.add_security_scheme(
            "OperatorRole",
            SecurityScheme::ApiKey(ApiKey::Header(ApiKeyValue::new("x-dastill-role"))),
        );
    }
}

#[derive(OpenApi)]
#[openapi(
    paths(
        crate::openapi::health,
        crate::openapi::get_openapi_json,
        crate::handlers::content::health_ai,
        crate::handlers::chat::get_client_config,
        crate::handlers::chat::suggest_channels,
        crate::handlers::chat::suggest_videos,
        crate::handlers::chat::list_conversations,
        crate::handlers::chat::create_conversation,
        crate::handlers::chat::get_conversation,
        crate::handlers::chat::update_conversation,
        crate::handlers::chat::delete_conversation,
        crate::handlers::chat::delete_all_conversations,
        crate::handlers::chat::start_ephemeral_reply,
        crate::handlers::chat::start_conversation_reply,
        crate::handlers::chat::resume_conversation_reply,
        crate::handlers::chat::cancel_conversation_reply,
        crate::handlers::preferences::get_preferences,
        crate::handlers::preferences::save_preferences,
        crate::handlers::search::search,
        crate::handlers::search::search_status,
        crate::handlers::search::search_status_stream,
        crate::handlers::search::rebuild_search_projection,
        crate::handlers::channels::workspace_bootstrap,
        crate::handlers::auth::create_mobile_auth_handoff,
        crate::handlers::auth::complete_mobile_auth_handoff,
        crate::handlers::auth::redeem_mobile_auth_handoff,
        crate::handlers::mini::get_mini_reader,
        crate::handlers::mini::update_mini_read_status,
        crate::handlers::channels::list_channels,
        crate::handlers::channels::add_channel,
        crate::handlers::channels::plan_openalex_query,
        crate::handlers::channels::get_channel,
        crate::handlers::channels::update_channel,
        crate::handlers::channels::delete_channel,
        crate::handlers::channels::get_channel_sync_depth,
        crate::handlers::channels::get_channel_snapshot,
        crate::handlers::channels::refresh_channel_videos,
        crate::handlers::channels::backfill_channel_videos,
        crate::handlers::videos::list_channel_videos,
        crate::handlers::videos::add_manual_video,
        crate::handlers::videos::get_video,
        crate::handlers::videos::get_video_info,
        crate::handlers::videos::ensure_video_info,
        crate::handlers::videos::backfill_video_info,
        crate::handlers::videos::update_video_acknowledged,
        crate::handlers::content::get_transcript,
        crate::handlers::content::generate_transcript,
        crate::handlers::content::update_transcript,
        crate::handlers::content::clean_transcript_formatting,
        crate::handlers::content::get_summary,
        crate::handlers::content::get_summary_audio,
        crate::handlers::content::generate_summary_audio,
        crate::handlers::content::get_summary_audio_debug,
        crate::handlers::content::generate_summary,
        crate::handlers::content::generation::update_summary,
        crate::handlers::content::regenerate_summary,
        crate::handlers::content::reset_video,
        crate::handlers::highlights::list_highlights,
        crate::handlers::highlights::list_video_highlights,
        crate::handlers::highlights::create_highlight,
        crate::handlers::highlights::delete_highlight,
        crate::handlers::analytics::ingest_events
    ),
    modifiers(&SecurityAddon),
    tags(
        (name = "Health"),
        (name = "Workspace"),
        (name = "Channels"),
        (name = "Videos"),
        (name = "Content"),
        (name = "Search"),
        (name = "Chat"),
        (name = "Highlights"),
        (name = "Auth"),
        (name = "Mini"),
        (name = "Preferences"),
        (name = "Analytics"),
        (name = "Debug")
    )
)]
struct ApiDoc;

static OPENAPI: LazyLock<utoipa::openapi::OpenApi> = LazyLock::new(|| {
    let mut doc = ApiDoc::openapi();
    doc.servers = Some(vec![Server::new("/")]);
    let note = "Import this live URL into Postman during local debugging. Protected routes accept either Authorization: Bearer <firebase-id-token> or x-dastill-proxy-auth.";
    match doc.info.description.as_mut() {
        Some(description) => {
            description.push_str("\n\n");
            description.push_str(note);
        }
        None => {
            doc.info.description = Some(note.to_string());
        }
    }
    doc
});

pub fn document() -> &'static utoipa::openapi::OpenApi {
    &OPENAPI
}

#[utoipa::path(
    get,
    path = "/api/health",
    responses(
        (status = 200, description = "Plain text health response", body = String, content_type = "text/plain")
    ),
    tag = "Health"
)]
pub async fn health() -> &'static str {
    "ok"
}

#[utoipa::path(
    get,
    path = "/api/openapi.json",
    responses(
        (status = 200, description = "Live backend OpenAPI document", body = String, content_type = "application/json")
    ),
    tag = "Debug"
)]
pub async fn get_openapi_json() -> Json<utoipa::openapi::OpenApi> {
    Json(document().clone())
}

#[cfg(test)]
#[path = "openapi_tests.rs"]
mod openapi_tests;
