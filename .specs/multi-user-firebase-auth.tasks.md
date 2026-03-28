# Tasks: Multi-User Firebase Auth

## Current State
Migration complete. Preferences are auto-migrated from legacy `dastill_preferences/user` to user-scoped documents on first authenticated access. Acknowledged video state is not migrated (no user identity mapping in legacy data). All user-owned state is now properly scoped. Authentication UI added to left sidebar showing sign-in/sign-out options.

## Steps
- [x] Inspect mission state, current repo state, and existing auth groundwork
- [x] Create backend/user-scoped storage primitives for subscriptions, preferences, highlights, manual-video membership, and acknowledged state
- [x] Complete backend access context and scope resolution
- [x] Rebuild channel, bootstrap, and video handlers around user scope
- [x] Replace global preferences/highlights/chat ownership with user scope
- [x] Namespace backend read cache by user scope
- [x] Update frontend proxy/auth gating and client storage namespacing
- [x] Add migration path for legacy global user-owned data
- [x] Run targeted verification and fix regressions

## Decisions Made During Implementation
- `Others` stays visible and behaves like the current product, but becomes user-scoped.
- `Others` grants access only to explicit manual-video memberships, not to whole unsubscribed channels.
- Public API payload shapes should remain unchanged.
- Frontend proxy runtime config is now isolated in `frontend/src/lib/server/auth-runtime.ts` so the route test no longer imports the full Firebase Admin module graph.
- Frontend localStorage, sessionStorage, theme state, chat model selection, shell layout, and IndexedDB workspace cache are now scoped by auth identity using `frontend/src/lib/auth-storage.ts`.
- Frontend channel deletion gating was switched from `isOperator` to authenticated-user capability checks where the new backend contract is expected to allow unsubscribe-style deletes.
- Backend channel update/delete routes are no longer operator-only; only maintenance endpoints such as search rebuild and channel refresh/backfill remain operator-gated.
- Backend read-cache keys now include user scope for channels, workspace bootstrap, channel snapshots, and sync-depth payloads.
- Backend chat conversations now persist under `user-conversations/{scope}/...`, where authenticated users use Firebase `uid` and unauthenticated traffic currently falls back to an `anonymous` scope placeholder.
- Search results are now filtered against `AccessContext.allowed_channel_ids` and `AccessContext.allowed_other_video_ids`, so `Others` grants access by exact video membership only.
- Legacy preferences (`dastill_preferences/user`) are auto-migrated to `dastill_preferences/{uid}` on first authenticated access via `db::migrate_legacy_preferences()` called from `security.rs`.
- Acknowledged video state is NOT migrated - legacy `acknowledged` values on Firestore documents have no user identity mapping and are abandoned. Users must re-acknowledge after migration.
