# Mobile Readiness Overview

Status: **Open**

This replaces the numbered mobile audit/spec set with a smaller feature-oriented plan.

## Goal

Bring the mobile experience up to a repo-acceptable baseline without duplicating frontend
standards that already live in [design.md](../design.md).

## Source Of Truth

- Frontend design and interaction rules: [design.md](../design.md)
- Repo workflow and verification requirements: [AGENTS.md](../AGENTS.md)
- Local and Tauri workflows: [docs/local-development.md](../docs/local-development.md), [docs/mobile-tauri.md](../docs/mobile-tauri.md)

## Current Blocking Themes

- Touch targets and mobile-visible actions are still inconsistent.
- Navigation still depends too heavily on the left rail on small screens.
- Chat is functional on mobile but not yet space-efficient or fully touch-optimized.

## Feature Map

| Feature | Scope | Priority |
| --- | --- | --- |
| [feature-mobile-interaction-accessibility](./feature-mobile-interaction-accessibility.md) | touch targets, hover-to-touch migration, input zoom, typography, guide touch fixes | P0 |
| [feature-mobile-shell-navigation](./feature-mobile-shell-navigation.md) | bottom nav, nav rail retirement on mobile, responsive breakpoint structure | P0 |
| [feature-mobile-chat](./feature-mobile-chat.md) | compact chat input, conversation access, message-width cleanup, chat touch affordances | P1 |

## Completion Gate

Mobile readiness is complete only when:

1. The three feature specs above are complete.
2. The full frontend verification gate from [AGENTS.md](../AGENTS.md) passes.
3. DOM-visible mobile changes have Playwright coverage per [design.md](../design.md).
4. Real-device or simulator QA confirms there are no blocking regressions in touch, navigation, and text-entry flows.
