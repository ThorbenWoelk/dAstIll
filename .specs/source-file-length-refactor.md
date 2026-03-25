# Source file length refactor

## Problem

Several first-party source files have grown very large. Oversized files make code review harder, increase merge conflict risk, obscure boundaries between concerns, and slow navigation. An audit of project source (excluding `node_modules`, Rust `target`, `.svelte-kit`, and build artifacts) identified clear outliers in the frontend and backend.

## Goal

Bring oversized application code back to a maintainable scale by splitting along natural boundaries (UI sections, domain modules, route-specific logic). Success is measured by smaller, cohesive files, unchanged user-visible behavior, and a documented policy so future growth is caught early.

## Requirements

- Reduce or eliminate "god files" above roughly **1000 lines** in application code (`frontend/src`, `backend/src`), prioritizing the worst offenders first.
- **Highest priority splits** (from audit):
  - `frontend/src/routes/+page.svelte` (~2700+ lines)
  - `backend/src/services/chat.rs` (~1900 lines)
  - `frontend/src/lib/components/workspace/WorkspaceSidebar.svelte` (~1700 lines)
  - `frontend/src/routes/chat/+page.svelte` (~1270 lines)
  - `frontend/src/lib/workspace/sidebar-state.svelte.ts` (~1030 lines)
  - `backend/src/services/summarizer.rs` (~1000 lines)
- **Secondary tier** (roughly 500–900 lines): plan incremental extraction where a single file mixes unrelated concerns; no requirement to shrink every file in this band in one pass.
- Refactors must preserve existing behavior: run relevant frontend checks/tests and backend `cargo test` / `cargo check` for touched areas.
- New modules and components should follow existing naming, import, and design patterns in the repo (see `AGENTS.md` / `CLAUDE.md` for UI tokens and style).
- Document agreed **line-count guidance** in this spec or in a short comment in `AGENTS.md` only if the team wants it enforced as policy (optional follow-up).

## Non-Goals

- Changing product behavior or UX as part of file splits (refactor-only unless a separate spec says otherwise).
- Trimming third-party, generated, or vendored code (`node_modules`, `target` build outputs, `.svelte-kit`, `frontend/build`, VitePress cache).
- Rewriting `backend/openapi.postman.yaml` for size; large API specs are acceptable as data.
- Introducing a hard CI gate on line count in the first iteration unless explicitly decided later.
- Drive-by renames or style-only churn across files not touched for splitting.

## Design Considerations

- **Svelte routes**: Prefer extracting presentational chunks into `$lib/components`, and moving pure logic into `$lib` TypeScript modules. Keep route files as composition and wiring.
- **Rust services**: Split by subsystem (e.g. streaming vs persistence vs prompts) or by type boundaries already implied by `mod` usage; avoid circular dependencies between new modules.
- **State**: `sidebar-state.svelte.ts` may split by concern (e.g. persistence vs derived UI state) only if boundaries are clear and tests still cover behavior.
- Order work by **risk and size**: the home `+page.svelte` and `chat.rs` deliver the most maintainability per effort but need careful regression testing.

## Audit snapshot (hand-written source)

Approximate line counts; rerun `wc -l` after major edits.

| Tier | Lines (approx.) | Path |
|------|----------------:|------|
| Critical | ~2627 | `frontend/src/routes/+page.svelte` (reduced; shell pieces in `WorkspaceDesktopTopBar` / `WorkspaceMobileBrowseOverlay`) |
| Critical | ~1910 (+ `constants` / `intent`) | `backend/src/services/chat/mod.rs` (service body; further splits possible) |
| Critical | ~1517 (+ filter subcomponent) | `frontend/src/lib/components/workspace/WorkspaceSidebar.svelte` |
| Critical | ~1066 (+ chat UI subcomponents) | `frontend/src/routes/chat/+page.svelte` |
| Critical | ~1033 | `frontend/src/lib/workspace/sidebar-state.svelte.ts` |
| Was critical | ~1017 total | `backend/src/services/summarizer/` (`mod.rs` ~653, `transcript_compare.rs` ~270, `prompts.rs` ~94) |
| Long | 500–914 | Multiple handlers, db, and workspace components (see prior audit list) |

Supporting files from this effort: `summary-tracking-id.ts`, `WorkspaceDesktopTopBar.svelte`, `WorkspaceMobileBrowseOverlay.svelte`, `sidebar-filter-options.ts`, `WorkspaceSidebarVideoFilterControl.svelte`, `services/summarizer/*`, `services/chat/constants.rs`, `services/chat/intent.rs`.

## Open Questions

- Whether to add an automated check (e.g. script or lint) with a soft or hard threshold after the first wave of splits.
- Preferred maximum line count for Svelte pages vs Rust modules (teams often use 300–500 for components, higher for thin orchestration).
