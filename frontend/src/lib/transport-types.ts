import type { AddVideoResponse as BindingAddVideoResponse } from "./bindings/AddVideoResponse";
import type { AiHealthPayload as BindingAiHealthPayload } from "./bindings/AiHealthPayload";
import type { AiStatus as BindingAiStatus } from "./bindings/AiStatus";
import type { Channel as BindingChannel } from "./bindings/Channel";
import type { ChannelSnapshotPayload as BindingChannelSnapshotPayload } from "./bindings/ChannelSnapshotPayload";
import type { ChannelVideoPagePayload as BindingChannelVideoPagePayload } from "./bindings/ChannelVideoPagePayload";
import type { ChatConversation as BindingChatConversation } from "./bindings/ChatConversation";
import type { ChatConversationSummary as BindingChatConversationSummary } from "./bindings/ChatConversationSummary";
import type { ChatMessage as BindingChatMessage } from "./bindings/ChatMessage";
import type { ChatMessageStatus as BindingChatMessageStatus } from "./bindings/ChatMessageStatus";
import type { ChatRole as BindingChatRole } from "./bindings/ChatRole";
import type { ChatSource as BindingChatSource } from "./bindings/ChatSource";
import type { ChatTitleStatus as BindingChatTitleStatus } from "./bindings/ChatTitleStatus";
import type { CleanTranscriptResponse as BindingCleanTranscriptResponse } from "./bindings/CleanTranscriptResponse";
import type { ContentAvailability as BindingContentAvailability } from "./bindings/ContentAvailability";
import type { ContentItem as BindingContentItem } from "./bindings/ContentItem";
import type { ContentItemKind as BindingContentItemKind } from "./bindings/ContentItemKind";
import type { ContentPart as BindingContentPart } from "./bindings/ContentPart";
import type { ContentPartKind as BindingContentPartKind } from "./bindings/ContentPartKind";
import type { ContentStatus as BindingContentStatus } from "./bindings/ContentStatus";
import type { ContentProvider as BindingContentProvider } from "./bindings/ContentProvider";
import type { ContentSource as BindingContentSource } from "./bindings/ContentSource";
import type { CreateConversationRequest as BindingCreateConversationRequest } from "./bindings/CreateConversationRequest";
import type { CreateHighlightRequest as BindingCreateHighlightRequest } from "./bindings/CreateHighlightRequest";
import type { Highlight as BindingHighlight } from "./bindings/Highlight";
import type { HighlightChannelGroup as BindingHighlightChannelGroup } from "./bindings/HighlightChannelGroup";
import type { HighlightSource as BindingHighlightSource } from "./bindings/HighlightSource";
import type { HighlightVideoGroup as BindingHighlightVideoGroup } from "./bindings/HighlightVideoGroup";
import type { LibraryBootstrapPayload as BindingLibraryBootstrapPayload } from "./bindings/LibraryBootstrapPayload";
import type { LibrarySectionKind as BindingLibrarySectionKind } from "./bindings/LibrarySectionKind";
import type { LibrarySectionSummary as BindingLibrarySectionSummary } from "./bindings/LibrarySectionSummary";
import type { MediaAsset as BindingMediaAsset } from "./bindings/MediaAsset";
import type { MediaAssetKind as BindingMediaAssetKind } from "./bindings/MediaAssetKind";
import type { ProviderMetadataEntry as BindingProviderMetadataEntry } from "./bindings/ProviderMetadataEntry";
import type { SourceArchetype as BindingSourceArchetype } from "./bindings/SourceArchetype";
import type { SourceBackingKind as BindingSourceBackingKind } from "./bindings/SourceBackingKind";
import type { SearchMatchPayload as BindingSearchMatchPayload } from "./bindings/SearchMatchPayload";
import type { SearchResponsePayload as BindingSearchResponsePayload } from "./bindings/SearchResponsePayload";
import type { SearchSourceKind as BindingSearchSourceKind } from "./bindings/SearchSourceKind";
import type { SearchStatusPayload as BindingSearchStatusPayload } from "./bindings/SearchStatusPayload";
import type { SearchVideoResultPayload as BindingSearchVideoResultPayload } from "./bindings/SearchVideoResultPayload";
import type { SendChatMessageRequest as BindingSendChatMessageRequest } from "./bindings/SendChatMessageRequest";
import type { Summary as BindingSummary } from "./bindings/Summary";
import type { SubscriptionContainerKind as BindingSubscriptionContainerKind } from "./bindings/SubscriptionContainerKind";
import type { SyncDepthPayload as BindingSyncDepthPayload } from "./bindings/SyncDepthPayload";
import type { Transcript as BindingTranscript } from "./bindings/Transcript";
import type { TranscriptRenderMode as BindingTranscriptRenderMode } from "./bindings/TranscriptRenderMode";
import type { UserPreferences as BindingUserPreferences } from "./bindings/UserPreferences";
import type { Video as BindingVideo } from "./bindings/Video";
import type { VideoInfo as BindingVideoInfo } from "./bindings/VideoInfo";
import type { VocabularyReplacement as BindingVocabularyReplacement } from "./bindings/VocabularyReplacement";
import type { WebsiteFolder as BindingWebsiteFolder } from "./bindings/WebsiteFolder";
import type { WorkspaceBootstrapPayload as BindingWorkspaceBootstrapPayload } from "./bindings/WorkspaceBootstrapPayload";

