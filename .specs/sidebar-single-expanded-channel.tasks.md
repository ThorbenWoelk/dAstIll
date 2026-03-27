# Tasks: Sidebar Single Expanded Channel

## Current State

Regression coverage is in place and the sidebar now normalizes preview expansion to a single active channel during restore, auto-expansion, and manual toggling. Targeted tests, frontend lint, `svelte-check`, production build, and the staged pre-commit hook all passed.

## Steps

- [x] Inspect the sidebar preview selection and expansion flow.
- [x] Write a failing regression test for single-channel expansion.
- [x] Implement single-expanded-channel normalization in preview state.
- [x] Verify with targeted tests, lint, `svelte-check`, build, and pre-commit.

## Decisions Made During Implementation

- The channel card active state will continue to map to preview expansion, but expansion must become mutually exclusive so the UI can only show one active preview row at a time.
