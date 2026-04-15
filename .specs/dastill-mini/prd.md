# dastill-mini PRD

## Problem

The full dAstIll workspace currently depends on the SQL-backed video catalog and search stack. When the app is paused behind the maintenance landing, signed-in users have no lightweight path to continue reading summaries from their subscribed channels.

This creates two problems:

1. The product has no low-cost fallback mode when the main workspace is unavailable or intentionally paused.
2. Read status is coupled to the main workspace access path instead of a durable user-state store that can survive Turso unavailability.

## Goal

Ship `dastill-mini`, a minimal signed-in summary reader that keeps the maintenance landing as the public entry point, lets authenticated users open a focused reader, and stores read status in S3-backed per-user state instead of the SQL-backed video catalog path.

## Non-goals

- Replace the main workspace, queue, chat, search, or highlights surfaces.
- Introduce a new design system or a separate frontend shell aesthetic.
- Rebuild the full content catalog or search index around S3-only access.
- Add new dependencies or infrastructure.

## Users or actors

- Signed-in users who want to continue reading summaries during maintenance mode.
- Operators who need a small, durable product mode that stays useful when the main workspace is paused.
- Future maintainers who need a clear boundary between `dastill-mini` and the main workspace.

## Requirements

### Landing behavior

- The landing page must remain the maintenance page in maintenance mode.
- The maintenance page must expose a clear sign-in or continue CTA into `dastill-mini`.
- Signed-in users returning to the landing page should be able to continue into the mini reader without leaving the maintenance framing.

### Mini reader route

- Add a dedicated signed-in route for `dastill-mini`.
- The mini route must follow the repo design system: muted, minimal, content-first, and low chrome.
- The route must center a subscribed-channel dropdown above the active summary.
- The route must show one active summary at a time with previous and next navigation.
- The route must support empty states for no subscriptions and no summaries.

### Data loading

- `dastill-mini` must derive channel and summary availability from S3-backed user subscriptions, `video-info`, and `summaries` data.
- The mini route must not depend on the SQL-backed search or workspace bootstrap APIs.
- Summary navigation order should be deterministic and newest-first within the selected channel.

### Read status

- Read status must be stored in the S3-backed per-user video state store.
- The minimal reader must mark the active summary as read on explicit user action.
- The API path for read status should no longer require the SQL-backed video lookup to succeed before writing user state.
- The mini reader must render read state from the S3-backed user state surface.

## Risks and open questions

- S3-backed summary discovery may be slower than the SQL-backed workspace catalog on very large libraries.
- Existing UI flows may still assume the older `acknowledged` naming even after the write path is moved off the SQL gate.
- The first increment may need to tolerate an on-demand scan-based summary list before a dedicated summary catalog exists.
