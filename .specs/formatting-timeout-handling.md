# Spec: Graceful handling for transcript formatting timeouts

## Problem

Production transcript formatting for long videos can run into the 300 second upstream request limit. Today the frontend also aborts at exactly 300 seconds, so users see a client-side timeout message instead of the app returning a controlled `timed_out` result.

## Evidence

- `frontend/src/lib/api.ts` aborts transcript formatting requests after 5 minutes and rewrites the error to `"Formatting timed out after 5 minutes."`
- `backend/src/services/summarizer.rs` also gives transcript formatting a 300 second hard timeout.
- `backend/src/services/ollama.rs` builds the cloud Ollama HTTP client with a 300 second request timeout.
- A live request against production for video `2JjKn7uhKqY` returned `HTTP 504` with body `upstream request timeout` after `300.081009` seconds.
- A short transcript request for video `dsttKIZ3XwA` completed successfully in `19.736636` seconds.

## Goal

Make long transcript formatting fail gracefully before the upstream 300 second cutoff, and avoid the frontend masking the backend result with its own identical timeout boundary.

## Recommended approach

### 1. Leave backend response headroom

Reduce the transcript-formatting hard timeout so the handler can return its existing `timed_out: true` JSON response before the upstream request deadline is reached.

### 2. Give the frontend a small grace window

Keep a client-side safeguard, but let it expire slightly after the server-side formatting limit so structured backend responses win when available.

### 3. Remove exact-minute wording from the UI

The current copy says "after 5 minutes", which becomes false as soon as backend and frontend timeouts diverge. Use a generic time-limit message instead.

## Files to change

- `backend/src/services/ollama.rs`
- `backend/src/services/summarizer.rs`
- `frontend/src/lib/api.ts`
- `frontend/src/routes/+page.svelte`
- `frontend/tests/api.test.ts`

## Acceptance criteria

- Long transcript formatting returns a controlled timeout result before the upstream 300 second request limit.
- The frontend no longer masks the backend outcome with an equal 300 second abort timer.
- The UI timeout copy is generic and remains accurate if limits change again.
