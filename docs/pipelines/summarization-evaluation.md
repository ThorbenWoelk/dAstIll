# Summarization and Evaluation

## Summarizer

The summarizer service handles:

- summary generation from a cleaned transcript
- transcript cleaning that normalizes formatting while preserving the speaker's wording

Both tasks call Ollama `/api/generate` with the configured summarizer model.

If the primary summarizer is cloud-backed and rate-limited, the service uses the configured fallback
model when present. Local fallback runs immediately and has its own capacity profile. Without a
fallback, summarization waits for the cloud cooldown to expire.

The summarizer reports availability to the frontend:

- primary model reachability
- fallback activity
- cooldown state

The workspace header uses this status for the AI availability indicator.

## Summary Evaluator

The evaluator is stricter than the summarizer.

Policy:

- the evaluator model must be cloud-backed
- the model name must indicate at least 31B parameters
- the evaluator model must differ from the summarizer model
- evaluator cloud cooldown pauses evaluation

Backend startup fails when the summarizer and evaluator use the same model.

The evaluator compares a generated summary against the canonical transcript on:

- faithfulness: summary claims are supported by the transcript
- completeness: the summary covers the transcript's substantive editorial content

The model returns structured JSON:

- `status`: `scored` or `unscorable`
- `faithfulness_score`, `completeness_score`, and `final_score`
- `defects[]` with type, severity, affected summary claim, and transcript anchor
- `unscorable_reason`
- `tags[]` as transcript-supported metadata

Rust validates the response against a backend-owned JSON schema before storage.

Stored fields:

- `quality_score`
- `quality_note`
- `quality_model_used`
- `summary_tags`

`quality_note` preserves axis scores and defect evidence. Unscorable inputs store a note without a
numeric score.

Score policy:

- `7` or above is acceptable
- `6` or below can requeue the summary for regeneration
- `videos.retry_count` caps regeneration attempts
