# Summarization

## Summarizer

The summarizer service handles:

- summary generation from a cleaned transcript
- transcript cleaning that normalizes formatting while preserving the speaker's wording

Both tasks call Ollama `/api/generate` with the configured summarizer model.

If the primary summarizer is cloud-backed and rate-limited, the service uses the configured fallback
model when present. Local fallback runs immediately and has its own capacity profile. Without a
fallback, summarization waits for the cloud cooldown to expire.
Cooldown values live in [Runtime Limits](/operations/runtime-limits#cooldowns).

The summarizer reports availability to the frontend:

- primary model reachability
- fallback activity
- cooldown state

The backend serves this status at `/api/health/ai`.

### Output Checks

Before a summary is stored, the backend cleans the model's reply:

- removes `<think>` blocks and any reasoning before the first summary heading
- removes a code fence around the whole reply
- keeps only the last full set of sections when the model wrote a draft and then a final version
- removes a leading title line and closing remarks such as "Let me know if..."

The cleaned summary must have an `## At a glance` section (or the older `## TL;DR`) and a
`## Key Points` section. It is rejected when:

- it is empty
- it contains Chinese or Japanese characters and the transcript does not
- it looks cut off, or Ollama reports that it stopped at the output token limit

A rejected summary is not stored. The attempt counts as failed, and the queue retries it.

### Long And Non-English Transcripts

- The summary request timeout grows with transcript length, up to 900 seconds.
- Local models get a context window (`num_ctx`) large enough for the whole prompt. Cloud models
  ignore this setting.
- Word counts for scripts written without spaces, such as Chinese or Thai, are estimated from the
  number of characters.

### Vocabulary Replacements

Vocabulary replacements only match whole words. A replacement is skipped where the target text is
already present, so `Claude` → `Claude Code` does not turn `Claude Code` into `Claude Code Code`.

### Description-Only Transcripts

Some videos have no captions, and the transcript tool returns the video description instead.
When most of the transcript's words come from the description and there are no timed caption
segments, the video is marked as having no transcript and is not summarized.

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
- `videos.retry_count` caps regeneration attempts; retry limits live in [Runtime Limits](/operations/runtime-limits#content-processing-limits)
