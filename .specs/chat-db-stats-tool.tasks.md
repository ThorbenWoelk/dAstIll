# Tasks: Chat Tool Loop

## Current State
Chat now enters through a model-driven tool loop where the model can choose `search_library`, `db_inspect`, or direct response, while backend validation and retrieval internals remain enforced server-side.
Verification completed cleanly with `cargo test`, `cargo build --release`, `bun run check`, `bun run build`, and `bun test tests`.

## Steps
- [x] Inspect current chat retrieval flow and DB access layer.
- [x] Implement a safe read-only `db_inspect` execution path and UI trace visibility.
- [x] Remove misleading chat sample copy that claims unsupported capability.
- [x] Make tool calling transparent in the chat UI.
- [x] Replace heuristic DB-question detection with model-planned tool selection plus server-side validation.
- [x] Replace the planner-heavy chat entry path with a model-driven tool loop.
- [x] Add a validated `search_library` tool that wraps current hybrid retrieval internals.
- [x] Run format, lint, test, and build verification.

## Decisions Made During Implementation
- Initial safe schema stays read-only and validated server-side, with `db_inspect` limited to approved operations and resources.
- `search_library` should hide keyword/vector retrieval internals from the model and keep those decisions in backend code.
- The tool loop drives the first response path; the older retrieval planner remains available as a backend fallback if the tool loop itself fails.
- Tool transparency is implemented through structured chat status payloads and a visible tool-call section in the chat meta panel.
