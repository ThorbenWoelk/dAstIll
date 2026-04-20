# ADR: Summary Evaluator Uses Structured Quality Output

## Status

Accepted.

## Context

Summary evaluation previously asked the judge model for one numeric score, one free-form
note, and tags. A recent audit showed that this made the evaluation hard to trust:

- many summaries received perfect scores without any note
- the scoring guide made `7` both acceptable and a score with several defects
- low and high scores did not expose whether the issue was faithfulness, completeness, or
  transcript quality
- bad source material had no way to be marked unscorable

The existing summary storage and frontend API already expose `quality_score`,
`quality_note`, `quality_model_used`, and `summary_tags`. Replacing that storage shape
would require a broader migration.

## Decision

The evaluator prompt now asks for structured JSON with:

- `status`: `scored` or `unscorable`
- faithfulness, completeness, and final numeric scores when scored
- evidence-backed defects for non-perfect scored summaries
- an unscorable reason when source material cannot be judged
- tags as metadata, not defect explanations

The backend keeps the existing summary storage shape for this increment:

- scored results store `final_score` in `quality_score`
- structured axis scores and defects are rendered into `quality_note`
- unscorable results store `quality_score = null` and a note
- evaluation queue filters treat a score or note plus evaluated tags as completed quality
  state

Invalid numeric scores now fail parsing instead of being clamped.

## Consequences

- Evaluation output is more auditable without a storage migration.
- Unscorable summaries can leave the evaluation queue without triggering regeneration.
- Downstream code must treat `quality_score` as nullable.
- A future migration can persist axis scores and defects as first-class fields if the UI
  needs structured display or analytics.
