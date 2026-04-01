# Tasks: Post-Login Workspace Cache Reset

## Current State
Frontend auth-state now clears auth-sensitive GET caches when the auth storage scope changes, and regression coverage proves workspace bootstrap data is re-fetched after login/logout scope transitions. Frontend format, lint, check, and unit tests are green.

## Steps
- [x] Trace the post-login video-loading regression to the frontend auth/cache boundary
- [x] Add regression coverage for auth-scope cache invalidation
- [x] Clear auth-sensitive frontend GET caches when auth scope changes
- [x] Verify targeted frontend tests pass
