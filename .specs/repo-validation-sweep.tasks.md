# Tasks: Repo Validation Sweep

## Current State
Backend and frontend verification are green against the current working tree. The sweep fixed frontend Prettier coverage for generated artifacts, stabilized local backend startup around renamed model env vars and stale credential-file paths, and updated stale Playwright expectations for anonymous chat and mobile browse empties. The next step is to stage the full requested diff, run the repo pre-commit hook on the staged tree, and push `main`.

## Steps
- [x] Create spec and task files for the repo validation sweep.
- [x] Run backend verification commands.
- [x] Run frontend verification commands, including E2E with the local stack when needed.
- [x] Fix compile errors, warnings, lint failures, and test regressions exposed by the sweep.
- [ ] Run the repo pre-commit hook against the staged tree.
- [ ] Commit and push the full requested working tree on `main` with plain `git`.

## Decisions Made During Implementation
- Treat this as a repo-wide validation pass over the existing dirty worktree instead of splitting ownership by originating spec.
- Use the repo's documented verification order and local pre-commit hook as the release gate before committing.
