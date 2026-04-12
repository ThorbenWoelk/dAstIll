# UI Tour Mobile Refresh PRD

## Problem

The public UI Tour no longer reflects the current product shape. It is still framed around desktop screenshots, the capture date is buried in the page body, and the in-product guide copy describes older behavior and value props instead of the current mobile web flow.

This creates two usability problems:

1. The docs misrepresent the product for visitors who first arrive on a phone.
2. The built-in guide does not clearly orient mobile web users around the current browse, read, queue, and chat loop.

## Goal

Refresh the UI Tour and the in-product guide so they describe the current mobile web experience, use current screenshots, and make the latest update date obvious at the top of the docs page.

## Non-goals

- Redesign the product UI itself in this task.
- Implement the larger mobile UX changes listed below.
- Rewrite unrelated docs pages.
- Add new dependencies or a new docs information architecture.

## Users or actors

- New visitors evaluating the app from the docs site.
- Mobile web users opening the workspace guide for the first time.
- Future maintainers who need a clear capture/update baseline for tour docs.

## Requirements

### Docs page

- The UI Tour page must show a clearly labeled latest update date near the top of the page.
- The page must lead with mobile web as the primary format.
- The screenshots on the page must be refreshed to match the current UI.
- The guide text on the page must describe the current mobile routes and behaviors.
- The page should still mention desktop where useful, but mobile should be the main framing.

### Screenshots

- Replace the stale UI Tour screenshots with fresh captures from the current web app.
- Prioritize phone-sized web screenshots for workspace, browse, queue, and chat flows.
- Image filenames and captions should make it clear what surface is shown.

### In-product guide

- Update `frontend/src/lib/workspace/home-tour.ts` copy so it matches the current product behavior and tone.
- Reduce outdated framing and desktop-heavy wording.
- Keep the current step count unless a smaller change is clearly safer.

## UX and usability ideas collected during this refresh

These are intentionally collected for follow-up rather than implemented in this task:

1. Replace the current mobile-first docs screenshots with a repeatable capture workflow so the page is easier to keep current after UI changes.
2. Rework the guide card for small screens so the copy is shorter and the actions are easier to scan one-handed.
3. Add a dedicated mobile queue screenshot state that shows actionable processing detail instead of only a clear/empty state.
4. Make the mobile chat entry point more obvious from the guide and product shell.
5. Revisit the mobile browse overlay so channel selection, filters, and add-source actions fit in one clear mental model.
6. Consolidate the earlier mobile audit notes into one prioritized roadmap instead of many flat `.specs/*.md` notes.
7. Add an explicit “what changed since last update” callout on the UI Tour page for future refreshes.
8. Capture both 390px and 430px mobile widths to avoid documenting a layout that only looks correct on one device class.

## Risks and open questions

- Screenshot capture depends on a stable local app state and seeded content.
- The best “latest update date” surface may require a small docs style adjustment to look intentional.
- Some earlier mobile audit items may make the guide feel dated again soon if those product changes land separately.
