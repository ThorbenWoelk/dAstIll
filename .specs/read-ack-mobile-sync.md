# Spec: Read / acknowledge list sync (mobile)

## Problem

Marking a video as read does not always update the filtered sidebar list immediately on mobile. Occasionally, read/unread state appears wrong later.

## Root causes (verified)

1. **GET cache poisoning after invalidation**  
   `cachedGetRequest` removes in-flight entries from `inFlightGetRequests` when `invalidateChannelReadCache` runs, but the underlying `fetch` promise still resolves. Its `.then` handler can write a **stale** response back into `getResponseCache`, so the next snapshot request returns old `acknowledged` flags.

2. **Stale snapshot apply after acknowledge**  
   `loadChannelSnapshotWithRefresh` awaits `getChannelSnapshot` while the user toggles read. A snapshot that started loading before the toggle can still be applied afterward and overwrite the optimistic / server-updated list.

## Mitigations

- Per-path **cache epoch**: bump when invalidating; only write GET cache when the epoch at request start still matches.
- **Mutation epoch** in workspace sidebar state: bump after a successful acknowledge; `loadChannelSnapshotWithRefresh` skips `applySnapshot` when the epoch changed during `await loadSnapshot()`.

## Out of scope

- Server-side acknowledge correctness (assumed OK if cache is fresh).
