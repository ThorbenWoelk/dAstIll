# AI Models

## Model Roles

dAstIll uses independently configured model roles through the configured Ollama endpoint. Use
`backend/.env.example` for the current key names.

| Role              | Used for                                                                        | Detail doc                                                          |
| ----------------- | ------------------------------------------------------------------------------- | ------------------------------------------------------------------- |
| Summarizer        | Primary summary generation and transcript cleaning                              | [Summarization and Evaluation](/pipelines/summarization-evaluation) |
| Fallback          | Optional local fallback when the primary summarizer is cloud-backed and limited | [Summarization and Evaluation](/pipelines/summarization-evaluation) |
| Chat              | RAG conversations over indexed library content                                  | [Chat RAG](/pipelines/chat-rag)                                     |
| Summary evaluator | Summary quality evaluation                                                      | [Summarization and Evaluation](/pipelines/summarization-evaluation) |
| Embedding         | Dense embeddings for semantic search                                            | [Search Indexing](/pipelines/search-indexing)                       |
| Reranker          | Optional cross-encoder reranking for hybrid search                              | [Search Indexing](/pipelines/search-indexing)                       |
| HyDE              | Optional short-query passage synthesis                                          | [Search Indexing](/pipelines/search-indexing)                       |
| TTS               | Summary audio synthesis                                                         | [Text to Speech](/pipelines/text-to-speech)                         |

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

## Degradation Model

dAstIll keeps unaffected features available when model or retrieval dependencies fail.

| Failure                               | Degradation                                        | Detail doc                                                          |
| ------------------------------------- | -------------------------------------------------- | ------------------------------------------------------------------- |
| Summarizer unavailable                | generation queue pauses; search and chat continue  | [Summarization and Evaluation](/pipelines/summarization-evaluation) |
| Embedding model unavailable           | FTS-only search; chunking and S3 writes continue   | [Search Indexing](/pipelines/search-indexing)                       |
| Evaluator unavailable or rate-limited | evaluation pauses; generation and search continue  | [Summarization and Evaluation](/pipelines/summarization-evaluation) |
| Reranker call fails                   | plain RRF ordering                                 | [Search Indexing](/pipelines/search-indexing)                       |
| HyDE generation fails                 | raw query embedding                                | [Search Indexing](/pipelines/search-indexing)                       |
| Cloud rate limit                      | local fallback when configured, then cooldown wait | [Summarization and Evaluation](/pipelines/summarization-evaluation) |
| Semantic embedding call fails         | FTS-only for that request                          | [Search Indexing](/pipelines/search-indexing)                       |
| Chat answer model rate-limited        | cited source-list fallback                         | [Chat RAG](/pipelines/chat-rag)                                     |
