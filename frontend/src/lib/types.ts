/** Backend-owned transport DTOs are re-exported for compatibility; declarations below stay frontend-only. */
export type {
  AddVideoResult,
  AiHealthResponse,
  AiStatus,
  Channel,
  ChannelSnapshot,
  ChannelVideoPage,
  ChatConversation,
  ChatConversationSummary,
  ChatMessage,
  ChatMessageStatus,
  ChatRole,
  ChatSource,
  ChatSuggestionItem,
  ChatTitleStatus,
  CleanTranscriptResponse,
  ContentStatus,
  CreateConversationRequest,
  CreateHighlightRequest,
  Highlight,
  HighlightChannelGroup,
  HighlightSource,
  HighlightVideoGroup,
  SearchMatch,
  SearchResponse,
  SearchResult,
  SearchSourceFilter,
  SearchSourceKind,
  SearchStatus,
  SendChatMessageRequest,
  Summary,
  SyncDepth,
  Transcript,
  TranscriptRenderMode,
  UserPreferences,
  Video,
  VideoInfo,
  VocabularyReplacement,
  WorkspaceBootstrap,
} from "./transport-types";

export type VideoTypeFilter = "all" | "long" | "short";
export type ChatRetrievalIntent =
  | "fact"
  | "synthesis"
  | "pattern"
  | "comparison";
export type QueueTab = "transcripts" | "summaries" | "evaluations";

export const OTHERS_CHANNEL_ID = "__others__";

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
