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

### ASI01 / ASI06 - Goal hijack and memory/context poisoning

- Chat answers are grounded in retrieved excerpts, tool outputs, and visible conversation history.
- Prompting now explicitly treats excerpts, summaries, transcripts, highlights, and tool outputs as **untrusted data**, not instructions.
- Anonymous persistent chat is not used. Signed-out chat stays on the ephemeral path, which narrows cross-user state exposure.
- Chat prompt size and conversation payload size are bounded, which reduces stale or hostile context accumulation in both client-supplied and persisted chat history.

### ASI02 / ASI05 - Tool misuse and unexpected code execution

- Chat tools are schema-bounded and read-only.
- There is no shell, eval, arbitrary code execution, or raw SQL tool in the chat loop.
- Tool execution is step-limited.

### ASI03 - Identity and privilege abuse

- The backend accepts two first-party auth modes: trusted proxy headers behind `x-dastill-proxy-auth`, or direct Firebase bearer-token validation from browser and Tauri clients.
- Backend route handlers always derive request identity into `AccessContext` before scoping channel, video, search, and chat access.
- Chat retrieval and chat-internal tools are now constrained to the caller's accessible library scope.
- Global `db_inspect` is treated as authenticated-only.

### ASI08 - Cascading failures

- Expensive routes have explicit rate-limit hooks.
- Anonymous chat has a persisted quota.
- Tool loops and retrieval passes have bounded step/query budgets.
- Chat turns now enforce per-message and per-conversation size ceilings, and persistent conversations drop the oldest stored messages when they hit storage bounds.

### ASI09 - Human-agent trust exploitation

- The product UI exposes tool-call and citation metadata for chat responses.
- The assistant is instructed to say when evidence is missing or incomplete instead of silently filling gaps.

### Repo and secret hygiene

- Production secrets belong in GCP Secret Manager and are provisioned through Terraform.
- The repo now fails validation when forbidden tracked artifacts such as service-account keys, WIF tokens, or Terraform plan/state files are present.

## Known Gaps

- Rate limiting is still process-local in memory, so it does not fully hold across Cloud Run scale-out.
- Red-team testing against the OWASP ASI scenarios is still manual and documentation-driven; it is not yet automated in CI.
- Tool-call audit logging can still be improved so security-relevant chat decisions are easier to inspect after the fact.
- Supply-chain review for models, deployment dependencies, and tool trust boundaries is still incomplete.

## Verification Checklist

When making security-relevant changes, verify at least the following:

1. `./scripts/check_forbidden_artifacts.sh`
2. `cd backend && cargo check && cargo test`
3. `cd backend && ./scripts/cargo_audit.sh`
4. `cd frontend && bun run format:check && bun run lint && bun run check && bun run test && bun run build`
5. Confirm signed-out chat uses the ephemeral path and cannot use persistent conversation routes.
6. Confirm signed-in chat cannot retrieve content, highlights, or recent-activity evidence outside the caller's library scope.
7. Confirm oversized chat prompts or oversized client-supplied conversation payloads are rejected with `400 Bad Request`.

## Related Docs

- [OWASP ASI Status](/security/owasp-asi-status)
- [Deployment and Operations](/operations/deployment)
- [Local development](/operations/local-development)
- [AI Models](/pipelines/ai-models)
