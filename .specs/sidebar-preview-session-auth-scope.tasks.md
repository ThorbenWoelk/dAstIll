# Tasks: Sidebar Preview Session Auth Scope

## Current State

Sidebar preview sessions are now scoped to the active auth boundary, and regression coverage plus targeted frontend lint/type/test checks are green. The only remaining repo-wide frontend gate issue is unrelated Prettier drift in pre-existing generated binding files.

## Steps

- [x] Trace the remaining video-loading regression to auth-insensitive sidebar preview-session keys
- [x] Scope sidebar preview sessions to the active auth storage boundary
- [x] Add regression coverage for anonymous vs authenticated preview sessions
- [x] Verify the affected frontend checks pass
