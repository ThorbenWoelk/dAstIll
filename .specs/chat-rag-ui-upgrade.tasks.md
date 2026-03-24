# Tasks: Chat RAG + UI Upgrade

## Current State
Implemented: planner `needs_retrieval`, skip path, 3rd retrieval pass, adaptive synthesis limits, hybrid citations, icon-only chat input, opaque chat surfaces, tests green.

## Steps
- [x] Add planner-driven retrieval mode in backend chat service so follow-up turns can skip retrieval when context is sufficient.
- [x] Refactor retrieval limits into runtime-configurable/adaptive controls and implement multi-stage evidence expansion with stop conditions.
- [x] Update grounding/synthesis flow to support large evidence sets through staged summarization rather than fixed small excerpt caps.
- [x] Implement hybrid citation rendering in chat messages: in-text markers when present plus compact inline fallback refs.
- [x] Replace card-grid source citations in chat with minimal inline link references and hover titles.
- [x] Refactor chat input to a single icon-only action button covering send/loading/cancel states.
- [x] Align chat UI surfaces/tooltips/spacing with `DESIGN.md` and remove non-compliant translucent/frosted overlay patterns in chat.
- [x] Fix detected symmetry issues in chat layout and interaction states.
- [ ] Validate with targeted tests plus manual screenshot verification across key states (idle, sending, streaming, canceled, cited response, long thread).

## Decisions Made During Implementation
- Server decides retrieval necessity per turn.
- Retrieval scale uses adaptive multi-stage strategy.
- Citation style uses hybrid rendering (in-text markers + compact inline fallback refs).
- Planner JSON includes `needs_retrieval`; conversation transcript is passed into the planner prompt; skip is ignored without a prior assistant message.
- Third retrieval pass runs for pattern/comparison with budget >= 16 when pass 2 still needs more and heuristic pass-3 queries exist.
- Candidate pool cap raised to 48; synthesis raw excerpt cap scales with plan budget (8–48); video observation groups scale via `synthesis_video_cap`.
