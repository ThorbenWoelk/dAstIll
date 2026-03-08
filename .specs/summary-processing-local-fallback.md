# Summary Processing During Local Fallback

## Problem
When cloud models hit rate limits, summary generation should continue on local Ollama models. In practice, background summary evaluation can fall back to the same local model path and occupy the single local inference slot, leaving actual summaries stuck in `loading`.

## Goal
Preserve local Ollama capacity for summary generation when cloud models are cooling down.

## Requirements
- Summary generation continues to use local fallback models when cloud models are rate-limited.
- Background summary evaluation does not consume local fallback capacity during cloud cooldown for cloud-primary evaluator configs.
- Existing local-primary evaluator configurations remain allowed.
- Regression tests cover the cooldown/fallback decision.

## Non-Goals
- Reworking the queue worker architecture.
- Changing summary prompt content.
- Increasing local Ollama concurrency.

## Design Notes
Treat summary evaluation as lower priority than summary generation. When the evaluator is configured with a cloud-primary model and cloud cooldown is active, or when that cloud model returns 429, defer evaluation instead of falling back to a local model.
