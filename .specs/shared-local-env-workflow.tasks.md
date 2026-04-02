# Tasks: Shared Local Env Workflow

## Current State
Shared env loading is wired into backend startup and `./start_app.sh`. The helper
script and documentation updates are in place, and targeted verification passed.

## Steps
- [x] Define the shared local env path and env precedence rules in repo code and docs.
- [x] Update backend startup to load `~/.config/dastill/backend.env` when present.
- [x] Update `./start_app.sh` to load shared backend/frontend env files for local startup.
- [x] Add a helper script that links worktree-local `.env` files to the shared config.
- [x] Update local development documentation for the new workflow.
- [x] Verify the shared env path works for startup-related commands and record the result.

## Decisions Made During Implementation
- Shared local env files live under `${XDG_CONFIG_HOME:-$HOME/.config}/dastill`.
- Backend startup precedence is shell env first, shared `backend.env` as the default file source, and worktree-local `backend/.env` as an explicit override.
- `./start_app.sh` injects frontend env from the shared dir without requiring `frontend/.env` to exist.
- Verification:
  - `cargo test --manifest-path backend/Cargo.toml local_env`
  - `cargo check --manifest-path backend/Cargo.toml`
  - temp-repo run of `scripts/link_shared_env.sh` migrated and linked env files under a temp XDG config dir
  - temp-copy run of `start_app.sh` failed early on a deliberately invalid model tag from shared `backend.env`, proving the shared env file is read before startup
