# Logfire AI Telemetry

## Problem

The backend already has partial Logfire integration, but AI-related telemetry is inconsistent. Chat spans exist in some paths, direct Ollama usage outside chat is not covered end-to-end, and error-level events are filtered too narrowly to give reliable failure visibility.

## Goal

Capture low-noise Logfire telemetry for:

- chat AI interactions
- every Ollama-backed AI run
- all error-level events

without logging raw prompts, streamed tokens, secrets, or high-volume request bodies.

## Requirements

- Keep Logfire volume intentionally low.
- Treat the Logfire project as external configuration via `LOGFIRE_TOKEN`; do not hardcode project identifiers in code.
- Send all `ERROR`-level events to Logfire regardless of target.
- Send AI-related spans and supporting info logs from chat, Ollama, search, summarizer, and summary-evaluator paths.
- Add structured AI span fields that are useful for debugging:
  - operation name
  - model id
  - input size or count
  - latency
  - outcome markers such as completion, retry, fallback, or failure
- Do not log raw prompts, transcript bodies, token streams, API keys, or full upstream error bodies.

## Non-Goals

- Full request tracing for every backend endpoint.
- Logging every token streamed to chat clients.
- Introducing a separate telemetry service abstraction.

## Design

### Filter policy

Move Logfire target selection into shared logging code so it can express a simple rule:

- always include `ERROR` events
- include AI-related targets at `INFO` and above

### AI coverage

Use one span per high-level AI run:

- chat reply lifecycle
- streamed chat generation
- Ollama prompt-with-fallback operations
- search embedding, rerank, and HyDE operations

Add explicit `error!` and concise `warn!` events at failure and retry boundaries.

## Verification

- `cargo test`
- `cargo check`
