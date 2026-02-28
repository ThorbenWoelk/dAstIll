# Rig Ollama Summarizer Migration

## Problem
Summary generation currently calls Ollama directly via `reqwest` against `/api/generate`. This has produced queue worker failures and duplicates model-client behavior that is already standardized in sibling repos using `rig`.

## Goal
Use a `rig` Ollama agent for summary generation while keeping the existing backend flow and status behavior intact.

## Requirements
- Summarizer uses `rig` Ollama provider for text generation.
- Existing `OLLAMA_URL` and `OLLAMA_MODEL` configuration remain supported.
- Existing summary endpoint and queue worker behavior remain compatible.
- Build/test gates pass locally after migration.

## Non-Goals
- Reworking queue worker architecture.
- Changing transcript extraction behavior.
- Introducing new external model providers.

## Design Notes
Reuse the current prompt contract, keep `is_available` probe with lightweight HTTP `/api/tags`, and migrate only the generation call to `rig`.
