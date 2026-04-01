# Sidebar Preview Session Auth Scope

## Status

Accepted

## Context

The workspace sidebar stores per-channel preview collections in an in-memory session map keyed only by a route-level string such as `workspace-sidebar-navigation`. After anonymous browsing, login, or a repaired library migration, the sidebar can restore the stale anonymous preview session even though the backend now returns the authenticated user's real channel snapshots.

## Decision

- Treat sidebar preview sessions as auth-scoped client state.
- Derive the sidebar preview session key from the route-level base key plus the current auth storage scope.
- Add regression coverage proving anonymous and authenticated preview sessions do not collide.

## Consequences

- Video preview state does not bleed across anonymous and authenticated sessions.
- Repaired or newly loaded authenticated libraries can render fresh sidebar videos immediately after auth changes.
- Existing per-route preview-session behavior remains unchanged within a single auth scope.
