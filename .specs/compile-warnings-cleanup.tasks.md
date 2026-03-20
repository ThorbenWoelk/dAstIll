# Tasks: Compile warning cleanup

## Current State

The warning cleanup verification is complete. All selected backend and frontend compile/check commands ran clean, so no additional code changes were required beyond the tracking files.

## Steps

- [x] Inspect the repo state and decide the validation commands that can emit warnings.
- [x] Write a dedicated warning-cleanup spec and tasks file under `.specs/`.
- [x] Run backend and frontend compile/check commands to capture active warnings.
- [x] Fix each warning with focused code changes.
- [x] Re-run validation until the warning-producing commands are clean.

## Decisions Made During Implementation

- Keep the warning cleanup separate from the broader DRY refactor tracking so the remaining work is obvious.
- Treat `cargo clippy --all-targets -- -D warnings` as a required backend warning gate, since it catches compile-adjacent issues that would otherwise accumulate.
- Verification found no active warnings in `cargo build`, `cargo test`, `cargo clippy --all-targets -- -D warnings`, `cargo build --release`, `bun run check`, or `bun run build`.
