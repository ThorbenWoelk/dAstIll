export type ContentStatus = "pending" | "loading" | "ready" | "failed";
export type VideoTypeFilter = "all" | "long" | "short";
export type AiStatus = "cloud" | "local_only" | "offline";
export type TranscriptRenderMode = "plain_text" | "markdown";
export type HighlightSource = "transcript" | "summary";
export type SearchSourceFilter = "all" | "transcript" | "summary";
export type ChatRole = "system" | "user" | "assistant";
export type ChatMessageStatus =
  | "completed"
  | "streaming"
  | "cancelled"
  | "rejected"
  | "failed";
export type ChatTitleStatus = "idle" | "generating" | "ready" | "manual";
export type ChatRetrievalIntent =
  | "fact"
  | "synthesis"
  | "pattern"
  | "comparison";

export const OTHERS_CHANNEL_ID = "__others__";

export interface Channel {
  id: string;
  handle?: string | null;
  name: string;
  thumbnail_url?: string | null;
  added_at: string;
  earliest_sync_date?: string | null;
  earliest_sync_date_user_set?: boolean;
}

export interface SyncDepth {
  earliest_sync_date: string | null;
  earliest_sync_date_user_set: boolean;
  derived_earliest_ready_date: string | null;
}

export interface ChannelSnapshot {
  channel_id: string;
  sync_depth: SyncDepth;
  /** Total videos for this channel in storage (ignores type / read / queue filters). */
  channel_video_count: number;
  videos: Video[];
}

export interface WorkspaceBootstrap {
  ai_available: boolean;
  ai_status: AiStatus;
  channels: Channel[];
  selected_channel_id: string | null;
  snapshot: ChannelSnapshot | null;
  search_status: SearchStatus;
}

export interface AiHealthResponse {
  available: boolean;
  status: AiStatus;
}

export type QueueTab = "transcripts" | "summaries" | "evaluations";

export interface Video {
  id: string;
  channel_id: string;
  title: string;
  thumbnail_url?: string | null;
  published_at: string;
  is_short: boolean;
  transcript_status: ContentStatus;
  summary_status: ContentStatus;
  acknowledged: boolean;
  retry_count?: number;
  quality_score?: number | null;
}

export interface AddVideoResult {
  video: Video;
  target_channel_id: string;
  already_exists: boolean;
}

export interface Transcript {
  video_id: string;
  raw_text?: string | null;
  formatted_markdown?: string | null;
  render_mode?: TranscriptRenderMode | null;
}

export interface CleanTranscriptResponse {
  content: string;
  preserved_text: boolean;
  attempts_used: number;
  max_attempts: number;
  timed_out: boolean;
}

export interface Highlight {
  id: number;
  video_id: string;
  source: HighlightSource;
  text: string;
  prefix_context: string;
  suffix_context: string;
  created_at: string;
}

export interface CreateHighlightRequest {
  source: HighlightSource;
  text: string;
  prefix_context: string;
  suffix_context: string;
}

export interface HighlightVideoGroup {
  video_id: string;
  title: string;
  thumbnail_url?: string | null;
  published_at: string;
  highlights: Highlight[];
}

export interface HighlightChannelGroup {
  channel_id: string;
  channel_name: string;
  channel_thumbnail_url?: string | null;
  videos: HighlightVideoGroup[];
}

export interface Summary {
  video_id: string;
  content: string;
  model_used?: string | null;
  quality_score?: number | null;
  quality_note?: string | null;
  quality_model_used?: string | null;
}

export interface VideoInfo {
  video_id: string;
  watch_url: string;
  title: string;
  description?: string | null;
  thumbnail_url?: string | null;
  channel_name?: string | null;
  channel_id?: string | null;
  published_at?: string | null;
  duration_iso8601?: string | null;
  duration_seconds?: number | null;
  view_count?: number | null;
}

export interface SearchMatch {
  source: Exclude<SearchSourceFilter, "all">;
  section_title?: string | null;
  snippet: string;
  score: number;
}

export interface SearchResult {
  video_id: string;
  channel_id: string;
  channel_name: string;
  video_title: string;
  published_at: string;
  matches: SearchMatch[];
}

export interface SearchResponse {
  query: string;
  source: SearchSourceFilter;
  results: SearchResult[];
}

export interface SearchStatus {
  available: boolean;
  model: string;
  dimensions: number;
  pending: number;
  indexing: number;
  ready: number;
  failed: number;
  total_sources: number;
  total_chunk_count: number;
  embedded_chunk_count: number;
  vector_index_ready: boolean;
  retrieval_mode: "hybrid_exact" | "hybrid_ann" | "fts_only";
}

export interface ChatSource {
  video_id: string;
  channel_id: string;
  channel_name: string;
  video_title: string;
  source_kind: Exclude<SearchSourceFilter, "all">;
  section_title?: string | null;
  snippet: string;
  score: number;
  /** Indexed search chunk id; used with cite= for deep links. */
  chunk_id: string;
  retrieval_pass?: number | null;
}

export interface ChatRetrievalPlan {
  intent: ChatRetrievalIntent;
  label: string;
  budget: number;
  max_per_video: number;
  queries: string[];
  expansion_queries: string[];
  rationale?: string | null;
  /** When true, the server answered from conversation only (no new retrieval). */
  skip_retrieval?: boolean;
  /** When true, this turn used the app’s maximum excerpt budget and wider query fan-out. */
  deep_research?: boolean;
}

export interface ChatToolStatus {
  name: string;
  label: string;
  state: string;
  input: string;
  output?: string | null;
}

export interface ChatToolCall {
  name: string;
  label: string;
  state: string;
  input: string;
  output?: string | null;
}

export interface ChatStreamStatus {
  stage: string;
  label?: string | null;
  detail?: string | null;
  decision?: string | null;
  plan?: ChatRetrievalPlan | null;
  tool?: ChatToolStatus | null;
}

export interface ChatMessage {
  id: string;
  role: ChatRole;
  content: string;
  sources: ChatSource[];
  status: ChatMessageStatus;
  created_at: string;
  /** Set on assistant turns produced by the configured chat model. */
  model?: string;
  prompt_tokens?: number;
  completion_tokens?: number;
  total_duration_ns?: number;
}

export interface ChatConversationSummary {
  id: string;
  title?: string | null;
  title_status: ChatTitleStatus;
  created_at: string;
  updated_at: string;
}

export interface ChatConversation extends ChatConversationSummary {
  messages: ChatMessage[];
}

export interface CreateConversationRequest {
  title?: string | null;
}

export interface SendChatMessageRequest {
  content: string;
  /** Ask the backend to use maximum library retrieval for this message. */
  deep_research?: boolean;
  /** Ollama cloud model id from chat config; server picks default if omitted. */
  model?: string;
}

export interface UserPreferences {
  channel_order: string[];
  channel_sort_mode: "custom" | "alpha" | "newest";
  vocabulary_replacements: VocabularyReplacement[];
}

export interface VocabularyReplacement {
  from: string;
  to: string;
  added_at: string;
}
