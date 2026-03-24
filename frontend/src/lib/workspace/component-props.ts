import type {
  Channel,
  CreateHighlightRequest,
  Highlight,
  QueueTab,
  TranscriptRenderMode,
  Video,
  VideoInfo,
  VideoTypeFilter,
} from "$lib/types";
import type { ChannelSyncDepthState } from "$lib/channel-view-cache";
import type {
  AcknowledgedFilter,
  ChannelSortMode,
  DistillationStatusCopy,
  QueueStats,
  WorkspaceContentMode,
} from "$lib/workspace/types";

export interface WorkspaceSidebarShellProps {
  collapsed: boolean;
  width?: number;
  mobileVisible: boolean;
  onToggleCollapse: () => void;
}

export interface WorkspaceSidebarChannelState {
  channels: Channel[];
  selectedChannelId: string | null;
  loadingChannels: boolean;
  addingChannel: boolean;
  channelSortMode: ChannelSortMode;
  canDeleteChannels?: boolean;
}

export interface WorkspaceSidebarChannelActions {
  onChannelSortModeChange: (next: ChannelSortMode) => void;
  onAddChannel: (input: string) => Promise<boolean> | boolean;
  onSelectChannel: (channelId: string) => Promise<void> | void;
  onOpenChannelOverview?: (channelId: string) => Promise<void> | void;
  onDeleteChannel: (channelId: string) => Promise<void> | void;
  onDeleteAccessRequired?: () => void;
  onReorderChannels: (nextOrder: string[]) => void;
  onChannelUpdated?: (channel: Channel) => void;
}

export interface WorkspaceSidebarVideoState {
  videos: Video[];
  pendingSelectedVideo?: Video | null;
  selectedVideoId: string | null;
  selectedChannel: Channel | null;
  loadingVideos: boolean;
  refreshingChannel: boolean;
  hasMore: boolean;
  historyExhausted: boolean;
  backfillingHistory: boolean;
  videoTypeFilter: VideoTypeFilter;
  acknowledgedFilter: AcknowledgedFilter;
  syncDepth: ChannelSyncDepthState | null;
  offset: number;
  allowLoadedVideoSyncDepthOverride: boolean;
}

/** When `forceReload` is true, the route should load content even if `videoId` is already selected (e.g. after `selectChannel` set the id). */
export type WorkspaceVideoSelectContext = {
  forceReload?: boolean;
};

export interface WorkspaceSidebarVideoActions {
  onSelectVideo: (
    videoId: string,
    context?: WorkspaceVideoSelectContext,
  ) => Promise<void> | void;
  onSelectChannelVideo?: (
    channelId: string,
    videoId: string,
  ) => Promise<void> | void;
  onLoadMoreVideos: () => Promise<void> | void;
  onVideoTypeFilterChange: (value: VideoTypeFilter) => Promise<void> | void;
  onAcknowledgedFilterChange: (
    value: AcknowledgedFilter,
  ) => Promise<void> | void;
  onClearAllFilters?: () => Promise<void> | void;
}

export interface WorkspaceContentSelection {
  mobileVisible: boolean;
  selectedChannel: Channel | null;
  selectedVideo: Video | null;
  selectedVideoId: string | null;
  contentMode: WorkspaceContentMode;
}

export interface WorkspaceContentState {
  loadingContent: boolean;
  editing: boolean;
  aiAvailable: boolean;
  summaryQualityScore: number | null;
  summaryQualityNote: string | null;
  summaryModelUsed: string | null;
  summaryQualityModelUsed: string | null;
  videoInfo: VideoInfo | null;
  contentHtml: string;
  contentText: string;
  transcriptRenderMode: TranscriptRenderMode;
  contentHighlights: Highlight[];
  selectedVideoHighlights: Highlight[];
  selectedVideoYoutubeUrl: string | null;
  draft: string;
  formattingContent: boolean;
  formattingVideoId: string | null;
  /** Video ids with an in-flight summary regeneration (supports concurrent retries). */
  regeneratingSummaryVideoIds: string[];
  revertingContent: boolean;
  revertingVideoId: string | null;
  resettingVideo: boolean;
  resettingVideoId: string | null;
  creatingHighlight: boolean;
  creatingHighlightVideoId: string | null;
  deletingHighlightId: number | null;
  canRevertTranscript: boolean;
  showRevertTranscriptAction: boolean;
  formattingNotice: string | null;
  formattingNoticeVideoId: string | null;
  formattingNoticeTone: "info" | "success" | "warning";
  /** When set, transcript/summary view scrolls to this excerpt text once (from `cite` URL param). */
  citationScrollText: string | null;
}

export interface WorkspaceContentActions {
  onBack: () => void;
  onSetMode: (mode: WorkspaceContentMode) => Promise<void> | void;
  onStartEdit: () => void;
  onCancelEdit: () => void;
  onSaveEdit: () => Promise<void> | void;
  onCleanFormatting: () => Promise<void> | void;
  onRegenerateSummary: () => Promise<void> | void;
  onRevertTranscript: () => Promise<void> | void;
  onResetVideo: () => Promise<void> | void;
  onDraftChange: (value: string) => void;
  onToggleAcknowledge: () => Promise<void> | void;
  onCreateHighlight: (payload: CreateHighlightRequest) => Promise<void> | void;
  onDeleteHighlight?: (highlightId: number) => Promise<void> | void;
  onShowChannels: () => void;
  onShowVideos: () => void;
  /** Called after citation deep-link scroll succeeds; should strip `cite` / `chunk` from the URL. */
  onCitationScrollConsumed?: () => void;
}

export interface WorkspaceOverlaysState {
  errorMessage: string | null;
  showDeleteConfirmation: boolean;
  showDeleteAccessPrompt: boolean;
  showResetVideoConfirmation: boolean;
}

export interface WorkspaceOverlaysActions {
  onDismissError: () => void;
  onConfirmDelete: () => void;
  onCancelDelete: () => void;
  onConfirmAccessPrompt: () => Promise<void> | void;
  onCancelAccessPrompt: () => void;
  onConfirmResetVideo: () => Promise<void> | void;
  onCancelResetVideo: () => void;
}

export interface QueueListItem {
  video: Video;
  distillationStatus: DistillationStatusCopy;
}

export interface QueueContentPanelState {
  mobileVisible: boolean;
  selectedChannel: Channel | null;
  selectedChannelId: string | null;
  queueTab: QueueTab;
  queueStats: QueueStats;
  failedTranscriptVideos?: Video[];
  retryingTranscriptVideoId?: string | null;
  effectiveEarliestSyncDate?: string | null;
  earliestSyncDateInput: string;
  savingSyncDate: boolean;
  refreshingChannel: boolean;
}

export interface QueueContentPanelActions {
  onBack: () => void;
  onQueueTabChange: (value: QueueTab) => void;
  onSaveSyncDate: (value: string) => Promise<void> | void;
  onRetryTranscript?: (videoId: string) => Promise<void> | void;
}
