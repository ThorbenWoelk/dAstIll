# Security

This section documents the current security posture for dAstIll and the highest-priority follow-up work.

This repo tracks **OWASP ASI alignment**, not a formal certification. The OWASP Top 10 for Agentic Applications is used here as a risk framework and prioritization model for hardening work.

The current review uses the **OWASP Top 10 for Agentic Applications (ASI 2026)** risk model as summarized by DeepTeam's framework guide:

- Agent goal hijack
- Tool misuse and exploitation
- Agent identity and privilege abuse
- Agentic supply-chain compromise
- Unexpected code execution
- Memory and context poisoning
- Insecure inter-agent communication
- Cascading agent failures
- Human-agent trust exploitation
- Rogue agents

## Current Posture

The app is not a general-purpose autonomous agent. Its most agentic surface is the chat subsystem:

- it plans retrieval
- it can call a bounded set of read-only tools
- it composes answers from retrieved evidence

That keeps the attack surface materially smaller than a tool-executing agent with write access, but it still creates real ASI risks around goal hijack, scope bypass, and trust in retrieved content.

For the detailed done-versus-open breakdown, see [OWASP ASI Status](/security/owasp-asi-status).

## Controls In Place

dAstIll's current controls focus on scoped retrieval, read-only chat tools, bounded chat context, bounded tool loops, visible citation/tool metadata, and repo secret hygiene.

## Known Gaps

The highest-priority gaps are shared rate limiting, automated ASI regression coverage, stronger tool-call audit logs, and an explicit supply-chain trust-boundary review.

## Verification Checklist

When making security-relevant changes, verify at least the following:

1. `./scripts/check_forbidden_artifacts.sh`
2. `cd backend && cargo check && cargo test`
3. `cd backend && ./scripts/cargo_audit.sh`
   Current upstream-only waivers in that script must carry an explicit `review_after` date and should be re-reviewed instead of silently carried forward.
4. `cd frontend && bun run format:check && bun run lint && bun run check && bun run test && bun run build`
5. Confirm signed-out chat uses the ephemeral path and cannot use persistent conversation routes.
6. Confirm signed-in chat cannot retrieve content, highlights, or recent-activity evidence outside the caller's library scope.
7. Confirm oversized chat prompts or oversized client-supplied conversation payloads are rejected with `400 Bad Request`.

## Related Docs

- [OWASP ASI Status](/security/owasp-asi-status)
- [Deployment and Operations](/operations/deployment)
- [Local development](/operations/local-development)
- [AI Models](/pipelines/ai-models)
- [Chat RAG](/pipelines/chat-rag)
