# Runtime Limits

This page owns operational limits, quotas, budgets, cooldowns, and timeout values.

Feature docs describe behavior. This page records the numeric values operators may need when
debugging load, cost, latency, quota exhaustion, or failed requests.

## Deployment Capacity

| Surface     | Limit                                  | Value                 | Source                         |
| ----------- | -------------------------------------- | --------------------- | ------------------------------ |
| Backend API | Cloud Run max serving instances        | `1`                   | `terraform/cloud_run.tf`       |
| Backend API | Cloud Run memory                       | `1Gi`                 | `.github/workflows/deploy.yml` |
| Backend API | Cloud Run request timeout              | `3600s`               | `.github/workflows/deploy.yml` |
| Backend API | Terraform service template CPU         | `1000m`               | `terraform/cloud_run.tf`       |
| Podcast ASR | Cloud Run max instances                | `1`                   | `.github/workflows/deploy.yml` |
| Podcast ASR | Cloud Run min instances                | `0`                   | `terraform/cloud_run.tf`       |
| Podcast ASR | Cloud Run concurrency                  | `1`                   | `.github/workflows/deploy.yml` |
| Podcast ASR | Cloud Run CPU                          | `2`                   | `.github/workflows/deploy.yml` |
| Podcast ASR | Cloud Run memory                       | `2Gi`                 | `.github/workflows/deploy.yml` |
| Podcast ASR | Cloud Run request timeout              | `3600s`               | `.github/workflows/deploy.yml` |
| Podcast ASR | Startup probe failure threshold/period | `18` failures / `10s` | `.github/workflows/deploy.yml` |
| Backend API | Startup probe failure threshold/period | `15` failures / `10s` | `.github/workflows/deploy.yml` |

The backend is capped at one serving instance because it keeps a local libSQL cache/index and runs
in-process workers. Multi-replica scale-out would duplicate worker execution and create per-replica
cache divergence. Horizontal backend scaling is blocked until the serving path and worker path are
split or coordinated.

## Request And User Quotas

| Config key                        | Default       | Bounds       | Used for                                                          |
| --------------------------------- | ------------- | ------------ | ----------------------------------------------------------------- |
| `BASELINE_RATE_LIMIT_PER_MINUTE`  | `600`         | `1`-`1000`   | Rolling per-client limit for most API routes                      |
| `EXPENSIVE_RATE_LIMIT_PER_MINUTE` | `120`         | `1`-`1000`   | Rolling per-client limit for AI, chat, search writes, and streams |
| `ANONYMOUS_CHAT_QUOTA`            | `30`          | `1`-`1000`   | Anonymous chat messages per visitor                               |
| `USER_IDLE_TIMEOUT_SECS`          | `900`         | positive int | Worker idle gate after the last user request                      |
| `BACKEND_CORS_ALLOWED_ORIGINS`    | local origins | string list  | Accepted browser origins                                          |
| `OPERATOR_EMAIL_ALLOWLIST`        | empty         | string list  | Local accounts with backend `operator` role                       |

The rate limiter is process-local. It protects one backend instance. Shared rate limiting is tracked
as a security hardening gap.

## Cooldowns

| Limit                          | Value / config key                                     | Used for                                       |
| ------------------------------ | ------------------------------------------------------ | ---------------------------------------------- |
| Cloud model cooldown           | `OLLAMA_CLOUD_COOLDOWN_SECS`, default `432000` seconds | Backoff after cloud model rate-limit responses |
| YouTube quota cooldown         | `86400` seconds                                        | Backoff after YouTube Data API quota errors    |
| Transcript dependency cooldown | `3600` seconds                                         | Backoff after temporary transcript failures    |

## Model And Tool Concurrency

| Lane                                                 | Limit                      | Notes                                         |
| ---------------------------------------------------- | -------------------------- | --------------------------------------------- |
| Local summary/evaluator/chat/guardrail/planner calls | `1` concurrent request     | Cloud-tagged models skip this local semaphore |
| Search embedding/rerank/HyDE calls                   | `1` concurrent request     | Bounds local model-heavy search work          |
| S3 operations                                        | `12` concurrent operations | Shared S3 helper semaphore                    |
| Chat tool loop                                       | `4` steps                  | Regular chat turn                             |
| Chat tool loop, deep research                        | `6` steps                  | Deep research chat turn                       |
| Chat model calls per turn                            | `12` calls                 | Regular chat turn                             |
| Chat model calls per turn, deep research             | `24` calls                 | Deep research chat turn                       |

## Worker Cadence And Batch Limits

| Worker                    | Active cadence | Idle cadence                   | Batch / scan limit                   |
| ------------------------- | -------------- | ------------------------------ | ------------------------------------ |
| Queue worker              | `5s`           | backs off from `15s` to `60s`  | `4` videos                           |
| Refresh worker            | `30m`          | same as active                 | source-dependent                     |
| Gap scan worker           | `10m`          | same as active                 | `8` videos per channel               |
| Summary evaluation worker | `7s`           | backs off from `30s` to `120s` | `4` summaries                        |
| Search backfill           | search worker  | search worker                  | `64` sources                         |
| Search indexing           | `3s`           | backs off from `15s` to `120s` | `8` sources                          |
| Search reconcile          | `60s`          | search worker                  | `64` sources                         |
| Search prune              | search worker  | search worker                  | `256` sources                        |
| Vector-index retry        | `5m`           | search worker                  | only after vector index is not ready |

All worker loops skip scheduled work when there is no recent user activity.

