# Tasks: Local Auth Recent Sign-In

## Current State
The frontend now forces a refreshed Firebase ID token before posting `/auth/session` and signs out any live anonymous Firebase user before Google popup sign-in. Regression coverage plus broader frontend verification are passing.

## Steps
- [x] Locate the `"Recent sign-in required"` path and identify the frontend/backend exchange involved
- [x] Add a regression test for popup sign-in when the first exchanged token is stale
- [x] Patch the frontend session exchange to request a fresh Firebase ID token
- [x] Verify the targeted auth tests pass
