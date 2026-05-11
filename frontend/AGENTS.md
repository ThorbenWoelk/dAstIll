# Frontend Agent Guide

Short entry point for agents working in `frontend/`. Detailed rules live in [../design.md](../design.md) — read that first.

## Responsive parity rule

This codebase is mobile-first with `@media (min-width: 640px)` / `@media (min-width: 960px)` overrides. One rule to prevent silent breakage:

**When you edit a base CSS rule or a breakpoint block, run the Playwright responsive spec for that route at both viewports before committing.**

Base styles apply at every width. Zeroing a padding, removing a width, or changing a flex direction at the base will silently affect desktop. The inverse holds: a desktop-only rule can re-enter mobile scope if you widen the selector.

Do not rely on manual resize testing. If a route has a desktop re-layout, it must have an E2E spec asserting:

- Mobile viewport (e.g. `375 × 812`) — mobile chrome visible, mobile layout present.
- Desktop viewport (e.g. `1280 × 900`) — desktop chrome visible, desktop layout present, desktop-only interactions work (e.g. internal sidebar scroll).

See [../design.md#responsive-regression-rule](../design.md#responsive-regression-rule).

## Verification before commit

From `frontend/`:

1. `bun run format:check`
2. `bun run lint`
3. `bun run check`
4. `bun run test`
5. `bun run test:e2e` (requires `./start_app.sh` running)
6. `bun run build`
7. `bun audit --production`

All green. No exceptions.

## Structure

- `src/routes/` — SvelteKit pages
- `src/lib/components/` — reusable components, grouped by feature
- `src/lib/` — shared type entry points and grouped frontend modules
- `src/lib/api/` — backend transport client and request helpers
- `src/lib/auth/` — auth state, storage, Firebase, browser login, and logout helpers
- `src/lib/config/` — app, docs, and maintenance-mode config
- `src/lib/navigation/` — app section navigation and workspace deep links
- `src/lib/platform/` — browser, native, service worker, runtime, and theme helpers
- `e2e/` — Playwright specs
- `tests/` — Bun unit tests

## Svelte 5 reminders

- `$state` / `$derived` returned from a function must be wrapped in getters/setters to preserve reactivity across the closure boundary.
- Do not branch on viewport width in JS. CSS owns the size story.
