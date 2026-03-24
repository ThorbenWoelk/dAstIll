/** Typed definitions for the analytics starter event set. */

export type AnalyticsEventName =
  | "channel_snapshot_loaded"
  | "transcript_ensure_requested"
  | "transcript_ensure_completed"
  | "video_opened"
  | "video_acknowledged_changed"
  | "content_mode_changed"
  | "highlight_created"
  | "summary_opened"
  | "summary_closed"
  | "summary_heartbeat";

interface BaseEvent {
  event: AnalyticsEventName;
  /** Stable client-generated id for server-side idempotency. */
  event_id: string;
  /** ISO-8601 timestamp, set by the tracker at send time. */
  ts: string;
  /** Unique session identifier (tab-scoped, regenerated on page load). */
  session_id: string;
}

export interface ChannelSnapshotLoadedEvent extends BaseEvent {
  event: "channel_snapshot_loaded";
  channel_id: string;
  video_count: number;
}

export interface TranscriptEnsureRequestedEvent extends BaseEvent {
  event: "transcript_ensure_requested";
  video_id: string;
  channel_id: string;
}

export interface TranscriptEnsureCompletedEvent extends BaseEvent {
  event: "transcript_ensure_completed";
  video_id: string;
  channel_id: string;
  success: boolean;
}

export interface VideoOpenedEvent extends BaseEvent {
  event: "video_opened";
  video_id: string;
  channel_id: string;
}

export interface VideoAcknowledgedChangedEvent extends BaseEvent {
  event: "video_acknowledged_changed";
  video_id: string;
  channel_id: string;
  acknowledged: boolean;
}

export interface ContentModeChangedEvent extends BaseEvent {
  event: "content_mode_changed";
  video_id: string;
  channel_id: string;
  from_mode: string;
  to_mode: string;
}

export interface HighlightCreatedEvent extends BaseEvent {
  event: "highlight_created";
  video_id: string;
  channel_id: string;
  source: string;
}

export interface SummaryOpenedEvent extends BaseEvent {
  event: "summary_opened";
  video_id: string;
  channel_id: string;
  summary_id: string;
}

export interface SummaryClosedEvent extends BaseEvent {
  event: "summary_closed";
  video_id: string;
  channel_id: string;
  summary_id: string;
  read_time_ms: number;
  active_time_ms: number;
  max_scroll_depth_pct: number;
}

export interface SummaryHeartbeatEvent extends BaseEvent {
  event: "summary_heartbeat";
  video_id: string;
  channel_id: string;
  summary_id: string;
  active_time_ms: number;
  max_scroll_depth_pct: number;
}

export type AnalyticsEvent =
  | ChannelSnapshotLoadedEvent
  | TranscriptEnsureRequestedEvent
  | TranscriptEnsureCompletedEvent
  | VideoOpenedEvent
  | VideoAcknowledgedChangedEvent
  | ContentModeChangedEvent
  | HighlightCreatedEvent
  | SummaryOpenedEvent
  | SummaryClosedEvent
  | SummaryHeartbeatEvent;
