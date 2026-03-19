# AI Models

## Model Roles

dAstIll uses different models for different jobs.

| Variable                  | Role                                                                                 |
| ------------------------- | ------------------------------------------------------------------------------------ |
| `OLLAMA_MODEL`            | Primary summarizer and transcript-cleaning model                                     |
| `OLLAMA_FALLBACK_MODEL`   | Optional local fallback when the summarizer primary is cloud-backed and rate-limited |
| `OLLAMA_CHAT_MODEL`       | Chat model for RAG conversations (falls back to `OLLAMA_MODEL` if not set)           |
| `SUMMARY_EVALUATOR_MODEL` | Summary quality evaluator                                                            |
| `OLLAMA_EMBEDDING_MODEL`  | Search embedding model                                                               |

## Summarizer Behavior

The summarizer service is responsible for:

- generating summaries
- cleaning transcript formatting while preserving wording
- reporting AI availability status to the frontend

If the primary summarizer model is cloud-backed and rate-limited, the app can fall back to a local model when one is configured.

## Evaluator Policy

The summary evaluator is stricter:

- it must be a cloud model
- it must represent a model larger than 40B parameters
- it must not be the same configured model as `OLLAMA_MODEL`
- its cooldown policy is `offline`, not local fallback

This means evaluation may pause instead of consuming local fallback capacity.

If `OLLAMA_MODEL` and `SUMMARY_EVALUATOR_MODEL` are identical, backend startup fails before the app serves requests.

## Search Embedding Behavior

The search service is separate from summarization and evaluation.

It owns:

- embedding model name
- embedding dimension count
- semantic-enabled flag
- optional local-model semaphore

If semantic search is disabled, the embedding model can still be configured but will not be used.

### Default Embedding Model

The default embedding model is **embeddinggemma:latest** (Gemma's embedding model via Ollama). This model:

- is optimized for semantic similarity tasks
- produces dense vector representations suitable for ANN retrieval
- runs locally through the configured Ollama endpoint

Other embedding models can be configured via `OLLAMA_EMBEDDING_MODEL` as long as they expose the `/api/embed` endpoint and produce float32 vectors.

## Chat Behavior

The chat service powers RAG (Retrieval-Augmented Generation) conversations with video content:

- **Model selection**: Uses `OLLAMA_CHAT_MODEL` if configured, otherwise falls back to `OLLAMA_MODEL`
- **Context retrieval**: Queries the semantic search index to find relevant transcript/summary chunks
- **Streaming responses**: Returns AI responses via server-sent events for real-time display
- **Source attribution**: Each response includes the source chunks used for grounding
- **Observability**: When `LOGFIRE_TOKEN` is configured, backend `tracing` events for Ollama calls and chat pipeline milestones are streamed to Logfire

Chat is optional and does not block other features. If the chat model is unavailable, chat requests fail gracefully while summarization and search continue.

Current Logfire coverage is operational rather than exhaustive: it includes prompt lifecycle metadata, retrieval timings, fallback/rate-limit events, and chat stage transitions, but not full raw prompt / response payloads by default.

## Availability and Cooldowns

The app tracks three cooldown domains:

| Cooldown               | Purpose                                    |
| ---------------------- | ------------------------------------------ |
| Cloud cooldown         | backs off after cloud model rate limits    |
| YouTube quota cooldown | suppresses repeated quota failures         |
| Transcript cooldown    | slows transcript retries after rate limits |

## Local vs Release Defaults

### Local debug runs

Defaults:

- semantic search on
- local embeddings generated when an embedding model is configured and available

### Release / production runs

Defaults:

- semantic search off
- plain FTS search unless explicitly overridden

`SEARCH_SEMANTIC_ENABLED` overrides either direction.

## Degradation Model

The application prefers degraded functionality over total failure.

Examples:

- summaries can fail while the rest of the app stays usable
- evaluator outages do not block search
- search can fall back to FTS-only mode
- cloud summarizer rate limits can degrade to local fallback when configured
