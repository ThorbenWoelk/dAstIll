# Chat

The chat service answers questions grounded in the indexed library. It uses:

- the configured chat model, falling back to the summarizer model
- structured planning with backend-owned JSON schemas
- workspace search retrieval
- source attribution stored with assistant messages
- server-sent events for streaming replies

Model selection happens when a conversation is created. If the selected model is unavailable at
message-send time, the service records a failed assistant message without corrupting the
conversation.

The model list shown to users comes from Ollama `/api/tags`, plus predefined cloud model entries.
Users can switch models per conversation.

## Intent And Budgets

Before retrieval, chat classifies the user message.

| Intent           | Use case                                  |
| ---------------- | ----------------------------------------- |
| `fact`           | Specific lookup from one or a few sources |
| `synthesis`      | Cross-video synthesis of a topic          |
| `recommendation` | Best/worth-watching style requests        |
| `comparison`     | Comparison between two or more subjects   |
| `pattern`        | Pattern detection across a large corpus   |

Deep research mode raises the source budget to the system maximum and enables extra retrieval
passes.
Source budgets live in [Runtime Limits](/operations/runtime-limits#chat-limits).

## Retrieval And Context

Chat retrieval can run multiple passes and generate expansion queries. Retrieval pass and query
limits live in [Runtime Limits](/operations/runtime-limits#chat-limits).

Query kinds:

- primary queries that directly address the user message
- expansion queries that cover adjacent concepts and alternate phrasing

Each query uses the same keyword and semantic retrieval machinery documented in
[Search](/features/search#query-path). Channel-scoped conversations pass a `channel_id` filter
through the retrieval path.

Context assembly:

- candidates are scored from keyword and semantic retrieval signals
- candidates are sorted within each video
- synthesis keeps a bounded number of chunks per video
- synthesis keeps a bounded number of videos
- source excerpts are capped before synthesis
- recent conversation history is bounded before it enters the prompt

Context and history limits live in [Runtime Limits](/operations/runtime-limits#chat-limits).

## Streaming And Attribution

Chat responses stream through SSE.

Conversation message states:

- `pending`
- `streaming`
- `complete`
- `failed`

The active chats tracker prevents concurrent streams for the same conversation. The client can
cancel a stream or reconnect to an in-progress stream.

Every assistant message stores source attribution:

- `video_id` and `video_title`
- `source_kind`
- `section_title`
- `snippet`
- `start_sec` for timed transcript chunks

Attribution is displayed in the chat UI so users can trace claims back to source content.

## Trace And Budgets

When tracing is configured, the backend sends structured chat traces to Logfire:

- query plan classification and generated queries
- per-pass retrieval timings and candidate counts
- context assembly and selected sources
- streaming lifecycle events

Raw prompts and full response bodies are excluded by default.

Completed assistant messages store a redacted `ChatTurnTrace` with:

- plan labels
- tool names
- retrieval counts
- selected-source counts
- per-turn budget snapshot

Each chat turn has budgets for model calls, tool calls, and retrieval passes. When a budget is
exhausted, the SSE stream emits `budget_exhausted` with a redacted snapshot. The service then falls
back to the best available evidence or records a rejected assistant message. Budget values live in
[Runtime Limits](/operations/runtime-limits#chat-limits).

When retrieval succeeds and the answer model is rate-limited, chat returns a cited source list
instead of synthesizing beyond retrieved text.
