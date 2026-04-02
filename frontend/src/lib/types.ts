import type {
  AddVideoResult as TransportAddVideoResult,
  AiHealthResponse as TransportAiHealthResponse,
  AiStatus as TransportAiStatus,
  Channel as TransportChannel,
  ChannelSnapshot as TransportChannelSnapshot,
  ChannelVideoPage as TransportChannelVideoPage,
  CleanTranscriptResponse as TransportCleanTranscriptResponse,
  ContentStatus as TransportContentStatus,
  CreateHighlightRequest as TransportCreateHighlightRequest,
  Highlight as TransportHighlight,
  HighlightChannelGroup as TransportHighlightChannelGroup,
  HighlightSource as TransportHighlightSource,
  HighlightVideoGroup as TransportHighlightVideoGroup,
  SearchMatch as TransportSearchMatch,
  SearchResponse as TransportSearchResponse,
  SearchResult as TransportSearchResult,
  SearchStatus as TransportSearchStatus,
  Summary as TransportSummary,
  SyncDepth as TransportSyncDepth,
  Transcript as TransportTranscript,
  TranscriptRenderMode as TransportTranscriptRenderMode,
  Video as TransportVideo,
  WorkspaceBootstrap as TransportWorkspaceBootstrap,
} from "./transport-types";

export type ContentStatus = TransportContentStatus;
export type VideoTypeFilter = "all" | "long" | "short";
export type AiStatus = TransportAiStatus;
export type TranscriptRenderMode = TransportTranscriptRenderMode;
export type HighlightSource = TransportHighlightSource;
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
export type ChatSuggestionKind = "channel" | "video";

export const OTHERS_CHANNEL_ID = "__others__";

export type Channel = TransportChannel;
export type SyncDepth = TransportSyncDepth;
export type ChannelSnapshot = TransportChannelSnapshot;
export type ChannelVideoPage = TransportChannelVideoPage;
export type WorkspaceBootstrap = TransportWorkspaceBootstrap;
export type AiHealthResponse = TransportAiHealthResponse;

export type QueueTab = "transcripts" | "summaries" | "evaluations";

export type Video = TransportVideo;
export type AddVideoResult = TransportAddVideoResult;
export type Transcript = TransportTranscript;
export type CleanTranscriptResponse = TransportCleanTranscriptResponse;
export type Highlight = TransportHighlight;
export type CreateHighlightRequest = TransportCreateHighlightRequest;
export type HighlightVideoGroup = TransportHighlightVideoGroup;
export type HighlightChannelGroup = TransportHighlightChannelGroup;
export type Summary = TransportSummary;

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

export type SearchMatch = TransportSearchMatch;
export type SearchResult = TransportSearchResult;
export type SearchResponse = TransportSearchResponse;
export type SearchStatus = TransportSearchStatus;

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

export interface ChatSuggestionItem {
  kind: ChatSuggestionKind;
  id: string;
  label: string;
  subtitle?: string | null;
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
