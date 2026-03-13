# Docs Overview Page Refresh

**Linear:** none

## Problem

The docs site overall is acceptable, but the overview page still reads like a marketing landing page instead of a documentation entry point. The current VitePress home hero and feature blocks make the page feel less like a technical reference than the rest of the site.

## Goal

Keep the existing docs shell intact while turning the overview page into a cleaner documentation-first landing page. The page should help readers orient quickly, surface the most useful entry points, and feel closer to a focused technical reference.

## Requirements

- The docs shell outside the overview page remains unchanged.
- The homepage uses a documentation-style layout instead of the current VitePress home hero.
- The landing page highlights the most important entry points with concise, scannable copy.
- The revised page keeps the existing documentation routes intact.
- The updated overview should take visual guidance from the referenced Ollama docs approach: restrained, documentation-first, and easy to scan.

## Non-Goals

- Reworking the global docs navigation, dark-mode behavior, or shared docs shell.
- Rewriting the technical content of the existing documentation pages.
- Adding a new API reference section that does not already exist in the repo.
- Changing the product frontend under `frontend/` or the backend under `backend/`.

## Design Considerations

- Prefer page-local styling over shared theme changes so the rest of the docs stay stable.
- Keep the visual language minimal and documentation-first, with emphasis on scanning and quick entry points.
- Use the existing sidebar and outline as the primary navigation surfaces and keep the overview page focused on orientation.

## Open Questions

- None at the moment. The user’s requested direction is specific enough to implement directly.
