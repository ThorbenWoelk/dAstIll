import type { AddVideoResponse as BindingAddVideoResponse } from "./bindings/AddVideoResponse";
import type { AiHealthPayload as BindingAiHealthPayload } from "./bindings/AiHealthPayload";
import type { AiStatus as BindingAiStatus } from "./bindings/AiStatus";
import type { Channel as BindingChannel } from "./bindings/Channel";
import type { ChannelSnapshotPayload as BindingChannelSnapshotPayload } from "./bindings/ChannelSnapshotPayload";
import type { ChannelVideoPagePayload as BindingChannelVideoPagePayload } from "./bindings/ChannelVideoPagePayload";
import type { CleanTranscriptResponse as BindingCleanTranscriptResponse } from "./bindings/CleanTranscriptResponse";
import type { ContentStatus as BindingContentStatus } from "./bindings/ContentStatus";
import type { CreateHighlightRequest as BindingCreateHighlightRequest } from "./bindings/CreateHighlightRequest";
import type { Highlight as BindingHighlight } from "./bindings/Highlight";
import type { HighlightChannelGroup as BindingHighlightChannelGroup } from "./bindings/HighlightChannelGroup";
import type { HighlightSource as BindingHighlightSource } from "./bindings/HighlightSource";
import type { HighlightVideoGroup as BindingHighlightVideoGroup } from "./bindings/HighlightVideoGroup";
import type { SearchMatchPayload as BindingSearchMatchPayload } from "./bindings/SearchMatchPayload";
import type { SearchResponsePayload as BindingSearchResponsePayload } from "./bindings/SearchResponsePayload";
import type { SearchStatusPayload as BindingSearchStatusPayload } from "./bindings/SearchStatusPayload";
import type { SearchVideoResultPayload as BindingSearchVideoResultPayload } from "./bindings/SearchVideoResultPayload";
import type { Summary as BindingSummary } from "./bindings/Summary";
import type { SyncDepthPayload as BindingSyncDepthPayload } from "./bindings/SyncDepthPayload";
import type { Transcript as BindingTranscript } from "./bindings/Transcript";
import type { TranscriptRenderMode as BindingTranscriptRenderMode } from "./bindings/TranscriptRenderMode";
import type { Video as BindingVideo } from "./bindings/Video";
import type { WorkspaceBootstrapPayload as BindingWorkspaceBootstrapPayload } from "./bindings/WorkspaceBootstrapPayload";

type OptionalCompat<T, K extends keyof T> = Omit<T, K> & Partial<Pick<T, K>>;

export type ContentStatus = BindingContentStatus;
export type AiStatus = BindingAiStatus;
export type TranscriptRenderMode = BindingTranscriptRenderMode;
export type HighlightSource = BindingHighlightSource;

export type Channel = OptionalCompat<
  BindingChannel,
  | "handle"
  | "thumbnail_url"
  | "earliest_sync_date"
  | "earliest_sync_date_user_set"
>;

export type SyncDepth = BindingSyncDepthPayload;
export type ChannelSnapshot = BindingChannelSnapshotPayload;
export type ChannelVideoPage = BindingChannelVideoPagePayload;

export type SearchMatch = OptionalCompat<
  BindingSearchMatchPayload,
  "section_title"
>;
export type SearchResult = BindingSearchVideoResultPayload;
export type SearchStatus = Omit<
  BindingSearchStatusPayload,
  "retrieval_mode"
> & {
  retrieval_mode: "hybrid_exact" | "hybrid_ann" | "fts_only";
};
export type SearchResponse = Omit<
  BindingSearchResponsePayload,
  "source" | "results"
> & {
  source: "all" | "transcript" | "summary";
  results: SearchResult[];
};

export type WorkspaceBootstrap = Omit<
  BindingWorkspaceBootstrapPayload,
  "snapshot" | "search_status"
> & {
  snapshot: ChannelSnapshot | null;
  search_status: SearchStatus;
};

export type AiHealthResponse = BindingAiHealthPayload;
export type Video = BindingVideo;
export type AddVideoResult = BindingAddVideoResponse;
export type Transcript = OptionalCompat<
  BindingTranscript,
  "raw_text" | "formatted_markdown" | "render_mode" | "timed_text"
>;
export type CleanTranscriptResponse = BindingCleanTranscriptResponse;
export type Highlight = Omit<BindingHighlight, "id"> & { id: number };
export type CreateHighlightRequest = BindingCreateHighlightRequest;
export type HighlightVideoGroup = OptionalCompat<
  BindingHighlightVideoGroup,
  "thumbnail_url"
>;
export type HighlightChannelGroup = OptionalCompat<
  BindingHighlightChannelGroup,
  "channel_thumbnail_url"
>;
export type Summary = OptionalCompat<
  BindingSummary,
  "model_used" | "quality_score" | "quality_note" | "quality_model_used"
>;
