# Loading Performance Overhaul

**Linear:** (none)

## Problem

The app feels slow to open and slow to switch into working state. The frontend currently performs multiple startup requests and repeated follow-up fetches for the same channel selection, while the backend serializes database access through a single shared connection. That increases time-to-interactive and makes the UI feel blocked under load.

## Goal

Make the primary workspace and queue views reach useful content noticeably faster by reducing startup round trips, avoiding duplicate fetches, and improving backend throughput for hot read paths.

## Requirements

- Initial workspace load should require fewer network round trips before the first channel and first page of videos are visible.
- Queue and workspace pages should avoid redundant requests for data that can be returned together or reused locally.
- Hot backend read paths for channels and channel videos should handle concurrent requests without being serialized behind a single database connection.
- Video listing queries should use indexes appropriate for the active filters so larger datasets remain responsive.
- Changes must be covered by automated tests for the new loading behavior and verified with local build/test commands.

## Non-Goals

- Rewriting the application architecture or moving away from the current Svelte/Rust stack.
- Reworking summary/transcript generation latency from external tools and APIs.
- Visual redesign work unrelated to loading and responsiveness.

## Design Considerations

The fastest gains are on the critical path: combine startup data into a single backend response, reduce duplicate client fetches, and remove backend request contention for read-heavy endpoints. Database improvements should favor simple schema/index changes and connection handling over risky feature rewrites.

## Open Questions

- No explicit numeric performance budget was provided, so success is relative to the current local behavior.
- External YouTube and model calls can still dominate some flows; this work focuses first on the app-controlled path before those background operations complete.
