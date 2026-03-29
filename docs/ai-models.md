# AI Models

## Model Roles

dAstIll uses different models for different jobs. Each role is independently configured.

| Variable                  | Role                                                                                 |
| ------------------------- | ------------------------------------------------------------------------------------ |
| `OLLAMA_MODEL`            | Primary summarizer and transcript-cleaning model                                     |
| `OLLAMA_FALLBACK_MODEL`   | Optional local fallback when the summarizer primary is cloud-backed and rate-limited |
| `OLLAMA_CHAT_MODEL`       | Chat model for RAG conversations (falls back to `OLLAMA_MODEL` if not set)           |
| `SUMMARY_EVALUATOR_MODEL` | Summary quality evaluator (LLM-as-judge)                                             |
| `OLLAMA_EMBEDDING_MODEL`  | Search embedding model for semantic search                                           |
| `SEARCH_RERANK_MODEL`     | Optional cross-encoder reranker for hybrid search (`/api/rerank`)                    |
| `SEARCH_HYDE_MODEL`       | Optional generative model for HyDE passage synthesis (`/api/generate`)               |

All models are accessed through the configured Ollama endpoint (`OLLAMA_URL`). The same
endpoint is shared across roles; there is no per-role endpoint override.

---

## Summarizer

### Responsibilities

The summarizer service handles two tasks:

- **Summary generation**: produces a consistent, structured markdown summary from a
  cleaned transcript
- **Transcript cleaning**: normalizes formatting (whitespace, linebreaks, repetitions)
  while preserving the speaker's wording

Both are driven by `OLLAMA_MODEL` calling Ollama's `/api/generate` endpoint.

### Fallback

If the primary summarizer is a cloud-backed model (identified by URL pattern) and
encounters a rate limit, the service falls back to `OLLAMA_FALLBACK_MODEL` if
configured. Local fallback runs without waiting - it does not share a rate-limit cooldown
with the cloud path.

If no fallback is configured, rate-limited work waits until the cloud cooldown expires.

### AI Availability Status

The summarizer service reports AI availability to the frontend:

- whether the primary model is reachable
- whether the fallback is active
- current cooldown state

This status is surfaced in the workspace header indicator so users can see if AI
processing is stalled.

---

## Summary Evaluator

### Policy

The evaluator is stricter than the summarizer by design:

- it must be a **cloud model** (local models are not accepted)
- it must represent a model larger than **40B parameters** (enforced by name pattern check)
- it must not be the same model string as `OLLAMA_MODEL`
- its cooldown policy is `offline` rather than local fallback - evaluation pauses instead
  of consuming local capacity when the cloud is unavailable

Backend startup **fails** if `OLLAMA_MODEL` and `SUMMARY_EVALUATOR_MODEL` are identical.
This guard exists to keep generation and judgment independent.

### Judgment Criteria

The evaluator compares the generated summary against the canonical transcript and scores
on a 0-100 scale, writing a `quality_score` and `quality_note` to the summary record.
Summaries below the threshold can be automatically requeued for regeneration.

### Regeneration Cap

Each video has a `retry_count` field that caps how many times a low-quality summary is
regenerated. Once the cap is reached, the video is not requeued further.

---

## Search Embedding

### Configuration

| Variable                  | Purpose                                                            |
| ------------------------- | ------------------------------------------------------------------ |
| `OLLAMA_EMBEDDING_MODEL`  | Model name for dense embeddings (default: `embeddinggemma:latest`) |
| `SEARCH_SEMANTIC_ENABLED` | Override switch; local debug defaults on, release defaults off     |

The embedding model is accessed via Ollama's `/api/embed` endpoint. The service:

- batches embedding requests (up to 8 texts per request)
- validates that returned embedding dimensions match the configured model
- checks model availability at startup via Ollama `/api/tags`

If the model is not pulled or Ollama is unreachable at startup, semantic search silently
disables itself. FTS continues to work.

