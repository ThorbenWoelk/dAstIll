# Tasks: Start App Fast Fail

## Current State
`start_app.sh` now survives missing optional `.env` keys under `set -e`, reports early child-process exits during readiness checks, and the stack was verified to start successfully on spare ports (`4643/4644/4673`).

## Steps
- [x] Reproduce the current startup failure and capture the backend exit mode
- [x] Patch `start_app.sh` to fail fast when a launched service exits before readiness
- [x] Update local development docs for Firestore credentials and ADC fallback
- [x] Verify script behavior with syntax checks and a controlled startup run
