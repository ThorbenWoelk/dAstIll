# Spec: Compile warning cleanup

## Context

The repo has just gone through a broad DRY and clean-code refactor. Before any further work, the codebase should compile and check without warnings so the validation loop stays trustworthy.

## Goals

- Identify all current compile and check warnings in the backend and frontend.
- Remove the warnings without regressing behavior.
- Re-run the relevant validation commands and confirm they are clean.

## Non-goals

- No unrelated refactors beyond what is required to eliminate warnings.
- No product behavior changes.

## Approach

1. Run the backend and frontend compile/check commands that can emit warnings.
2. Fix warnings in the smallest possible patches, preserving the existing refactor work.
3. Re-run the same commands until they are clean.
4. Record the final validation results in the tasks file.

## Validation

Backend:

```bash
cargo build
cargo test
cargo clippy --all-targets -- -D warnings
cargo build --release
```

Frontend:

```bash
bun run check
bun run build
```
