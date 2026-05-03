# AI Models

## Model Roles

dAstIll uses independently configured model roles through the configured Ollama endpoint. Use
`backend/.env.example` for the current key names.

| Role              | Used for                                                                        | Detail doc                               |
| ----------------- | ------------------------------------------------------------------------------- | ---------------------------------------- |
| Summarizer        | Primary summary generation and transcript cleaning                              | [Summarization](/features/summarization) |
| Fallback          | Optional local fallback when the primary summarizer is cloud-backed and limited | [Summarization](/features/summarization) |
| Chat              | RAG conversations over indexed library content                                  | [Chat](/features/chat)                   |
| Summary evaluator | Summary quality evaluation                                                      | [Summarization](/features/summarization) |
| Embedding         | Dense embeddings for semantic search                                            | [Search](/features/search)               |
| Reranker          | Optional cross-encoder reranking for hybrid search                              | [Search](/features/search)               |
| HyDE              | Optional short-query passage synthesis                                          | [Search](/features/search)               |
| TTS               | Summary audio synthesis                                                         | [TTS](/features/tts)                     |

There is one endpoint setting for all Ollama-backed roles.

## Search Model Hooks

Search owns embedding, HyDE, reranking, retrieval modes, and semantic defaults.

| Role      | Endpoint               | Used for                               |
| --------- | ---------------------- | -------------------------------------- |
| Embedding | Ollama `/api/embed`    | semantic chunk and query embeddings    |
| Reranker  | Ollama `/api/rerank`   | optional cross-encoder reranking       |
| HyDE      | Ollama `/api/generate` | optional short-query passage synthesis |

## Availability And Cooldowns

The app tracks separate cooldown domains.

| Cooldown               | Used for                                          |
| ---------------------- | ------------------------------------------------- |
| Cloud cooldown         | backs off after cloud model rate limits           |
| YouTube quota cooldown | suppresses repeated YouTube Data API quota errors |
| Transcript cooldown    | slows transcript retries after rate limits        |

Services check cooldown state before attempting work and skip work that is inside an active
cooldown window.
Cooldown values live in [Runtime Limits](/operations/runtime-limits#cooldowns).

## Degradation Model

dAstIll keeps unaffected features available when model or retrieval dependencies fail.

| Failure                               | Degradation                                        | Detail doc                               |
| ------------------------------------- | -------------------------------------------------- | ---------------------------------------- |
| Summarizer unavailable                | generation queue pauses; search and chat continue  | [Summarization](/features/summarization) |
| Embedding model unavailable           | FTS-only search; chunking and S3 writes continue   | [Search](/features/search)               |
| Evaluator unavailable or rate-limited | evaluation pauses; generation and search continue  | [Summarization](/features/summarization) |
| Reranker call fails                   | plain RRF ordering                                 | [Search](/features/search)               |
| HyDE generation fails                 | raw query embedding                                | [Search](/features/search)               |
| Cloud rate limit                      | local fallback when configured, then cooldown wait | [Summarization](/features/summarization) |
| Semantic embedding call fails         | FTS-only for that request                          | [Search](/features/search)               |
| Chat answer model rate-limited        | cited source-list fallback                         | [Chat](/features/chat)                   |
