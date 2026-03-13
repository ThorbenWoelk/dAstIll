# Tasks: Mobile Shell And PWA

## Current State
Follow-up mobile shell fixes are in place after the footer still appeared to slide below the visible viewport on workspace/queue and the app still allowed a second document-level scroll at the bottom. Workspace and queue now use a mobile viewport-height shell with an inner `main` row constrained to `minmax(0, 1fr)`, so the panel scrollers own vertical overflow instead of the page. Frontend `bun test`, `bun run check`, and `bun run build` all passed again, and a Playwright mobile screenshot comparison on `/?guide=3` produced identical viewport/full-page dimensions, confirming the outer page no longer scrolls in mobile content view.

## Steps
- [x] Add a failing frontend test for the service-worker registration rules.
- [x] Implement PWA registration logic and mount it from the root layout.
- [x] Add the manifest, service worker, and resized install icons to the frontend static assets.
- [x] Make the mobile bottom navigation fully opaque and keep it above decorative overlays.
- [x] Run frontend format, tests, checks, and build; then verify the generated PWA assets manually.

## Decisions Made During Implementation
- "PWA ability" in this scope means installability support: manifest, icons, and service-worker registration.
- The service worker stays intentionally minimal and pass-through only, so installability is added without changing API/network behavior or claiming offline support.
- Existing `favicon.png` artwork is reused to generate the 192px and 512px install icons instead of introducing a new icon set.
- The footer and mobile tab bar now both use the fully opaque `--surface-strong` token instead of the overlay tokens so mobile shell chrome is consistently non-transparent.
- Mobile fixed bars now read from shared CSS variables so the footer, tab bar, toast offsets, and mobile scroll padding all stay in sync across workspace and queue.
- A root layout helper updates `--mobile-viewport-offset-bottom` from `window.visualViewport` so fixed bottom chrome stays visible even when the mobile browser UI changes the visible viewport height.
- Workspace and queue use a dedicated mobile panel-shell class so the route wrapper itself is pinned to the viewport and only the inner panel scroll containers can continue scrolling at the bottom.
