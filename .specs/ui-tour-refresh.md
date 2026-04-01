# UI Tour Refresh

## Problem

The existing `docs/ui-tour.md` is a light summary of the product surfaces, but it does not read like a purposeful tour and its screenshots can drift away from the current product UI. That makes the documentation less useful for understanding the actual workspace, queue, chat, and mobile flows.

## Goal

Replace the current UI tour page with a from-scratch walkthrough of the product's major surfaces that is anchored in fresh screenshots captured from the current application state.

## Requirements

- The `UI Tour` documentation page is rewritten from scratch instead of lightly editing the existing copy.
- The page presents the current major product surfaces in a coherent tour format, not just a flat route list.
- The page includes fresh screenshots captured during this work and stored in the repo.
- The screenshots cover meaningful desktop and mobile states of the current UI.
- The docs page remains compatible with the existing VitePress docs site and navigation.
- If the current in-app guide text is materially inconsistent with the rewritten docs, align the relevant copy in scope.

## Non-Goals

- Redesigning the product UI itself.
- Reworking the overall docs information architecture or navigation.
- Exhaustively documenting every route, edge state, or implementation detail outside the UI tour page.

## Design Considerations

- The docs page should feel intentionally structured and visual while still fitting the restrained documentation design system already used in the repo.
- Screenshot selection should emphasize real workflows and information density over empty or placeholder states.
- Any supporting style changes should be reusable page-level docs styles, not one-off inline clutter.

## Open Questions

- Whether the rewritten docs should exactly mirror the in-app feature guide copy or only stay directionally aligned on major feature claims.
