# Tasks: Sidebar Cross-Channel Reload Regression

## Current State
Regression test is in place and passing. The workspace controller now keys cached channel views by stable scope + filter dimensions and clears cached channel entries when sync-date changes or a channel is deleted.

## Steps
- [x] Inspect the shared sidebar/workspace controller path and identify the likely regression surface.
- [x] Add a failing regression test for cross-channel selection without sidebar reload.
- [x] Implement the minimal cache-key or state-restore fix.
- [x] Run targeted tests, then broader relevant verification.
