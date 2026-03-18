# UI Tour

## Product Surfaces

The current product UI is organized into three route-level experiences:

- main workspace (`/`)
- download queue (`/download-queue`)
- highlights (`/highlights`)

## Main Workspace

The workspace combines:

- channel management
- video browsing
- transcript and summary reading/editing
- search with result highlighting
- acknowledgement tracking

### Desktop workspace

![Desktop workspace](./images/desktop-1280.png)

## Queue Views

The queue surfaces operational backlog and incomplete content states.

### Queue with outstanding items

![Queue with items](./images/desktop-queue-with-items.png)

### Queue verification state

![Queue check](./images/desktop-queue-check.png)

## Mobile Layouts

The mobile screenshots in this repo document how the workspace adapts across:

- channels
- videos
- content view
- queue details

### Mobile full workspace

![Mobile full workspace](./images/mobile-375-full.png)

### Mobile content view

![Mobile content view](./images/mobile-375-content-view.png)

### Mobile queue details

![Mobile queue details](./images/mobile-375-queue-details.png)

## Why the UI Matters Architecturally

The product UI is not a thin shell over CRUD. It reflects multiple backend lifecycle states:

- transcript readiness
- summary readiness
- quality evaluation availability
- search indexing coverage
- acknowledgement state

That is why the backend exposes combined bootstrap and snapshot payloads instead of forcing the frontend to infer everything from fragmented endpoint calls.
