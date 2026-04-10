# Feature: Mobile Shell Navigation

Priority: **P0**

## Outcome

Replace the current small-screen shell dependence on the collapsed left rail with a mobile-first navigation structure that matches repo design intent and scales better to tablet layouts.

## Source Of Truth

Use [design.md](../design.md) for shell/navigation principles and avoid re-defining visual-system rules here.

## Consolidated Scope

This feature replaces the intent of:

- `02-mobile-bottom-nav.md`
- `07-mobile-nav-rail.md`
- `10-responsive-breakpoints.md`

## Requirements

### 1. Shared Mobile Bottom Navigation

Implement a shared mobile bottom navigation component for the main app sections.

Requirements:

- mounted at the app shell level
- visible on small screens
- uses the existing section-routing model instead of inventing a parallel nav system
- integrates with the existing `mobileBottomBar` state where that store already carries footer intent
- respects safe-area and viewport-offset handling already used by the shell

### 2. Retire The Mobile Left-Rail Dependency

Once the bottom navigation exists, the current left rail must stop being the primary mobile navigation surface.

Acceptance:

- the mobile shell does not reserve the current rail width unnecessarily
- hover-only labels are no longer the only way to understand primary navigation
- the mobile user menu and section switching remain accessible after the rail change

### 3. Add A Usable Tablet Tier

The shell should not treat tablets exactly like narrow phones.

Scope:

- introduce a deliberate intermediate responsive layout where justified
- start with surfaces that benefit most from extra width, especially workspace and chat
- keep changes incremental and behaviorally testable rather than redesigning every page at once

## Acceptance Criteria

- primary navigation is explicit and touch-usable on phone-sized screens
- the mobile shell reclaims space currently wasted by the collapsed rail
- at least one tablet-width path behaves differently from the narrow-phone layout where that improves usability

## Verification

Required:

- `bun install --frozen-lockfile`
- `bun run format:check`
- `bun run lint`
- `bun run check`
- `bun run test`
- `bun run build`
- `bun audit --production`

Required Playwright coverage:

- bottom-nav rendering and route switching on a mobile viewport
- mobile shell layout sanity on at least one primary route
- one tablet-width assertion if a dedicated tablet layout is introduced
