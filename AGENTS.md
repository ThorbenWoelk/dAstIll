# Agent Guide

Keep this file short, operational, and focused on how to work in the repo.
Deeper domain-specific guidance belongs in dedicated docs and should be linked from here.

## Source Of Truth

- Frontend design system, Svelte frontend cleanliness rules, UI architecture guidance, file-size thresholds, and frontend testing expectations live in [DESIGN.md](./DESIGN.md).
- user docs in [./docs/](./docs/)

## How To Work Here

- Read this file first, then open the linked domain doc you need.
- Do not duplicate large guidance blocks across multiple markdown files.
- When frontend rules change, update `DESIGN.md` and keep only the pointer here.
- Keep repo guidance legible for agents: short entry points here, detailed source-of-truth docs elsewhere.

## Documentation Split

- `AGENTS.md`: agent workflow entry point, document map, repo-level instructions.
- `DESIGN.md`: design system and frontend engineering standards.

# Developer Guide

## Run the app

From the repo root, start backend, frontend, and docs with [`./start_app.sh`](./start_app.sh). Default ports: frontend **3543** (`FRONTEND_PORT`), backend **3544** (`BACKEND_PORT`), docs **4173** (`DOCS_PORT`). Use `./start_app.sh --detach` when you need a long-running process without tying up the shell (logs: `start_app.log`; follow with `tail -f start_app.log`).

## Verification

Related work or not, all tests have to be green before committing anything.

Run the blocks that match what you changed.

### Backend (`backend/`)

1. `cargo check`
2. `cargo test`

### Frontend (`frontend/`)

1. `bun install --frozen-lockfile`
2. `bun run format:check` (Prettier)
3. `bun run lint` (ESLint)
4. `bun run check` (Svelte / `svelte-check`)
5. `bun run test` (unit tests)
6. `bun run test:e2e` (Playwright E2E - requires running stack: `./start_app.sh`)

### Docs (`docs/`)

1. `bun install --frozen-lockfile`
2. `bun run build`

### Dependency audits

**Backend** (`backend/`): install [`cargo-audit`](https://github.com/rustsec/rustsec) once with `cargo install cargo-audit --locked`, then `cargo update` and `cargo audit` (use `cargo update` when you intend to refresh `Cargo.lock`; otherwise `cargo audit` alone is the usual local check).

**Frontend** (`frontend/`):

1. `bun install --frozen-lockfile`
2. `bun audit --production`

**Docs** (`docs/`):

1. `bun install --frozen-lockfile`
2. `bun audit --production`

### Release images (optional local parity)

From the repo root:

1. `docker build ./backend`
2. `docker build ./docs`
3. `docker build ./frontend`

### Not in CI

**Frontend production bundle** (`frontend/`): `bun run build` (release Docker build runs this inside the image).

**E2E/Playwright** (`frontend/`): runs in pre-commit hook when `frontend/src/`, `frontend/e2e/`, or Playwright config changes. Requires running stack (`./start_app.sh`). Not in CI workflow.

When to add unit vs E2E tests: [DESIGN.md#testing](./DESIGN.md#testing).