### Default Model

**embeddinggemma:latest** - Gemma's embedding model via Ollama:

- optimized for semantic similarity tasks
- produces 512-dimensional float32 vectors
- runs entirely locally through the configured Ollama endpoint

Any Ollama-compatible embedding model can be substituted via `OLLAMA_EMBEDDING_MODEL` as
long as it exposes `/api/embed` and returns float32 vectors.

### Local Concurrency

The embedding service shares a semaphore with the summarizer and evaluator services to
bound concurrent Ollama calls and avoid saturating local GPU/CPU resources.

---

## Neural Reranker (`SEARCH_RERANK_MODEL`)

The reranker applies cross-encoder scoring after Reciprocal Rank Fusion (RRF) to
improve precision in hybrid search results.

### When It Activates

- `SEARCH_RERANK_MODEL` is configured
- The search request is in `hybrid` execution mode
- Both FTS and semantic candidate lists are non-empty

FTS-only queries and semantic-only queries bypass the reranker entirely.

### How It Works

1. The FTS and semantic candidate lists are merged via RRF into a single flat ordered
   list (`collect_rrf_candidates`)
2. Up to 50 candidates from this list (by RRF rank) are sent to Ollama's `/api/rerank`
   endpoint (30 s timeout) along with the original query
3. Ollama returns a `relevance_score` for each candidate; results are sorted descending
4. The reranked list flows into the standard video grouping step

Falls back to the plain RRF ordering if the call fails or returns an empty result set.
The search log records `rerank_configured` and `rerank_elapsed_ms` per request.

### Model Selection

Any Ollama-compatible cross-encoder model that supports `/api/rerank` can be used. A
solid open-source option is `bge-reranker-v2-m3`. Cross-encoders are more accurate than
bi-encoders for this task because they attend jointly to the query and the document
rather than comparing independent embeddings.

---

## HyDE (`SEARCH_HYDE_MODEL`)

HyDE (Hypothetical Document Embeddings) improves semantic recall for short, ambiguous
queries by generating a plausible answer passage before computing the query embedding.

### The Problem It Solves

Short queries like "memory safety" or "async patterns" produce query vectors that may
not align well with document vectors even when relevant content exists. The query is too
sparse to land near documents in embedding space.

HyDE shifts the embedding target from the sparse query to a dense hypothetical document
that would plausibly answer the query, moving the query vector closer to where real
content vectors cluster.

### When It Activates

All of the following must be true:

- `SEARCH_HYDE_MODEL` is configured
- Semantic search is enabled for the request
- The query contains **4 or fewer meaningful tokens** after stopword removal

Queries with more terms are specific enough to embed well without synthesis.

### How It Works

The backend posts to Ollama's `/api/generate` (30 s timeout) with:

```
Write a concise 2-3 sentence passage that directly answers: "<query>".
Be specific. Output only the passage, nothing else.
```

The returned passage is embedded in place of the raw query. The original raw query is
still used for:

- BM25 FTS retrieval
- FTS pre-ranking
- keyword snippet extraction in results

Falls back to embedding the raw query on any failure (timeout, empty passage, network
error). The search log records `hyde_triggered` and `hyde_elapsed_ms` per request.

### Model Selection

Any instruction-following generative model available in Ollama works. Smaller models
(7B-8B) are sufficient because the task is constrained 2-3 sentence passage generation,
not open-ended reasoning. Fast generation matters more than reasoning depth here; pick
a model that can respond within the 30 s timeout under load.

---

## Chat and RAG

The chat service powers RAG (Retrieval-Augmented Generation) conversations grounded in
the indexed video corpus. It uses its own multi-stage pipeline that goes beyond simple
search - it classifies intent, generates multiple queries, and uses a multi-pass
retrieval strategy to gather the right context.

### Model Configuration

