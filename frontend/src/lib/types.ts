export type ContentStatus = "pending" | "loading" | "ready" | "failed";
export type VideoTypeFilter = "all" | "long" | "short";
export type AiStatus = "cloud" | "local_only" | "offline";
export type TranscriptRenderMode = "plain_text" | "markdown";
export type HighlightSource = "transcript" | "summary";
export type SearchSourceFilter = "all" | "transcript" | "summary";

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
