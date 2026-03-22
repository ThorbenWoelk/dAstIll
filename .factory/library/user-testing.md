# User Testing

Testing surface, required testing skills/tools, resource cost classification per surface.

**What belongs here:** Validation surface details, testing tool configuration, concurrency limits.

---

## Validation Surface

- **Primary surface:** Browser (SvelteKit web app on localhost:3543)
- **Tool:** agent-browser
- **Routes to validate:** `/` (workspace), `/highlights`, `/download-queue`, `/chat`, `/channels/[id]`
- **Start app:** `./start_app.sh --detach` from repo root
- **Stop app:** `./end_app.sh` from repo root
- **Backend health:** `curl -sf http://localhost:3544/api/health`
- **Frontend health:** `curl -sf http://localhost:3543`

- **Backend API surface:** local HTTP API on `http://localhost:3544`
- **Tools:** `curl` for response header/status checks, `cargo test` for backend behavior assertions
- **Milestone usage:** `backend-perf` assertions (`VAL-BACKEND-001` through `VAL-BACKEND-005`)

## Known Limitations

- `networkidle` wait strategy does NOT work due to persistent SSE connections. Use explicit element waits (`wait --text`, `wait --selector`) or `sleep` delays.
- Ollama cloud models may hit rate limits (429). Non-blocking for UI validation but affects chat/summary features.
- Docs VitePress may crash with double-port bug. Not needed for validation.
- Protected backend API routes require `x-dastill-proxy-auth: local-dev-backend-proxy-token` for local validation calls.
- Backend `db::videos` integration tests marked `#[ignore]` require live/reachable S3; otherwise they can fail with `S3("dispatch failure")` during setup.

## Testing Approach

- agent-browser navigates routes, captures screenshots, inspects DOM and network
- Network throttling and offline simulation: agent-browser MAY support `evaluate` to set network conditions via CDP, but prefer testing observable outcomes (e.g., "does cached data render?" rather than "does it work offline?")
- For service worker validation: load data, navigate away, navigate back, verify data renders from cache without visible loading states
- For optimistic UI: use default network speed and verify mutations update UI immediately (before response)

## Validation Concurrency

- **Machine:** 48 GB RAM, 14 CPU cores (Apple Silicon)
- **App stack:** ~250 MB (backend ~62 MB + frontend ~182 MB + bun ~5 MB)
- **Per agent-browser instance:** ~500 MB (chrome-headless-shell ~480 MB)
- **Max concurrent validators:** 5 (at 70% headroom of ~14 GB available: 250 MB app + 5 x 500 MB = 2.75 GB, well within budget)

### Backend API + Rust test validators

- **Current machine sample:** ~51.5 GB RAM total (`sysctl -n hw.memsize`), substantial free pages from `vm_stat`
- **Shared resources:** backend API port `3544`, Rust build cache (`backend/target`)
- **Isolation impact:** API curl checks are read-only against running backend; backend cargo tests are local process checks and can run in parallel with curl checks
- **Max concurrent validators for backend-perf:** 2
  - Validator 1: API cache-header flow (`curl` against running backend)
  - Validator 2: Rust backend test flow (`cargo test` focused on backend assertions)

## Flow Validator Guidance: backend-api

- Use only `http://localhost:3544` and do not start additional backend instances or alternate ports.
- Keep checks read-only unless an assertion explicitly requires mutation.
- Save raw header/status evidence under mission evidence dir for your assigned group.
- Do not modify application code; this validator only validates runtime behavior and writes a flow report.

## Flow Validator Guidance: backend-rust-tests

- Run tests only from `backend/` in this repo; do not run frontend validators.
- Avoid changing source files; only execute tests and collect outputs/evidence.
- Use assertion-focused test commands where possible, but ensure coverage for assigned backend assertions.
- Write report JSON to the assigned flow report path and store command output evidence in mission evidence dir.
