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

## Documentation Split

- `AGENTS.md`: agent workflow entry point, document map, repo-level instructions.
- `design.md`: design system and frontend engineering standards.

## Secrets and production config

**Follow these rules**
- Sensitive values for production belong in **GCP Secret Manager**, managed by **Terraform** from your local **`terraform.tfvars`** (same pattern as `youtube_api_key`, `backend_proxy_token`, `firebase_web_api_key`, etc.).

Terraform creates the secrets, writes secret versions from tfvars, and grants **Cloud Run** service accounts plus the **GitHub Actions deploy** service account `roles/secretmanager.secretAccessor` on the relevant secrets. See `terraform/secrets.tf` and `terraform/iam.tf`.

The **Release** workflow (`.github/workflows/deploy.yml`) deploys to Cloud Run and **mounts** Secret Manager secrets onto the service as named environment variables (for example `BACKEND_PROXY_TOKEN=dastill-backend-proxy-token:latest`). Non-secret runtime config uses GitHub **vars** or plain `env` in the workflow where documented.

**Anti-patterns to avoid**

- Application API keys or tokens in GitHub repository variables (use Terraform and Secret Manager instead).

**CI**

GitHub **encrypted secrets** in the repo are only for **CI authentication to GCP** (e.g. WIF and project id), not for storing app credentials.

Full list of boundaries, Firebase, and CI steps: [docs/operations/deployment.md](./docs/operations/deployment.md).

# Developer Guide

## Svelte 5 Reactive State Rules

- When returning `$state` or `$derived` from a function, use getters/setters to preserve the reactive boundary. The function scope becomes a closure that stays connected to the reactive proxies.

## Run the app

From the repo root, start backend, frontend, and docs with [`./start_app.sh`](./start_app.sh). Use `./start_app.sh --detach` to not tie up the shell (follow with `tail -f start_app.log`).

## Verification

IMPORTANT: Related work or not, ALL TESTS HAVE TO BE GREEN before committing anything.

Navigate to the respective frontend and backend folders and run the following before commit:

**Backend** (`backend/`):

1. `cargo check`
2. `cargo test`
3. `cargo audit` (use `cargo update` when you intend to refresh `Cargo.lock`; otherwise `cargo audit` alone is the usual local check).

**Frontend** (`frontend/`):

1. `bun install --frozen-lockfile`
2. `bun run format:check` (Prettier)
3. `bun run lint` (ESLint)
4. `bun run check` (Svelte / `svelte-check`)
5. `bun run test` (unit tests)
6. `bun run test:e2e` (Playwright E2E — requires running stack: `./start_app.sh`)
7. `bun run build`
8. `bun audit --production`

*E2E requires a running stack (`./start_app.sh`). Not in CI — run locally before commit.*

When to add unit vs E2E tests: [design.md#testing](./design.md#testing).

