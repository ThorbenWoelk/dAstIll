# Workspace Auth Scope Refresh

## Status

Accepted

## Context

The main workspace route keeps selected-channel video state in live in-memory caches and only hydrates bootstrap data on mount. After the auth scope changes inside an already-mounted page, the route can keep rendering anonymous channel/video state even after the browser auth context and backend data have changed.

## Decision

- Scope all in-memory workspace channel/video caches to the current auth storage boundary.
- When the workspace auth/storage scope changes, restore the new scoped workspace view state and reload bootstrap data for that scope.
- Clear the currently rendered video/content state before that reload so stale anonymous rows cannot be reused.

## Consequences

- Login and logout transitions refresh the mounted workspace without requiring a hard reload.
- Anonymous and authenticated channel/video cache entries no longer collide in-memory.
- The rendered workspace content always rebinds to the active auth scope before showing videos.