| Variable            | Chat behavior                                             |
| ------------------- | --------------------------------------------------------- |
| `OLLAMA_CHAT_MODEL` | Primary chat model. Falls back to `OLLAMA_MODEL` if unset |

Chat model selection is done at conversation creation time. If the selected model is no
longer available at message-send time, the service fails gracefully without corrupting
the conversation.

The model list exposed to the user comes from Ollama's `/api/tags` endpoint, filtered by
predefined cloud model entries. Users can switch models per-conversation.

### Intent Classification

Before retrieval, the chat service classifies each user message into an intent category:

| Intent           | Description                                           |
| ---------------- | ----------------------------------------------------- |
| `fact`           | Specific factual lookup from one or few sources       |
| `synthesis`      | Cross-video synthesis of a topic across many sources  |
| `pattern`        | Pattern detection across a large corpus               |
| `comparison`     | Comparative analysis between two or more subjects     |
| `recommendation` | Recommendation request (best X, worth watching, etc.) |

Intent drives the source budget (how many chunks the context window targets) and the
query fan-out strategy.

### Source Budgets by Intent

| Intent           | Source budget |
| ---------------- | ------------- |
| `fact`           | 6             |
| `synthesis`      | 12            |
| `recommendation` | 14            |
| `comparison`     | 20            |
| `pattern`        | 24            |

Deep research mode (a per-request flag set by the client) raises the budget to the system maximum and enables additional query passes beyond the standard limit.

### Multi-Pass Retrieval

Retrieval runs for up to 3 passes. Each pass generates up to 3 queries (5 total across
all passes). The planner produces:

- **Primary queries**: directly address the user message
- **Expansion queries**: cover adjacent concepts, related terminology, or alternative
  phrasings to widen recall

For each query, the chat service runs:

1. **FTS retrieval** against the in-memory Tantivy index (same BM25 path as workspace
   search, with keyword snippet extraction)
2. **Semantic retrieval** via S3 Vectors ANN (if semantic search is enabled and the
   model is available)

Both retrieval legs support an optional `channel_id` filter for channel-scoped
conversations.

### Context Assembly and Scoring

Retrieved candidates from all passes are scored by a composite function:

```
combined_score = keyword_score + semantic_score
```

- `keyword_score`: non-zero when the chunk appeared in an FTS result list
- `semantic_score`: non-zero when the chunk appeared in a semantic result list
- Chunks appearing in both lists accumulate both scores

The top candidates are then organized per video using `rank_chat_sources`:

- Candidates are sorted by combined score within each video
- A synthesis limit caps chunks per video (`CHAT_SYNTHESIS_SOURCES_PER_VIDEO = 3`)
- The video list is capped by a synthesis video limit (`CHAT_SYNTHESIS_VIDEO_LIMIT = 6`)
  computed from the intent budget

Final context passed to the model is truncated to `CHAT_SYNTHESIS_CONTEXT_MAX_CHARS`
(1200 characters per source) to stay within the model's context window.

### Streaming Responses

Chat responses are streamed to the frontend via server-sent events:

- The conversation state transitions from `pending` → `streaming` → `complete` (or
  `failed`)
- Each streaming chunk is forwarded as it arrives from Ollama
- The client can reconnect to an in-progress stream after a transient disconnect

Concurrent streams for the same conversation are prevented by the active chats tracker.
Cancellation is supported mid-stream.

### History Context

The last 12 messages of the conversation (`CHAT_HISTORY_LIMIT`) are included in the
prompt context for each new message. Earlier messages are excluded to keep the prompt
within the model's context window.

### Source Attribution

Every assistant message includes the source chunks used for grounding:

- `video_id` and `video_title`
- `source_kind` (transcript or summary)
- `section_title` (for summary chunks)
- `snippet` (the excerpt shown in the UI)
- `start_sec` (for timed transcript chunks - enables timestamp navigation)

Attribution is stored with the message and displayed in the chat UI so users can trace
claims back to the original content.

