# Tasks: Chat Capability Sweep

## Current State
The evaluator, dataset, and first remediation pass are implemented. The live sweep exposed missing referent handling for prompts like `this video` and channel-specific questions, and the backend now supports `@` mention scoping plus highlight lookup.
Next step is to continue the 100-prompt sweep with the improved backend, cluster the remaining failures, and address the next highest-impact retrieval and answer-shaping gaps.

## Steps
- [x] Create spec and task files for the chat capability sweep.
- [x] Persist the canonical 100-prompt dataset with rubric metadata.
- [x] Implement the backend evaluator runner and report generation.
- [x] Add parser and scoring tests for the evaluator.
- [ ] Run the first full 100-prompt sweep against the live local backend.
- [x] Cluster initial failures by capability class and implement the first high-impact backend fixes.
- [ ] Rerun affected classes, then rerun the full sweep.
- [ ] Run final verification commands and record unresolved limitations.

## Decisions Made During Implementation
- The canonical execution path is the backend HTTP API, not Playwright or the chat UI.
- Each prompt uses a fresh conversation by default so prompt runs remain independent.
- Sweep artifacts are generated under `.artifacts/chat-capability/` and are not the source of truth.
- Prompt subsets will be rerunnable by capability class and prompt id to support the remediation loop.
- The first remediation batch fixed direct tool-loop action parsing, added highlight lookup as a first-class chat tool, and added backend `@mention` scoping for channels and videos.
- `@name` scopes simple channel/video handles or names. `@"Exact Title"` and `@{Exact Title}` scope exact titles with spaces.