/** Generated bindings own backend transport DTOs; compatibility aliases here keep current frontend ergonomics. */
type Compat<T, R> = Omit<T, keyof R> & R;

export type ContentStatus = BindingContentStatus;
export type ContentAvailability = BindingContentAvailability;
export type AiStatus = BindingAiStatus;
export type TranscriptRenderMode = BindingTranscriptRenderMode;
export type HighlightSource = BindingHighlightSource;
export type SearchSourceKind = BindingSearchSourceKind;
export type SearchSourceFilter = "all" | SearchSourceKind;
export type ChatRole = BindingChatRole;
export type ChatMessageStatus = BindingChatMessageStatus;
export type ChatTitleStatus = BindingChatTitleStatus;
export type LibrarySectionKind = BindingLibrarySectionKind;
export type ContentProvider = BindingContentProvider;
export type SourceArchetype = BindingSourceArchetype;
export type SourceBackingKind = BindingSourceBackingKind;
export type SubscriptionContainerKind = BindingSubscriptionContainerKind;
export type ContentItemKind = BindingContentItemKind;
export type ContentPartKind = BindingContentPartKind;
export type MediaAssetKind = BindingMediaAssetKind;
export type ProviderMetadataEntry = BindingProviderMetadataEntry;

export type Channel = Compat<
  BindingChannel,
  {
    handle?: BindingChannel["handle"];
    thumbnail_url?: BindingChannel["thumbnail_url"];
    earliest_sync_date?: BindingChannel["earliest_sync_date"];
    earliest_sync_date_user_set?: BindingChannel["earliest_sync_date_user_set"];
  }
>;

export type SyncDepth = BindingSyncDepthPayload;

export type Video = Compat<
  BindingVideo,
  {
    thumbnail_url?: BindingVideo["thumbnail_url"];
    retry_count?: BindingVideo["retry_count"];
    quality_score?: BindingVideo["quality_score"] | null;
  }
>;

export type ChannelSnapshot = Compat<
  BindingChannelSnapshotPayload,
  {
    sync_depth: SyncDepth;
    videos: Video[];
  }
>;

export type ChannelVideoPage = Compat<
  BindingChannelVideoPagePayload,
  {
    videos: Video[];
  }
>;

export type LibrarySectionSummary = Compat<
  BindingLibrarySectionSummary,
  {
    container_kinds: SubscriptionContainerKind[];
    backing_kinds: SourceBackingKind[];
  }
>;

export type ContentSource = Compat<
  BindingContentSource,
  {
    container_id?: BindingContentSource["container_id"];
    subtitle?: BindingContentSource["subtitle"];
    thumbnail_url?: BindingContentSource["thumbnail_url"];
    item_count?: number | null;
    unread_count?: number | null;
    provider_metadata: ProviderMetadataEntry[];
  }
>;

export type ContentItem = Compat<
  BindingContentItem,
  {
    thumbnail_url?: BindingContentItem["thumbnail_url"];
    available_parts: ContentPartKind[];
    available_media_assets: MediaAssetKind[];
  }
>;

export type ContentPart = Compat<
  BindingContentPart,
  {
    render_mode?: BindingContentPart["render_mode"];
    text_length?: number | null;
  }
>;

export type MediaAsset = Compat<
  BindingMediaAsset,
  {
    mime_type?: BindingMediaAsset["mime_type"];
  }
>;

export type WebsiteFolder = BindingWebsiteFolder;

export type LibraryBootstrap = Compat<
  BindingLibraryBootstrapPayload,
  {
    sections: LibrarySectionSummary[];
    sources: ContentSource[];
    selected_source_id?: BindingLibraryBootstrapPayload["selected_source_id"];
    selected_source?: ContentSource | null;
    selected_items: ContentItem[];
    website_folders: WebsiteFolder[];
  }
