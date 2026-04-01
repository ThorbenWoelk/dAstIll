---
title: OWASP ASI Status
---

# OWASP ASI Status

This page tracks dAstIll's current alignment against the **OWASP Top 10 for Agentic Applications (ASI 2026)**.

Important note: this is an **alignment tracker**, not a formal compliance attestation. OWASP presents the Top 10 for Agentic Applications as a peer-reviewed framework and practical guidance, and DeepTeam summarizes it as a risk model for agentic systems. This repo therefore uses statuses like `addressed`, `partially addressed`, and `open` instead of claiming formal certification.

Reference material:

- [OWASP Top 10 for Agentic Applications for 2026](https://genai.owasp.org/resource/owasp-top-10-for-agentic-applications-for-2026/)
- [DeepTeam framework guide for OWASP ASI 2026](https://www.trydeepteam.com/docs/frameworks-owasp-top-10-for-agentic-applications)

## Done In This Hardening Track

- Restricted persistent chat to authenticated users. Signed-out chat stays on the ephemeral path.
- Propagated `AccessContext` into chat retrieval and chat-internal tools.
- Blocked regular-user access to global `db_inspect`.
- Added explicit prompt guidance that retrieved excerpts, summaries, transcripts, highlights, and tool outputs are untrusted data.
- Added shared prompt-size and conversation-size limits, plus bounded stored-history growth for persistent chat.
- Added repo hygiene checks for forbidden tracked credential-adjacent artifacts.

## Status Matrix

| ASI risk | Current status | Controls now in place | What remains open |
| --- | --- | --- | --- |
| ASI01 - Agent Goal Hijack | Partially addressed | Grounded answering, explicit untrusted-data prompt rules, signed-out persistent chat removed, bounded conversation context | Add adversarial regression tests for indirect instruction and cross-context prompt injection; consider stronger content quarantine or scoring for retrieved hostile text |
| ASI02 - Tool Misuse & Exploitation | Partially addressed | Read-only tool set, schema-bounded inputs, bounded tool loops, scoped retrieval tools, operator-only `db_inspect` | Add distributed rate and budget enforcement, tool-call audit trails, and stronger anomaly detection for repeated tool abuse |
| ASI03 - Agent Identity & Privilege Abuse | Partially addressed | Proxy-authenticated backend requests, `AccessContext` on routes and chat tooling, authenticated-only persistent chat | Improve auditability of access decisions and keep expanding negative tests around authorization boundaries |
| ASI04 - Agentic Supply Chain Compromise | Partially addressed | No dynamic tool registry, curated internal tool surface, repo hygiene checks, version-pinning rules in repo guidance | Perform an explicit trust-boundary review for model providers, deployment actions, dependencies, and tool schemas; automate more of that review in CI |
| ASI05 - Unexpected Code Execution | Mostly addressed for current architecture | No shell tool, no eval path, no arbitrary code execution tool, no raw SQL tool in chat | Keep regression checks in place; if code-execution features are added later, require sandboxing and a new security spec first |
| ASI06 - Memory & Context Poisoning | Partially addressed | Untrusted-data prompt rules, bounded prompts, bounded conversation payloads, bounded stored history, no shared anonymous persistent memory | Add automated poisoning tests and consider stronger integrity checks or reset flows around persisted chat state |
| ASI07 - Insecure Inter-Agent Communication | Low current exposure | The app does not currently expose a distributed multi-agent mesh; planner and tool loop stay inside one backend boundary | If agent-to-agent or remote planner/executor flows are introduced, add signed message integrity, component authentication, and explicit trust boundaries |
| ASI08 - Cascading Agent Failures | Partially addressed | Tool step budgets, retrieval pass/query budgets, anonymous quota, request limits on some routes, bounded history | Replace process-local rate limiting with shared enforcement and add stronger circuit breakers and abuse telemetry |
| ASI09 - Human-Agent Trust Exploitation | Partially addressed | Citation metadata, tool-call metadata, instructions to admit when evidence is missing | Improve user-facing signaling around evidence freshness, tool use, and cases where the system is answering from prior conversation only |
| ASI10 - Rogue Agents | Low current exposure | No autonomous background agent with write access, bounded tool loop, read-only tool posture | Keep this architecture constraint explicit; if autonomy expands, require approvals, kill switches, isolation, and audit logs |

## Highest-Priority Open Work

1. Move rate limiting and abuse budgets to a shared backend store so controls hold across Cloud Run instances.
2. Add structured audit logging for tool calls, tool denials, retrieval scoping, and operator-only decisions.
3. Add automated OWASP ASI regression cases to CI for goal hijack, indirect instruction, cross-context retrieval, and tool-budget abuse.
4. Review and document external supply-chain trust boundaries for models, dependencies, GitHub Actions, and deployment config.

## Related Docs

- [Security Overview](/security/)
- [Deployment and Operations](/operations/deployment)
- [Local development](/local-development)
