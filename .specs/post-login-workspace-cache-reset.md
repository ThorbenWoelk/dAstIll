# Post-Login Workspace Cache Reset

## Status
Accepted

## Context

The frontend caches GET responses for workspace bootstrap and channel reads by URL alone. After a user moves between anonymous and authenticated sessions, the cache can return responses fetched under the previous auth scope. That can leave the workspace showing stale or empty video data immediately after login.

## Decision

- Treat auth-scope transitions as a cache boundary for frontend GET data.
- Clear the frontend GET cache whenever auth moves between anonymous/bootstrap and authenticated scopes.
- Add regression coverage for both the cache helper and the auth-state transition that triggers it.

## Consequences

- Post-login and post-logout workspace loads always re-fetch auth-sensitive bootstrap and channel data.
- Frontend cache reuse remains intact within a single auth scope.
- The fix stays narrow and does not change backend auth behavior.
