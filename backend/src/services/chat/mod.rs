mod cloud_models;
mod constants;
mod intent;
mod recent;
mod tools;

pub use cloud_models::{default_chat_cloud_model_id, is_chat_cloud_model_choice};
pub(crate) use constants::*;
pub use intent::ChatQueryIntent;

use std::collections::{HashMap, HashSet};
use std::convert::Infallible;
use std::future::Future;
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{Duration, Instant};

use axum::response::sse::Event;
use chrono::Utc;
use serde::{Deserialize, Serialize};
use tokio::sync::{Mutex, broadcast, mpsc, watch};
use tokio::time::timeout;
use tokio_stream::wrappers::ReceiverStream;
use tracing::Instrument;

use crate::db;
use crate::models::{
    ChatConversation, ChatMessage, ChatMessageStatus, ChatRole, ChatSource, ChatTitleStatus,
};
use crate::services::ollama::OllamaCore;
use crate::services::search::SearchCandidate;
use crate::services::text::limit_text;
use crate::state::AppState;

use super::chat::recent::{
    execute_recent_library_activity_query, is_explicit_realtime_status_query,
    is_recent_activity_query,
};
use super::chat_heuristics::{
    build_plan_label, collect_focus_terms, heuristic_expansion_queries, heuristic_query_variants,
    is_attributed_preference_query, push_unique_query, recommendation_query_variants,
    sanitize_queries,
};
use super::chat_prompt::{
    build_conversation_only_grounding, build_grounding_context, build_ollama_messages,
    build_synthesis_grounding_context, build_tool_grounding_context, synthesis_raw_limit_for_plan,
};
use super::chat_ranking::{
    accumulate_ranked_candidates, assess_coverage, build_video_observation_inputs,
    count_unique_videos, rank_chat_sources, retrieval_candidate_limit,
};


include!("frag_types_a.rs");
include!("frag_types_b.rs");



include!("frag_impl.rs");



include!("frag_post_a.rs");
include!("frag_post_b.rs");
