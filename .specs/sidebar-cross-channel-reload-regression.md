# Sidebar Cross-Channel Reload Regression

## Problem

Selecting a video from a different sidebar channel collection can trigger a full sidebar reload instead of restoring the already loaded state for that channel.

## Goal

Preserve sidebar state when switching across channel collections, without regressing selected-video hydration, filter handling, or refresh behavior.

## Requirements

- Reproduce the regression with a focused automated test before changing implementation.
- Restore cached channel video state when switching channels, even if the current channel has different transient paging or sync-depth state.
- Keep existing refresh behavior for stale channel snapshots.
- Avoid broad changes to route bootstrap or preview-controller behavior unless the test proves they are required.

## Non-Goals

- Reworking the sidebar preview/session architecture.
- Changing unrelated channel overview navigation behavior.
