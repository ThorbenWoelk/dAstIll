# Sync Depth Boundary Regression

## Problem

The workspace can present an older "Synced to" boundary than the stored channel sync boundary without the user explicitly extending history. That makes the UI look like the app automatically pushed the oldest synced date backward.

## Goal

The sync boundary only moves backward when the user explicitly chooses an older date or clicks the history loading control. Passive data loads, refreshes, and filter changes must not make the displayed sync boundary appear older.

## Requirements

- The displayed sync boundary in the main workspace must prefer the persisted or derived channel sync boundary by default.
- Older loaded ready videos may influence the displayed boundary only when they were loaded through an explicit history expansion action.
- Manual earliest sync date updates remain authoritative.
- The behavior is covered by a frontend regression test.

## Non-Goals

- Changing backend sync-depth derivation rules.
- Reworking the download queue sync-depth presentation.

## Design Considerations

Use a small pure frontend helper so the display decision is testable without mounting the full Svelte route.

## Open Questions

- None.
