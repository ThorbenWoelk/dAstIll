# Tasks: Workspace Auth Scope Refresh

## Current State

The mounted workspace now refreshes correctly across auth-scope changes: in-memory channel/video caches are auth-scoped, the workspace re-fetches bootstrap data when auth changes, and a live browser auth-transition check rendered authenticated videos successfully.

## Steps

- [x] Trace the remaining loading issue to the mounted workspace auth-transition path
- [x] Scope in-memory workspace channel/video caches to the auth boundary
- [x] Reload workspace state and bootstrap data when the auth scope changes
- [x] Verify the fix with targeted checks and a live browser reproduction
