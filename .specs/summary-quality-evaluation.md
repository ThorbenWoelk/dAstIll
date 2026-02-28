# Spec: Asynchronous Summary Quality Evaluation

## Problem
Summaries are generated from transcripts, but there is no persisted quality signal that indicates coherence against source transcript content.

## Goals
- Evaluate each summary asynchronously against its transcript using Ollama model `qwen3-coder:480b-cloud`.
- Persist a quality score in range 0-10 and a brief incoherence note (when applicable).
- Surface score and note subtly in the summary UI.

## Non-Goals
- Blocking summary generation on evaluation.
- Complex historical tracking of multiple evaluation runs.

## Backend Requirements
- Extend summary storage with nullable evaluation fields.
- Reset evaluation fields whenever summary content changes.
- Add a background worker loop that finds summaries pending evaluation and stores results.
- Keep failures non-fatal for summary availability.

## API Requirements
- Include evaluation fields in summary responses.

## Frontend Requirements
- Display quality score and note subtly in summary mode.
- Handle pending/no-evaluation state without visual noise.

## Verification
- Backend tests for DB persistence/reset behavior.
- Local backend verification: format, clippy with warnings denied, tests, release build.
- Local frontend verification: type check and production build.
