# UI Tour Mobile Refresh Test Spec

## Acceptance criteria

- `docs/ui-tour.md` shows a latest update date near the top of the page.
- The UI Tour page is written as a mobile-web-first guide.
- The tour screenshots shown on the page are refreshed and correspond to the current mobile web UI.
- The in-product guide text in `frontend/src/lib/workspace/home-tour.ts` matches the current mobile flow.
- The new planning artifacts exist under `.specs/ui-tour-mobile-refresh/` and `.adr/`.

## Automated checks

- `cd frontend && bun run test -- home-tour`
- `cd frontend && bun run test -- workspace-guide`
- `cd frontend && bun run check`
- `cd frontend && bun run lint`
- `cd docs && bun run build`

## Manual checks

- Open the docs site and verify the latest update date is visible without scrolling past the intro.
- Review the UI Tour page at a narrow viewport and confirm the screenshot order and captions read naturally on mobile.
- Open the workspace on mobile web width and walk through the guide steps to confirm the copy still matches the visible UI.
- Verify the refreshed screenshots visually match the current product state.

## Edge cases

- Signed-out mobile web state with no manual library setup.
- Narrow phone widths around 375px where captions and figures stack tightly.
- Cases where the guide cannot select a video because auth is unavailable.

## Observability or failure signals

- Missing or broken image paths in the docs build.
- Guide tests failing because the step count or first-step title changed unexpectedly.
- Mobile screenshots showing stale or inconsistent route chrome versus the written guide.
