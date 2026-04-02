# Tasks: Logfire AI Telemetry

## Current State
Implemented. Logfire now forwards all `ERROR` events plus AI-target telemetry, startup sets explicit backend service metadata, chat reply lifecycle logs are in place, and Ollama-backed search operations emit low-volume spans and completion/failure events. `cargo fmt --all`, `cargo check`, and focused logging tests are green.

## Steps
- [x] Create spec and task files for Logfire AI telemetry.
- [x] Move the Logfire filter policy into shared logging code and include all `ERROR` events.
- [x] Add low-volume AI telemetry around chat and direct Ollama-backed search paths.
- [x] Run targeted backend verification and record the result.

## Decisions Made During Implementation

- The Logfire project name remains external to code and is determined by the configured token.
- Telemetry will favor one span per AI operation plus concise retry/error events over verbose per-token or per-chunk logging.
- All `ERROR`-level events are sent to Logfire regardless of target, while `INFO` and `WARN` remain scoped to AI-related targets to keep volume down.
