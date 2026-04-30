# Agent Guide

Keep this file short, operational, and focused on how to work in the repo.
Deeper domain-specific guidance belongs in dedicated docs and should be linked from here.

## Source Of Truth

- Frontend design system, Svelte frontend cleanliness rules, UI architecture guidance, file-size thresholds, and frontend testing expectations live in [design.md](./design.md).
- User docs in [./docs/](./docs/)

## How To Work Here

- Read this file first, then open the linked domain doc you need.
- Do not duplicate large guidance blocks across multiple markdown files.
- When frontend rules change, update `design.md` and keep only the pointer here.
- Keep repo guidance legible for agents: short entry points here, detailed source-of-truth docs elsewhere.
- Never put required environment values into [`start_app.sh`](./start_app.sh); adjust the shared/local `.env` files and their setup flow instead.

## Documentation Split

- `AGENTS.md`: agent workflow entry point, document map, repo-level instructions.
- `DESIGN.md`: design system and frontend engineering standards.

## Docs Writing Style

- Write docs in plain, direct language.
- Prefer simple words over clever or abstract wording.
- Optimize for reader understanding, not voice, flourish, or sounding impressive.
- Avoid unnecessary jargon, layered metaphors, and inflated product language.
- If a sentence can be shorter or more concrete without losing meaning, rewrite it.

## Env and secrets strategy

**Follow these rules**

- Local app config lives in the shared machine-local env files:
  - `~/.config/dastill/backend.env`
  - `~/.config/dastill/frontend.env`
- Use [`./scripts/link_shared_env.sh`](./scripts/link_shared_env.sh) to link repo-local `.env` files to that shared location.
- Do not hardcode required env values in [`start_app.sh`](./start_app.sh) or commit secrets into repo files.
- Sensitive values for production belong in **GCP Secret Manager**. Terraform creates the secret containers and IAM bindings; secret payloads are written directly in Secret Manager and must not be stored in Terraform state.
- Infra CI auth model:
  - GCP: GitHub OIDC -> GCP Workload Identity Federation -> `dastill-github-sa`
  - AWS: GitHub OIDC -> AWS role `dastill-github-terraform`
  - Cloud Run runtime still uses separate GCP -> AWS federation via `dastill-gcp-backend`
- One bootstrap edge remains for AWS CI auth: the first creation of the AWS GitHub OIDC provider and `dastill-github-terraform` role must be applied from an already authenticated AWS context. After that, recurring Terraform runs can stay in CI.
- Secret bootstrap/rotation flow:
  1. Apply Terraform first so secret container and IAM exist.
  2. Add secret payload with `gcloud secrets versions add <secret-name> --project <project-id> --data-file=-`.
  3. Redeploy consumer so latest secret version is picked up.
- `infra.yml` auto-syncs only Firebase frontend build secrets (`dastill-firebase-web-api-key`, `dastill-firebase-auth-domain`) after Terraform apply. All other app secrets are still manual Secret Manager version adds.
- Current non-Firebase secrets expected in production: `dastill-youtube-api-key`, `dastill-openalex-api-key`, `dastill-ollama-api-key`, `dastill-logfire-token`, `dastill-backend-proxy-token`, `dastill-databricks-token`.
- Some internal code and script names still say `turso`. Treat those as historical names for the local `libSQL` store, not as active Turso Cloud usage.
- Secret deprecation stays IaC:
  1. Remove consumers.
  2. Remove workflow refs.
  3. Remove IAM refs in `terraform/iam.tf`.
  4. Remove secret resource in `terraform/secrets.tf`.
  5. Apply Terraform. No console-only cleanup as source of truth.
- Production runtime config has only two sources:
  1. **Secret Manager** for sensitive values.
  2. **Cloud Run service configuration** for non-secret values.
- CI credentials are only for CI authentication to GCP, not for storing app secrets.
- If a workflow passes a value through CI, treat that as transport, not as the source of truth.

Source-of-truth details:

- local dev env flow: [docs/operations/local-development.md](./docs/operations/local-development.md)
- production boundaries, Terraform, Secret Manager, and deploy behavior: [docs/operations/deployment.md](./docs/operations/deployment.md)

# Developer Guide

## Svelte 5 Reactive State Rules

- When returning `$state` or `$derived` from a function, use getters/setters to preserve the reactive boundary. The function scope becomes a closure that stays connected to the reactive proxies.

## Rust Module Structure

- `frag_*.rs` source fragments included via `include!()` are bad practice for hand-written repo code.
- Use true Rust submodules and explicit internal APIs when refactoring large files instead of adding new `frag_` files.

## Naming Conventions

- Prefer names that describe the domain concept or user-visible behavior.
- Avoid technical prefixes and suffixes like `_api`, `_util`, `_helper`, or `manager` unless they disambiguate a real domain concept.
- Name functions for what they do, not how they do it. Prefer `resume_conversation_reply` over transport-shaped names like `reconnect_stream`.
- If a function starts a larger workflow, name it as a workflow start, not as a low-level mechanism. Avoid names like `spawn_reply` when the code actually runs retrieval, synthesis, persistence, and follow-up work.
- Extract non-trivial branches into named helpers when the helper name makes the behavior easier to understand.

## Run the app

From the repo root, start backend, frontend, and docs with [`./start_app.sh`](./start_app.sh). Use `./start_app.sh --detach` to not tie up the shell (follow with `tail -f start_app.log`).

## Verification

IMPORTANT: Related work or not, ALL TESTS HAVE TO BE GREEN before committing anything.

Navigate to the respective frontend and backend folders and run the following before commit:

**Backend** (`backend/`):

1. `cargo check`
2. `cargo test`
3. `./scripts/cargo_audit.sh` (use `cargo update` when you intend to refresh `Cargo.lock`; keep the script allowlist short and remove entries as upstream crates publish fixes).

**Frontend** (`frontend/`):

1. `bun install --frozen-lockfile`
2. `bun run format:check` (Prettier)
3. `bun run lint` (ESLint)
4. `bun run check` (Svelte / `svelte-check`)
5. `bun run test` (unit tests)
6. `bun run test:e2e` (Playwright E2E — requires running stack: `./start_app.sh`)
7. `bun run build`
8. `bun audit --production`

_E2E requires a running stack (`./start_app.sh`). Not in CI — run locally before commit._

When to add unit vs E2E tests: [design.md#testing](./design.md#testing).
