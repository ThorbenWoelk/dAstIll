# Tasks: Mobile Shell And PWA

## Current State
Implementation and verification are complete. The frontend now registers a service worker from the root layout in secure and localhost contexts, serves a manifest plus 192/512 install icons, and the mobile bottom navigation ships with an opaque surface and higher stacking order. Frontend `format:check`, `bun test`, `bun run check`, `bun run build`, and a built-preview smoke test of `/`, `/manifest.webmanifest`, `/sw.js`, `/icon-192.png`, and `/icon-512.png` all passed locally.

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
