export type ContentStatus = "pending" | "loading" | "ready" | "failed";
export type VideoTypeFilter = "all" | "long" | "short";
export type AiStatus = "cloud" | "local_only" | "offline";

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
}

export interface CleanTranscriptResponse {
  content: string;
  preserved_text: boolean;
  attempts_used: number;
  max_attempts: number;
  timed_out: boolean;
}

export interface Summary {
  video_id: string;
  content: string;
  model_used?: string | null;
  quality_score?: number | null;
  quality_note?: string | null;
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