## Content Processing Limits

| Limit                                 | Value        | Used for                                   |
| ------------------------------------- | ------------ | ------------------------------------------ |
| Queue distillation retries            | `3`          | Transcript and summary processing attempts |
| Summary auto-regeneration attempts    | `2`          | Low-quality summary regeneration           |
| Transcript formatting attempts        | `5`          | Model-assisted transcript cleaning         |
| Transcript formatting hard timeout    | `270s`       | Transcript cleaning                        |
| Description-like transcript threshold | `1000` words | YouTube transcript fallback detection      |

## Search Limits

| Limit                        | Value                                      | Used for                         |
| ---------------------------- | ------------------------------------------ | -------------------------------- |
| Transcript target chunk size | `300` words                                | Search projection chunking       |
| Transcript chunk overlap     | `40` words                                 | Search projection chunking       |
| Transcript chunks per source | `80` max                                   | Search projection chunking       |
| Summary target chunk size    | `300` words                                | Search projection chunking       |
| Summary chunks per source    | `80` max, including full-document chunk    | Search projection chunking       |
| Embedding dimensions         | `512`                                      | Common embeddinggemma setup      |
| Embedding batch size         | `8` chunks                                 | Search worker embedding requests |
| Embedding request timeout    | `90s`                                      | Ollama embedding calls           |
| HyDE timeout                 | `30s`                                      | Optional query expansion         |
| Rerank timeout               | `30s`                                      | Optional cross-encoder reranking |
| Rerank candidate cap         | `50` chunks                                | Reranker input                   |
| FTS query term cap           | `4` terms                                  | Keyword search parser            |
| Snippet window               | `420` characters                           | Search result excerpts           |
| Search result `limit`        | default `8`, clamped `1`-`25` video groups | `/api/search`                    |
| Hybrid FTS candidates        | `limit * 8`, clamped `10`-`100`            | Keyword leg                      |
| Keyword FTS candidates       | `limit * 2`, clamped `10`-`50`             | Keyword-only leg                 |
| ANN semantic candidates      | `limit * 8`, clamped `10`-`100`            | S3 Vectors leg                   |
| Exact semantic candidates    | `limit * 4`, clamped `10`-`50`             | Exact dot-product fallback       |

## Chat Limits

| Limit                           | Value                          | Used for                        |
| ------------------------------- | ------------------------------ | ------------------------------- |
| Fact source budget              | `6`                            | Specific lookup intent          |
| Synthesis source budget         | `12`                           | Cross-video synthesis intent    |
| Recommendation source budget    | `14`                           | Best/worth-watching intent      |
| Comparison source budget        | `20`                           | Comparison intent               |
| Pattern source budget           | `24`                           | Corpus-wide pattern intent      |
| Deep research source budget     | `48`                           | Maximum selected sources        |
| Recent activity source budget   | `12`                           | Recent activity intent          |
| Recent activity video limit     | `6`                            | Recent activity intent          |
| Retrieval passes                | `3`                            | Multi-pass chat retrieval       |
| Queries per pass                | `3` regular, `5` deep research | Query planning                  |
| Queries per turn                | `5`                            | Query planning                  |
| Deep research primary queries   | `6`                            | Query planning                  |
| Deep research expansion queries | `8`                            | Query planning                  |
| Retrieval candidate limit       | min `8`, max `48`              | Candidate selection             |
| Synthesis videos                | `6`                            | Final answer context            |
| Synthesis chunks per video      | `3`                            | Final answer context            |
| Source excerpt size             | `1200` characters              | Final answer context            |
| Chat context text size          | `1400` characters              | Candidate ranking context       |
| Conversation history in prompt  | `12` messages                  | Chat answer context             |
| Stored conversation messages    | `200` messages                 | Storage validation and trimming |
| Stored conversation payload     | `500000` characters            | Storage validation and trimming |
| User message size               | `12000` characters             | Request validation              |
| Stored sources per message      | `48` sources                   | Storage validation and trimming |
| Conversation title              | `80` characters                | Title generation and validation |
| Planner context                 | `6000` characters              | Planning prompt                 |
| Classifier timeout              | `15s`                          | Chat intent classification      |
| Mention scope timeout           | `5s`                           | Chat mention resolution         |
| Stream timeout                  | `30m`                          | Chat response streaming         |
| Stream retry attempts           | `3`                            | Chat response streaming         |

`CHAT_MULTI_PASS_ENABLED=false` disables multi-pass retrieval and keeps chat to the first retrieval
pass.

## Local ASR Limits

| Config key                  | Default     | Used for                                   |
| --------------------------- | ----------- | ------------------------------------------ |
| `LOCAL_ASR_MAX_AUDIO_BYTES` | `262144000` | Maximum podcast audio download size        |
| `LOCAL_ASR_TIMEOUT_SECS`    | `3600`      | Download and transcription request timeout |

## Billing Alert Budgets

| Config key                                      | Default                                        | Used for                                      |
| ----------------------------------------------- | ---------------------------------------------- | --------------------------------------------- |
| `billing_budget_app_monthly_amount_units`       | `50`                                           | Monthly all-service alert budget              |
| `billing_budget_cloud_run_monthly_amount_units` | `10`                                           | Monthly Cloud Run service-scoped alert budget |
| `billing_budget_thresholds`                     | `50%`, `80%`, `100%` actual, `100%` forecasted | Alert thresholds                              |

Billing budgets are alerts only. They do not cap, stop, or throttle spend.
