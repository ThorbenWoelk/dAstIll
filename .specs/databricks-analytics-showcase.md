# Databricks Analytics Showcase

## Problem

The app currently has no clear analytics story that demonstrates how product usage can flow into Databricks for downstream transformation pipelines and dashboards. The showcase needs to focus on a few high-value behaviors instead of broad generic product analytics, and it needs a technically defensible definition of summary read time across desktop and mobile.

## Goal

Create a focused analytics implementation and demo story in which app events are tracked in the Svelte frontend, ingested into Databricks, transformed into curated marts, and visualized in dashboards that explain:

- where content processing slows down
- how content moves from unacknowledged to acknowledged and into highlights usage
- which summaries receive the deepest and longest reading attention

## Requirements

- Use a custom first-party tracker in the Svelte app.
- Track only the chosen use cases:
  - content processing efficiency
  - acknowledged/unacknowledged progression
  - highlights usage
  - summary read time / summary attention depth
- Send events through a small ingestion endpoint and land them directly in a Databricks Delta Bronze table through the SQL Statement Execution API.
- Keep the first version intentionally small with a constrained starter event set.
- Instrument app events using existing first-party domain entities such as `channel`, `video`, `summary`, `acknowledged`, `highlights`, and `contentMode`.
- Define summary read time with both `read_time_ms` and `active_time_ms`.
- Treat `active_time_ms` as the primary dashboard metric for reading attention.
- Read-time tracking must work on desktop and mobile web.
- Mobile tracking must account for touch events, visibility changes, backgrounding, and unreliable unload behavior.
- Do not track raw transcript or summary text in analytics events.
- Databricks models must include:
  - a Bronze raw event table written directly by the backend
  - a Silver typed and deduplicated event table
  - Gold marts for processing, engagement progression, and summary read time
- Dashboard outputs must include:
  - processing efficiency dashboard
  - acknowledgment and highlights dashboard
  - summary consumption dashboard

## Non-Goals

- Broad retention, acquisition, or generic product analytics outside the selected use cases
- Third-party analytics SDKs in the first version
- Tracking raw content bodies such as transcript text or summary text
- Full BI-tool expansion beyond an initial Databricks-native dashboard setup
- Over-instrumenting every user action in the app

## Design Considerations

- A custom tracker is preferred because the app already has domain-specific concepts that do not map cleanly to generic analytics tools.
- The read-time metric must be credible. Raw "summary open time" is not enough because idle tabs, backgrounded sessions, and mobile interruptions would inflate the numbers.
- The ingestion path itself is part of the showcase, so the architecture should clearly show browser events flowing into Databricks rather than disappearing into a third-party analytics product.
- Because the app is targeting Databricks Free Edition for the showcase, the first implementation should avoid S3, Auto Loader, and DLT and instead write directly into Delta through a SQL warehouse.
- The event model should stay small and explicit at first. A compact, high-quality schema is more valuable than a large noisy event set.
- Mobile web behavior matters from day one because visibility and session-ending behavior differ substantially from desktop.

## Read Time Definition

- A summary session starts when the user enters `contentMode = "summary"` for a specific `video_id` and `summary_id`.
- A summary session ends when the user leaves summary mode, switches videos, navigates away, backgrounds the page, closes the tab, or the relevant summary component unmounts.
- `read_time_ms` is the total wall-clock elapsed time between `summary_opened` and `summary_closed`.
- `active_time_ms` only accrues while:
  - the current content mode is still `summary`
  - the same `summary_id` is still active
  - the page is visible
  - the user has been active recently
- Recent user activity is refreshed by signals such as `scroll`, `click`, `keydown`, text selection, pointer movement, `touchstart`, `touchmove`, and `touchend`.
- If no qualifying activity occurs for 30 seconds, active time accumulation pauses until the next activity signal.
- Track `max_scroll_depth_pct` during the session and include it in the session close payload.
- On mobile, flush the session on `visibilitychange` to hidden rather than relying only on page unload.
- If close-event delivery is not reliable enough, add `summary_heartbeat` checkpoints every 10-15 seconds during active reading.
- Dashboard ranking should use `active_time_ms` or an engagement-weighted score rather than raw `read_time_ms`.

## Starter Event Set

- `channel_snapshot_loaded`
- `transcript_ensure_requested`
- `transcript_ensure_completed`
- `video_opened`
- `video_acknowledged_changed`
- `content_mode_changed`
- `highlight_created`
- `summary_opened`
- `summary_closed`
- optional later: `summary_heartbeat`

## Open Questions

- Which backend deployment environment should receive the Databricks credentials first: local only, Cloud Run, or both?
- Should summary ranking in dashboards use `active_time_ms` directly or an engagement-weighted score such as `active_time_ms * max_scroll_depth_pct`?