### Observability

When `LOGFIRE_TOKEN` is configured, the backend sends structured traces to Logfire
covering:

- query plan classification and generated queries
- per-pass retrieval timings and candidate counts
- context assembly and source selection
- streaming lifecycle events (start, complete, error)

Raw prompt payloads and full response bodies are not logged by default.

---

## Availability and Cooldowns

The app tracks three cooldown domains:

| Cooldown               | Purpose                                           |
| ---------------------- | ------------------------------------------------- |
| Cloud cooldown         | backs off after cloud model rate limits           |
| YouTube quota cooldown | suppresses repeated YouTube Data API quota errors |
| Transcript cooldown    | slows transcript retries after rate limits        |

Each cooldown tracks its own expiry. Services check the cooldown state before attempting
work and skip to the next item rather than blocking.

---

## Local vs Release Defaults

### Local debug runs

- semantic search **on** by default
- embeddings generated when the embedding model is configured and the Ollama endpoint is
  reachable
- summary evaluation runs if `SUMMARY_EVALUATOR_MODEL` is set and meets policy

### Release / production runs

- semantic search **off** by default (explicit `SEARCH_SEMANTIC_ENABLED=true` required)
- FTS-only search unless semantic is explicitly enabled
- same model guard and evaluator policy as local

`SEARCH_SEMANTIC_ENABLED` overrides either direction.

---

## Text-to-Speech (TTS)

dAstIll integrates with Amazon Polly to synthesize audio from summaries.

### Configuration

| Variable                | Purpose                                    |
| ----------------------- | ------------------------------------------ |
| `POLLY_TTS_ENABLED`     | Enable TTS (default: `false`)             |
| `POLLY_TTS_VOICE_ID`    | Polly voice ID (e.g., `Joanna`, `Matthew`) |
| `POLLY_TTS_ENGINE`      | Engine type: `standard` or `neural`        |
| `POLLY_TTS_OUTPUT_FORMAT`| Output format (e.g., `wav`, `pcm`)       |
| `POLLY_TTS_SAMPLE_RATE` | Sample rate in Hz (e.g., `8000`, `16000`)  |

### Processing Pipeline

1. **Markdown sanitization**: Summary text is stripped of HTML tags, markdown links are converted to plain text, and decorative characters are removed
2. **SSML injection**: Pause markers (`<break time="..."/>`) are inserted after headings and list items for natural pacing
3. **Chunking**: Long texts are split into chunks under 2500 characters, preserving SSML tag boundaries
4. **Synthesis**: Each chunk is sent to Polly via SSML; returned PCM audio is concatenated
5. **WAV wrapping**: Raw PCM is packaged into a WAV container for browser compatibility

### Statistics Tracking

Completed synthesis samples are recorded in Firestore (`dastill_tts_stats`) to estimate future synthesis duration based on word count and historical throughput.

| Field                | Description                            |
| -------------------- | -------------------------------------- |
| `sample_count`       | Number of completed TTS generations    |
| `total_words`        | Cumulative words processed             |
| `total_duration_secs`| Cumulative synthesis duration in seconds |

---

## Degradation Model

The application prefers degraded functionality over total failure. Failures in one domain
do not cascade to others.

| Failure                               | Degradation                                            |
| ------------------------------------- | ------------------------------------------------------ |
| Summarizer unavailable                | Search and chat continue; generation queue pauses      |
| Embedding model unavailable           | FTS-only search; chunking and S3 writes still happen   |
| Evaluator unavailable or rate-limited | Evaluation pauses; generation and search unaffected    |
| Reranker call fails                   | Falls back to plain RRF ordering; results still return |
| HyDE generation fails                 | Raw query is embedded as fallback; search continues    |
| Cloud rate limit                      | Degrades to local fallback if configured, else waits   |
| Semantic embedding call fails         | Hybrid request falls back to FTS-only for that request |
