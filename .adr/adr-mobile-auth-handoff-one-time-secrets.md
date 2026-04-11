# ADR: Mobile Auth Handoff Uses Split One-Time Secrets

## Status

Accepted

## Context

The Android Tauri shell cannot complete Google sign-in inside the embedded WebView.
The app uses a browser handoff so sign-in can finish in the system browser and return
control to the shell.

The earlier handoff stored reusable Google tokens under a caller-provided session id and
returned them over `GET`. That widened the trust boundary: leaked ids in logs, URLs, or
crash reports could expose reusable credentials.

## Decision

The mobile auth handoff now uses:

- a server-minted `handoff_id`
- a browser-only completion secret
- an app-only redeem secret
- creator binding derived from the originating caller context
- one-shot redeem semantics

The browser completes the handoff with the completion secret.
The Android shell redeems the finished handoff exactly once with the redeem secret.
Reusable Google tokens are never returned over `GET`.

## Consequences

Positive:

- removes token-bearing `GET` from the flow
- blocks session-id guessing and simple replay
- narrows token exposure to one completion path and one redeem path

Tradeoffs:

- backend still holds raw Google tokens in memory until redeem or expiry
- frontend/browser flow now carries a fragment-based completion secret and session storage state

## Directive

If this flow changes again, prefer server-side token exchange over storing third-party reusable
tokens in memory. Do not reintroduce token-bearing `GET` endpoints or client-chosen handoff ids.