>;

export type WorkspaceBootstrap = Compat<
  BindingWorkspaceBootstrapPayload,
  {
    channels: Channel[];
    snapshot: ChannelSnapshot | null;
    library: LibraryBootstrap;
    search_status: SearchStatus;
  }
>;

export type AiHealthResponse = BindingAiHealthPayload;
export type AddVideoResult = Compat<BindingAddVideoResponse, { video: Video }>;

export type Transcript = Compat<
  BindingTranscript,
  {
    raw_text?: BindingTranscript["raw_text"];
    formatted_markdown?: BindingTranscript["formatted_markdown"];
    render_mode?: BindingTranscript["render_mode"];
  }
>;

export type CleanTranscriptResponse = BindingCleanTranscriptResponse;
export type CreateHighlightRequest = BindingCreateHighlightRequest;

export type Highlight = Compat<
  BindingHighlight,
  {
    id: number;
  }
>;

export type HighlightVideoGroup = Compat<
  BindingHighlightVideoGroup,
  {
    thumbnail_url?: BindingHighlightVideoGroup["thumbnail_url"];
    highlights: Highlight[];
  }
>;

export type HighlightChannelGroup = Compat<
  BindingHighlightChannelGroup,
  {
    channel_thumbnail_url?: BindingHighlightChannelGroup["channel_thumbnail_url"];
    videos: HighlightVideoGroup[];
  }
>;

export type Summary = Compat<
  BindingSummary,
  {
    model_used?: BindingSummary["model_used"];
    quality_score?: BindingSummary["quality_score"] | null;
    quality_note?: BindingSummary["quality_note"];
    quality_model_used?: BindingSummary["quality_model_used"];
  }
>;

export type VideoInfo = Compat<
  BindingVideoInfo,
  {
    description?: BindingVideoInfo["description"];
    thumbnail_url?: BindingVideoInfo["thumbnail_url"];
    channel_name?: BindingVideoInfo["channel_name"];
    channel_id?: BindingVideoInfo["channel_id"];
    published_at?: BindingVideoInfo["published_at"];
    duration_iso8601?: BindingVideoInfo["duration_iso8601"];
    duration_seconds?: number | null;
    view_count?: number | null;
  }
>;

export type SearchMatch = Compat<
  BindingSearchMatchPayload,
  {
    source: Exclude<SearchSourceFilter, "all">;
    section_title?: BindingSearchMatchPayload["section_title"];
  }
>;

export type SearchResult = Compat<
  BindingSearchVideoResultPayload,
  {
    matches: SearchMatch[];
  }
>;

export type SearchResponse = Compat<
  BindingSearchResponsePayload,
  {
    source: SearchSourceFilter;
    results: SearchResult[];
  }
>;

export type SearchStatus = Compat<
  BindingSearchStatusPayload,
  {
    retrieval_mode: "hybrid_exact" | "hybrid_ann" | "fts_only";
  }
>;

export type ChatSource = Compat<
  BindingChatSource,
  {
    section_title?: BindingChatSource["section_title"];
  }
>;

export type ChatMessage = Compat<
  BindingChatMessage,
  {
    sources: ChatSource[];
    prompt_tokens?: number;
    completion_tokens?: number;
    total_duration_ns?: number;
  }
>;

export type ChatConversationSummary = Compat<
  BindingChatConversationSummary,
  {
    title?: BindingChatConversationSummary["title"];
  }
>;

export type ChatConversation = Compat<
  BindingChatConversation,
  {
    title?: BindingChatConversation["title"];
    messages: ChatMessage[];
  }
>;

/** Temporary compatibility DTO until the backend generates a binding for suggestions. */
export type ChatSuggestionItem = {
  kind: "channel" | "video";
  id: string;
  label: string;
  subtitle?: string | null;
};

export type CreateConversationRequest = Compat<
  BindingCreateConversationRequest,
  {
    title?: BindingCreateConversationRequest["title"];
  }
>;

export type SendChatMessageRequest = Compat<
  BindingSendChatMessageRequest,
  {
    deep_research?: BindingSendChatMessageRequest["deep_research"];
  }
>;

export type VocabularyReplacement = BindingVocabularyReplacement;

export type UserPreferences = Compat<
  BindingUserPreferences,
  {
    channel_sort_mode: "custom" | "alpha" | "newest";
    vocabulary_replacements: VocabularyReplacement[];
  }
>;
