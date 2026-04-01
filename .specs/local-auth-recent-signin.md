# Local Auth Recent Sign-In

## Status
Accepted

## Context

Local sign-in can fail after the app has already established an anonymous Firebase session. The frontend then opens Google sign-in and exchanges the popup user's ID token with `/auth/session`, but the backend rejects the token with `Recent sign-in required.` when the exchanged token is stale relative to the backend's freshness window.

This is user-facing and blocks the local authentication flow, so the fix needs to be narrow and backed by regression coverage.

## Decision

- Reproduce the stale-token path in frontend auth-state tests.
- Force a fresh ID token when exchanging the popup-authenticated Firebase user for the server session cookie.
- Keep the backend freshness guard in place and avoid broadening the accepted sign-in window.

## Consequences

- Local Google sign-in no longer depends on a cached or pre-popup token state.
- The backend still enforces recent authentication before minting a session cookie.
- Regression coverage documents the expected client behavior for popup sign-in.
