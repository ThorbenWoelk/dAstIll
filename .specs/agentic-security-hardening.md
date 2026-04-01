# Agentic Security Hardening

## Problem

The current app has several security controls in place, but the chat subsystem still violates core OWASP ASI expectations around scope isolation, tool authorization, and prompt/data trust boundaries. In particular, persistent anonymous chat is not safely partitioned, chat-internal tools can bypass normal per-user access restrictions, and the repo still tracks credential-adjacent artifacts without fail-fast validation.

## Goal

Raise security awareness and improve compliance incrementally by hardening the highest-risk agentic paths first, documenting the current posture clearly, and creating a durable backlog for the remaining gaps.

## Status Note

OWASP ASI is a risk framework and prioritization guide, not a formal certification program. This work therefore tracks **alignment** against the OWASP Top 10 for Agentic Applications instead of claiming a binary compliant or non-compliant state.

## Requirements

- Persistent chat must not allow multiple signed-out users to share the same stored conversation namespace.
- Chat retrieval and tool execution must honor the caller's `AccessContext` for library data, recent-activity views, highlights, and direct video lookups.
- Global database inspection must not be available to regular users through chat.
- Prompting and grounding layers must explicitly treat retrieved/tool content as untrusted data, not executable instructions.
- Chat prompts and stored conversation history must be size-bounded so one request or one conversation cannot grow agent context or stored payloads without limit.
- The repo must fail fast when forbidden tracked artifacts such as service-account keys, WIF tokens, or Terraform plan/state artifacts are present.
- Security documentation must live in a dedicated `docs/security/` section and map current controls and gaps to the OWASP Top 10 for Agentic Applications.
- The task file must capture both what this pass fixes now and the remaining follow-up items.

## Non-Goals

- Replacing the current frontend-to-backend proxy architecture.
- Building a full automated red-team harness in this pass.
- Solving distributed, cross-instance rate limiting in Cloud Run in the same change.
- Redesigning all chat UX flows beyond what is needed to close the current security gaps.

## Design Considerations

- The repo already has an anonymous ephemeral chat path, so the lowest-risk fix for shared anonymous persistent chat is to reserve persistent conversations for authenticated users and keep signed-out chat ephemeral.
- Existing REST handlers already rely on `AccessContext`; chat should use the same boundary instead of inventing a parallel access model.
- Tool surfaces should be least-privilege by default: regular users need grounded content retrieval, not global database inspection.
- OWASP ASI issues need both controls and visibility, so code changes should be paired with docs that explain what is covered now versus still pending.
- Repo hygiene needs prevention and detection: `.gitignore` additions alone are not enough while tracked artifacts can still exist.

## Delivered Controls To Date

- Persistent anonymous chat was removed. Signed-out users use the ephemeral flow only.
- Chat retrieval, recent activity, direct video lookup, and highlight access now inherit `AccessContext`.
- `db_inspect` is treated as an operator-only capability.
- Prompting and grounding explicitly treat retrieved excerpts and tool outputs as untrusted data, not instructions.
- Chat prompts and stored conversation history are size-bounded, and persistent history now trims oldest entries at storage limits.
- Forbidden tracked artifacts such as service-account keys, WIF tokens, and Terraform plan files are blocked by local and CI checks.
- Dedicated security docs now capture both implemented controls and open gaps.

## Remaining Alignment Work

- Replace process-local request and chat throttling with a shared limiter that holds across Cloud Run instances.
- Add security-relevant audit logging for tool calls, tool denials, retrieval scoping, and privilege decisions.
- Add automated OWASP ASI regression coverage for prompt injection, indirect instruction, cross-context retrieval, and tool-budget abuse.
- Review external model, dependency, and deployment supply-chain trust boundaries as an explicit ASI04 workstream.
- Preserve the current single-agent, no-code-execution architecture as an explicit constraint unless a later spec reopens that surface with sandboxing and approval controls.

## Open Questions

- Whether future signed-out persistent chat should be reintroduced with a per-client namespace, or remain authenticated-only long term.
- Whether chat tool-call audit logging should become part of the next hardening pass.
