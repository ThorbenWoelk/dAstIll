# Tasks: Agentic Security Hardening

## Current State
OWASP ASI review completed and a second hardening pass has landed. Persistent anonymous chat is disabled in favor of the existing ephemeral path, chat retrieval/tools now inherit `AccessContext`, operator-only database inspection is blocked for regular users, prompt grounding calls out untrusted data boundaries, repo hygiene fails on forbidden tracked artifacts, chat now enforces shared prompt/conversation size limits with bounded stored-history growth for persistent conversations, and the docs/specs now split implemented controls from remaining OWASP ASI alignment work.

## Steps
- [x] Create spec and task files for agentic security hardening.
- [x] Restrict persistent chat to authenticated users and keep anonymous chat on the ephemeral path only.
- [x] Propagate `AccessContext` into chat retrieval and tool execution so search, direct video lookup, recent activity, and highlights stay inside caller scope.
- [x] Remove or block non-operator access to global database inspection through chat.
- [x] Add prompt/grounding instructions that treat retrieved content and tool outputs as untrusted data, not instructions.
- [x] Add shared chat prompt and conversation-size limits so oversized requests are rejected and persistent history cannot grow without bound.
- [x] Add repo validation for forbidden tracked artifacts and align ignore rules with the intended secret posture.
- [x] Add dedicated docs under `docs/security/` with OWASP ASI mapping, current controls, known gaps, and verification guidance.
- [x] Record the remaining follow-up work for distributed rate limiting, deeper red-teaming, and any unresolved anonymous-chat design decision.

## Decisions Made During Implementation
- Anonymous users keep chat access through the existing ephemeral flow; persistent conversations are authenticated-only in this pass.
- Chat authorization reuses `AccessContext` instead of introducing a separate tool-specific policy model.
- Global database inspection is treated as an operator-only capability.
- The initial repo-hygiene guardrail is a tracked-file check enforced locally and in CI, with stronger history cleanup or rotation work left to follow-up.

## Remaining Backlog

- [ ] Move request and chat rate limiting to a shared backend store so quotas and abuse controls hold across Cloud Run instances.
- [ ] Add structured audit logging for tool calls, tool denials, retrieval scoping, and privilege decisions.
- [ ] Add automated OWASP ASI regression coverage for indirect instruction, cross-context retrieval, and tool-budget abuse.
- [ ] Document and review model, dependency, and deployment supply-chain trust boundaries as an ASI04 workstream.
- [ ] Keep single-agent and no-code-execution constraints explicit unless a later spec intentionally expands that surface.
