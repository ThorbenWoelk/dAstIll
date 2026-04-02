# Shared Local Env Workflow

## Problem

Local development currently relies on ignored per-worktree `.env` files such as
`backend/.env` and `frontend/.env`. When new worktrees are created for feature work,
those files are absent, which blocks `./start_app.sh` and makes local E2E validation
unreliable.

## Goal

Move local developer environment configuration to a shared machine-local directory at
`~/.config/dastill/` so every worktree can boot the stack and run E2E checks without
copying ignored secrets into each checkout.

## Requirements

- Local backend startup must load developer config from `~/.config/dastill/backend.env`
  without requiring `backend/.env` to exist in each worktree.
- Local frontend startup through `./start_app.sh` must load developer config from
  `~/.config/dastill/frontend.env`.
- Existing shell environment variables must continue to override file-based defaults.
- Worktree-local `.env` files may still exist as explicit overrides for debugging, but
  the shared config directory becomes the documented default workflow.
- The repo must provide a helper for new worktrees to connect `backend/.env` and
  `frontend/.env` to the shared files when developers want direct tool compatibility.
- Local development docs must explain the shared config layout, setup steps, and env
  precedence clearly enough that a new worktree can run the app and E2E tests.

## Non-Goals

- Changing production secret management or deployment configuration.
- Copying secret values into the repository.
- Reworking the full git hook strategy beyond what is needed for the shared env flow.

## Design Considerations

- Shared local config should follow the XDG-style default path so it is stable across
  worktrees and remains outside git.
- Backend binaries should support the shared config directly so ad hoc local commands
  keep working outside `./start_app.sh`.
- Compatibility matters during migration: existing local `.env` files should not break.

## Open Questions

- None.
