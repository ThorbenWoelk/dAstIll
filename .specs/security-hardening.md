# Spec: Security hardening -- perimeter, hygiene, and abuse prevention

All seven items below were verified against the current codebase. Every one is warranted.

---

## 1. Auth boundary (critical, do first)

**Problem.** `terraform/cloud_run.tf` lines 65-69 grant `roles/run.invoker` to `allUsers` on backend, frontend, and docs services. `backend/src/main.rs` has zero auth middleware -- no JWT, no API key, no session check. The entire 37-route API surface is reachable by anyone on the internet.

**Evidence.** Grep for `middleware`, `Bearer`, `jwt`, `cookie`, `session` across `backend/src/` returns zero hits for incoming-request auth. The only auth code is outbound (Ollama API key, YouTube API key, AWS STS).

**Fix.** This is a single-user/operator app. Fastest safe path:

1. Remove the three `allUsers` IAM bindings in `cloud_run.tf`.
2. Protect Cloud Run with IAP or IAM (require a Google identity to invoke).
3. Optionally add a lightweight API-key or session middleware in `main.rs` for defense-in-depth, but IAP alone closes the perimeter.

If multi-user support is planned, add real app-level auth (e.g. Firebase Auth or OAuth2 + JWT validation middleware on every route) plus per-route authorization.

**Files.** `terraform/cloud_run.tf`, `backend/src/main.rs`.

---

## 2. GET endpoints with side-effects (high)

**Problem.** Three GET handlers perform remote fetches, LLM invocations, DB writes, search-index syncs, and cache invalidation:

| GET route | Side-effects |
|---|---|
| `/api/videos/{id}/transcript` | Downloads transcript from YouTube, upserts to DB, syncs search, clears read cache (`content.rs:38`) |
| `/api/videos/{id}/summary` | Calls `ensure_transcript` (above), invokes LLM summarizer, upserts summary, auto-regenerates if quality below threshold (`content.rs:143`) |
| `/api/videos/{id}/info` | Fetches metadata from YouTube API, upserts to DB (`videos.rs:71`) |

**Fix.** Split each into two endpoints:

- **GET** returns only what is already in the DB (or 404/202 if not yet generated).
- **POST** triggers the expensive ensure/generate pipeline.

The frontend currently relies on GET-triggers-work semantics, so update the fetch logic in the SvelteKit frontend to POST first, then poll or read the GET.

**Files.** `backend/src/handlers/content.rs`, `backend/src/handlers/videos.rs`, frontend callers.

---

## 3. Wildcard CORS (medium)

**Problem.** `main.rs` lines 375-379: `CorsLayer::new().allow_origin(Any).allow_methods(Any).allow_headers(Any)`. Any browser origin can call any endpoint with any method.

**Fix.** Restrict `allow_origin` to the actual frontend origin(s) -- read from an env var or config. Restrict `allow_methods` to the methods actually used (GET, POST, PUT, DELETE, OPTIONS). Restrict `allow_headers` to Content-Type and any custom headers actually sent.

**Files.** `backend/src/main.rs`, `backend/src/config.rs` (add allowed-origins config).

---

## 4. User-derived content in telemetry (high)

**Problem.** Six tracing spans in `services/chat.rs` emit the first 120 characters of user prompts via `query.preview` / `prompt.preview` (lines 620, 759, 858, 978, 1239, 1480). Line 1505 logs the full AI-generated conversation title. The Logfire filter (`main.rs:27-29`) forwards all `dastill::services::chat` spans, so this data reaches the external telemetry backend.

**Fix.**

1. Remove `query.preview` and `prompt.preview` fields from all six spans. Replace with opaque metadata (prompt length, conversation ID, model name).
2. Remove `title = %generated_title` from the `tracing::info!` call at line 1505.
3. If Logfire has been enabled in production, review retention policy and purge any exported chat data.

**Files.** `backend/src/services/chat.rs`, `backend/src/main.rs`.

---

## 5. Rate limits and hard quotas (high, after auth)

**Problem.** Zero client-facing rate limiting exists. The only throttling is internal cooldowns for upstream API quotas (YouTube 24h cooldown, cloud LLM rate-limit detection). No `tower-governor`, `governor`, or any rate-limiting crate in `Cargo.toml`.

**Expensive endpoints at highest risk:**

| Route | Risk |
|---|---|
| `POST .../messages` (chat) | LLM streaming, unbounded cost |
| `GET .../stream` (chat SSE) | Long-lived connection, LLM cost |
| `POST /api/search/rebuild` | Full index rebuild, heavy I/O |
| `POST .../backfill` (channels, videos) | Bulk YouTube API calls |
| `POST .../summary/regenerate` | LLM invocation |
| `POST .../transcript/clean` | LLM invocation |

**Fix.** Add `tower-governor` (or equivalent) as middleware:

- Global baseline: e.g. 60 req/min per IP.
- Expensive-endpoint tier: e.g. 5 req/min for LLM and backfill routes.
- Operator-only endpoints: even stricter or gated by auth tier (see item 6).

**Files.** `backend/Cargo.toml`, `backend/src/main.rs`.

---

## 6. Operator-only endpoint separation (high, after auth)

**Problem.** Administrative/destructive endpoints are on the same footing as normal reads:

- `POST /api/search/rebuild`
- `POST /api/channels/{id}/refresh`
- `POST /api/channels/{id}/backfill`
- `POST /api/videos/info/backfill`
- `POST /api/videos/{id}/summary/regenerate`
- `POST /api/videos/{id}/transcript/clean`
- `DELETE /api/channels/{id}`

**Fix.** After auth is in place, split routes into two groups:

1. **User routes** -- reads, search, chat, highlight CRUD.
2. **Operator routes** -- rebuild, backfill, refresh, bulk regeneration, channel deletion. Require a stronger auth claim (e.g. admin role, separate API key, or IAM policy).

Implement as a nested router with an additional auth-check layer, or as a middleware guard on the operator route group.

**Files.** `backend/src/main.rs`.

---

## 7. Dependency audit and secret-retention sweep (medium, after perimeter)

**Problem.** No `cargo audit`, `cargo deny`, or `bun audit` runs in CI or locally. No `deny.toml` or `audit.toml` exists. The single GitHub Actions workflow (`deploy.yml`) does build + deploy only. Secret Manager bindings and runtime env injection have not been audited for leakage.

**Fix.**

1. Run `cargo audit` and `bun audit` (frontend + docs) now, fix any findings.
2. Add both to the CI pipeline (`deploy.yml` or a dedicated security workflow).
3. Review Secret Manager bindings: confirm secrets are injected only as env vars at runtime, not baked into images or logged.
4. Confirm no secrets or sensitive prompts landed in external logs (Logfire, GCloud Logging) -- ties into item 4.

**Files.** `.github/workflows/deploy.yml`, `backend/Cargo.toml` (add `cargo-deny` config), `frontend/package.json`, `docs/package.json`.

---

## Execution order

1. **Auth boundary** (item 1) -- closes the open perimeter.
2. **Telemetry PII** (item 4) -- stop the bleeding of user data.
3. **GET side-effects** (item 2) -- correct HTTP semantics, reduce attack surface.
4. **CORS** (item 3) -- tighten browser access.
5. **Operator separation** (item 6) -- requires auth to be in place.
6. **Rate limits** (item 5) -- requires auth for per-user tracking.
7. **Dependency sweep** (item 7) -- ongoing hygiene.

## Validation

After each item, run validators step-by-step (per project guidelines):

```bash
cd backend && cargo check
cd backend && cargo test
cd frontend && npm run check
cd frontend && bun test
```
