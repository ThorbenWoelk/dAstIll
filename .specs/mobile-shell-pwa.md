# Mobile Shell And PWA

**Linear:** none

## Problem

On mobile, the bottom navigation uses a translucent surface that lets page content and the global grain overlay show through, which reduces legibility. The app also lacks the minimum PWA wiring needed for installability, so it cannot be added cleanly to the home screen as an app-like experience.

## Goal

The mobile bottom navigation should render as an opaque, legible app chrome, and the frontend should expose the basic assets and browser integration needed for a standard installable PWA experience.

## Requirements

- The mobile bottom navigation renders with an opaque background and remains visually above decorative overlays.
- The frontend exposes a web app manifest with the app name, theme colors, start URL, and install icons.
- The frontend registers a service worker in secure or local development contexts so the app satisfies baseline PWA installability requirements.
- Existing app metadata continues to advertise the correct app name, theme color, and iOS home-screen support.
- The change is verified with frontend tests, static checks, and a production build.

## Non-Goals

- Adding offline-first caching, background sync, or push notifications.
- Designing a custom in-app install prompt.
- Reworking the mobile navigation information architecture.

## Design Considerations

Use the neighboring `toto-chores` app only as a reference for the minimum manifest and service-worker shape, but keep the implementation aligned with this SvelteKit app's existing static asset and layout structure. Reuse the current app icon artwork instead of introducing a new icon set.

## Open Questions

- None. "PWA ability" is interpreted here as installability metadata plus service-worker registration, not an offline feature set.
