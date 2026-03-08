# Ollama Indicator Status

## Problem
The UI currently treats AI health as a boolean and shows the same "ready" state whenever the Ollama endpoint responds, even when cloud models are rate limited and the app is operating on local fallback only.

## Goal
Expose enough backend status to let the UI distinguish between cloud-capable, local-only fallback, and offline modes.

## Requirements
- The backend AI health response distinguishes `cloud`, `local_only`, and `offline`.
- Existing `ai_available`/`available` booleans remain usable for feature gating.
- The workspace and queue header indicators render green for `cloud`, grey for `local_only`, and red for `offline`.
- Verification gates pass locally after the change.

## Non-Goals
- Changing summary generation behavior.
- Reworking queue or worker lifecycle behavior.
- Adding new providers or model routing paths.

## Design Notes
Classify the indicator state from the configured primary/fallback models plus the shared cloud cooldown. Preserve the existing boolean availability signal so buttons stay enabled when only local fallback is available.
