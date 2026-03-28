# Multi-User Firebase Auth

## Summary

Continue the Firebase auth migration from the current partial state and convert the app from global single-user state to user-scoped state for channels, conversations, highlights, preferences, acknowledged video state, and manual-video `Others` membership.

## Current State

- Firebase SDK setup, session endpoints, anonymous bootstrap, Google sign-in UI, auth controller, server auth context, and proxy groundwork already exist.
- Backend request identity exists via `AccessContext`, but data storage and most handlers are still global.
- `Others` should remain visible and keep the current product behavior, but become strictly user-scoped.

## Goals

- Keep canonical content global: channels metadata, videos, transcripts, summaries, search sources.
- Move user-owned state to per-user storage.
- Scope workspace/library reads by authenticated user identity.
- Keep anonymous users read-only and non-persistent.
- Preserve existing public API response shapes.

## Non-Goals

- Anonymous-to-authenticated state merge.
- Replacing Firebase session architecture.
- Changing the canonical content storage model.

## Key Decisions

- `Others` remains visible in the UI.
- `Others` is backed by explicit per-user manual-video membership, not global fallback behavior.
- Public payload types remain stable; tenancy changes stay internal.

## Migration Path

### Preferences

Legacy preferences stored in `dastill_preferences/user` are automatically copied to `dastill_preferences/{uid}` on the first authenticated request from each user. The migration is idempotent and silent:

1. Check if `dastill_preferences/{uid}` exists; if yes, skip.
2. Check if `dastill_preferences/user` exists (legacy document).
3. If legacy exists, copy to `dastill_preferences/{uid}` with normalized content.
4. Subsequent reads/writes use the user-scoped document.

This runs in `security.rs:load_authenticated_allowed_channel_ids()` alongside the seeded channel subscription.

### Acknowledged Video State

Legacy `acknowledged` field on Firestore `dastill_videos` documents is **not migrated**. There is no user identity mapping in the legacy single-user model. After migration:

- New acknowledged state is stored in `user-video-states/{user_id}/{video_id}.json` (S3).
- Legacy `acknowledged` values remain on Firestore videos for historical reference but are ignored by new reads.
- Users must re-acknowledge videos after migration.

### Channels, Conversations, Highlights

- Channel subscriptions: New feature with `user-channel-subscriptions/{user_id}/{channel_id}.json` - no legacy data.
- Conversations: Already user-scoped to `user-conversations/{scope}/` - uses Firebase `uid` or `"anonymous"`.
- Highlights: Already user-scoped to `user-highlights/{user_id}/` - no migration needed.
