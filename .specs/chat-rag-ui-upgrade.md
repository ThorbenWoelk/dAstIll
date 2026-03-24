# Chat RAG + UI Upgrade

## Problem

The current chat flow always runs retrieval on every user turn, which makes simple follow-up questions slower and more expensive than necessary. Retrieval is also constrained by fixed limits that can block deeper answers when broader evidence is needed. On the UI side, source citations are visually heavy, chat controls are split across separate send/cancel states, and parts of the chat experience do not fully match the design system or symmetry expectations.

## Goal

Deliver a chat experience where retrieval is used intentionally instead of automatically, long-evidence questions can scale to many documents through adaptive stages, citations are minimal and inline like modern RAG chat, and the chat UI is visually consistent with `DESIGN.md` with verified symmetry.

## Requirements

- Retrieval decision must be server-controlled per turn:
  - the assistant may answer a follow-up from existing conversation context without starting a new retrieval pass
  - retrieval must still run when the assistant determines fresh evidence is useful
- Chat retrieval must support adaptive multi-stage evidence expansion so the assistant can process many documents when needed.
- Hardcoded retrieval ceilings must be replaced by adaptive/configurable behavior that does not artificially stop large evidence reads.
- Citation rendering must use a hybrid model:
  - in-text markers when the model emits marker references
  - compact inline fallback references directly below the response when markers are absent
- Source references must be minimal, link to existing source destinations, and show source title on hover tooltip.
- Chat input actions must be unified into one icon-only button that represents send, loading, and cancel states.
- Chat route/components must be aligned to `DESIGN.md`, including opaque overlay/tooltip rules, token usage, restrained borders, and 4px grid spacing discipline.
- Visual symmetry issues in chat must be fixed and validated with screenshots of key states.
- Existing core chat behaviors must remain intact: streaming answers, cancellation, keyboard submit flow, and source link correctness.

## Non-Goals

- Replacing the current model provider/runtime stack.
- Redesigning non-chat pages or global shells outside chat-specific components.
- Introducing a full long-term memory system beyond current conversation history.
- Maintaining the old card-grid citation presentation as an equal primary mode.

## Design Considerations

- Retrieval intent should be decided as part of the existing planning phase so behavior stays explainable and observable.
- "Unlimited" evidence reads should be implemented through adaptive stages and synthesis compaction, not by dumping unbounded raw excerpts into one prompt.
- Hybrid citations preserve readability and provide graceful fallback when model output is not perfectly attributed.
- A single icon-button state machine keeps interaction predictable and removes layout shift between send/cancel states.
- `DESIGN.md` favors calm, minimal interfaces with limited border use and strict opaque overlays; chat changes should prioritize spacing, typography, and contrast over decorative chrome.

## Open Questions

- None at spec time. Implementation should raise new questions only if adaptive retrieval thresholds conflict with model/context limits in production.
