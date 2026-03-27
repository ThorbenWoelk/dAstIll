# Sidebar Filter State Refactor

## Problem

The workspace sidebar mixes state ownership across route state, sidebar state, and component-local preview state. The unread filter bug exposed three structural issues:

- filter state can be mutated through multiple code paths, not all of which trigger the same side effects
- `AcknowledgedFilter` is translated back and forth to `boolean | undefined`, which obscures intent and duplicates conversion logic
- `WorkspaceSidebar.svelte` owns preview-mode fetching, caching, reconciliation, virtualization, and UI rendering in one file

## Goal

Make sidebar filter behavior explicit and consistent by enforcing setter-based state transitions, using `AcknowledgedFilter` as the canonical app-level type, and extracting per-channel preview behavior out of `WorkspaceSidebar.svelte`.

## Requirements

- Document the Svelte state-management rule in `AGENTS.md`: when reactive state has setter methods or controller actions, callers must use them instead of mutating the backing rune directly.
- Keep `AcknowledgedFilter` as the filter type across sidebar state and route callbacks; only convert to `boolean | undefined` at API boundaries.
- Extract per-channel preview state and side effects from `WorkspaceSidebar.svelte` into dedicated modules with a smaller, testable API surface.
- Add regression coverage for the unread filter behavior at the logic layer and at the rendered UI layer.
- Preserve existing workspace and download-queue behavior for preview expansion, optimistic acknowledge updates, and route URL sync.

## Non-Goals

- Redesigning the sidebar UI
- Reworking unrelated workspace content loading paths
- Changing backend filter semantics

## Design Notes

- Prefer `.svelte.ts` controller modules for stateful preview logic so Svelte runes stay encapsulated behind a small API.
- Keep filter helpers pure where possible so they remain unit-testable without the Svelte runtime.
- Push API-shape conversions to the narrowest possible boundary, ideally the API call site.
