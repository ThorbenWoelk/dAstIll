# Tasks: Start App Runtime Readiness

## Current State
`./start_app.sh --detach` now starts cleanly on the default local path. Startup prunes malformed
legacy Firestore video rows so refresh/gap workers can repopulate them, and the script validates
the frontend-proxied workspace bootstrap before reporting success.

## Steps
- [x] Reproduce the startup failure and capture the backend error mode
- [x] Make Firestore video reads tolerate malformed legacy rows
- [x] Make `start_app.sh` validate a real workspace bootstrap response
- [x] Verify the stack starts cleanly again and update this task state
