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

The evaluator compares a generated summary against the transcript on:

- faithfulness: summary claims are supported by the transcript
- completeness: the summary covers the transcript's substantive editorial content

The evaluator reads the same vocabulary-corrected transcript as the summarizer. Without this, a
correct name from the vocabulary list would look like a hallucination.

The backend sends a JSON schema with each request (Ollama `format`). The model returns:

- `status`: `scored` or `unscorable`
- `faithfulness_score`, `completeness_score`, and `final_score`
- `defects[]` with type, severity, affected summary claim, and transcript anchor
- `unscorable_reason` and `unscorable_cause` (`summary` or `transcript`)
- `tags[]` as transcript-supported metadata

Models do not always follow the schema. The backend accepts these gaps:

- a defect without `severity` counts as `minor`
- a defect without `type` is listed as `issue`
- a defect with neither a claim nor an anchor is dropped
- missing axis scores are left out of the note
- a missing `final_score` is the lower of the two axis scores
- a score below 10 without defects is kept; the note says no defects were listed
- an unscorable answer without a reason gets a placeholder reason

The backend rejects an answer with no usable score, a score outside 0-10, or an unknown `status`.

Stored fields:

- `quality_score`
- `quality_note`
- `quality_model_used`
- `summary_tags`

`quality_note` preserves axis scores and defect evidence. Unscorable inputs store a note without a
numeric score.

A result is stored only if the summary text is still the one that was evaluated. If the summary was
replaced while the evaluator ran, the result is dropped and the new summary is evaluated later.

### Evaluation Order And Failures

The evaluation worker picks summaries with fewer failed attempts first, then newer videos first.
Summaries that keep failing cannot block newer ones.

Failed attempts are counted per summary text. A new summary text starts a new count.

- An unusable model answer (for example invalid JSON or no score) counts toward the limit. After
  `3` unusable answers the summary gets an "Unscorable" note with the last error, and the worker
  stops picking it.
- A failed request (timeout, HTTP error) is retried without a limit. It only moves the summary
  behind others.
- An unavailable evaluator or an active cloud cooldown pauses evaluation and counts nothing.

### Automatic Regeneration

The evaluation worker sets the video's summary back to `pending` when:

- the score is `6` or below, or
- the evaluator says the summary itself is unscorable (`unscorable_cause: summary`)

An unscorable transcript (show notes, wrong language, corrupted text) does not trigger
regeneration, because a new summary would not help.

Hand-written summaries (`model_used: manual`) are evaluated but never regenerated automatically.

Before an automatic regeneration, the backend keeps a copy of the current summary and its score.
When the new summary is evaluated, the better-scored one stays. If the new one scored lower, the
old summary comes back.

A per-video auto-regeneration counter caps automatic regenerations at `2` (see
[Runtime Limits](/operations/runtime-limits#content-processing-limits)). `videos.retry_count` does
not limit regeneration; it only limits queue processing failures. Saving a manual summary or
resetting the video sets the counter back to `0`.

Startup repair does not cancel a queued regeneration. It sets a stored summary back to `ready` only
when the row is stuck in `loading` or marked `failed`.

### Manual Regeneration

Regenerating a summary (`POST /api/videos/{id}/summary/regenerate`) generates the new summary
first. The stored summary is replaced only when generation succeeds, and search is updated for the
new text. If generation fails, the old summary stays and the video stays `ready`.
