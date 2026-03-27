# Channel Video List Performance

## Problem

Channel videos in the shared sidebar feel slow to appear and slow to expand because the channel route boots through separate list + sync-depth requests, expanded sections eagerly drain the entire channel history, and normal channel snapshot/list reads still scan the whole stored video set on the backend.

## Goal

Make the first visible channel videos appear faster across all routes using per-channel preview mode, and make expanded channel sections feel incremental instead of blocking on full-history loads.

## Requirements

- The selected channel's initial preview rows must hydrate from a single bootstrap response on hard loads of the channel route.
- Per-channel expanded sections must load incrementally and append additional pages on demand instead of draining the whole channel history up front.
- Expanded channel sections must render a bounded DOM window once enough rows are loaded.
- Normal channel snapshot/list reads must avoid full-store video scans on the backend hot path.
- Shared cache/session state must preserve already loaded preview rows and paging state across route transitions.
- The new behavior must be covered by targeted backend tests, frontend unit tests, and route/shared-sidebar verification.

## Non-Goals

- Restoring inner scroll position inside an expanded channel list across route changes.
- Perfectly optimizing the `__others__` virtual channel path in this pass.
- Adding new production analytics or telemetry.

## Design Considerations

- Reuse the existing workspace bootstrap endpoint instead of adding a separate channel bootstrap API.
- Keep preview mode lightweight and only virtualize expanded paged lists once enough rows are loaded.
- Prefer no new frontend dependency for virtualization; keep the implementation inside the existing Svelte sidebar component.
- Accept nullable exact video counts on snapshot payloads to avoid blocking on an additional aggregate query.

## Open Questions

- None.
