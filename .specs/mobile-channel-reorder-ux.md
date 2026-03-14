# Mobile Channel Reorder UX

**Linear:** none

## Problem

On mobile, channel reordering in the workspace relies on desktop-style HTML drag events attached to the full channel card. That interaction has weak touch affordance, conflicts with normal card tapping, and gives almost no visual guidance about when reordering is available or where the channel will land.

## Goal

Make mobile channel reordering feel explicit and touch-native. Users should be able to understand at a glance when channels can be reordered, initiate the gesture from a dedicated control, see a clear target while dragging, and still have a non-drag fallback for accessibility.

## Requirements

- Mobile channel reordering is entered through an explicit reorder mode instead of relying on implicit whole-card dragging.
- In reorder mode, each mobile channel row exposes a dedicated drag handle with a touch-friendly target size and clear visual affordance.
- Dragging on mobile shows a lifted/active state and a visible drop target so the destination is understandable before release.
- A non-drag fallback is available on mobile for users who cannot or do not want to drag.
- Desktop channel reordering and queue channel rendering keep their current behavior unless a shared card change is required to support the mobile workspace flow cleanly.
- Channel sorting modes continue to gate reordering correctly: reordering is only available in custom order, and filtered/sorted derived views do not pretend to support manual reorder.
- Frontend verification covers targeted tests, format, type checking, and production build.

## Non-Goals

- Reworking the desktop reorder interaction.
- Replacing the saved custom channel order model or route persistence.
- Rebuilding the queue route’s channel list UX beyond any low-risk shared card support needed for the workspace change.

## Design Considerations

- Use an explicit mobile reorder mode with instructional copy and a clear exit action so the app separates “browse/select” from “rearrange”.
- Prefer a dedicated drag handle over whole-card dragging to reduce accidental selection and align with current mobile reorder patterns.
- Add a non-drag alternative such as move up/down controls in reorder mode so the interaction is still operable without drag gestures.
- Keep the visual language aligned with the existing workspace shell instead of introducing a new design system.

## Open Questions

- None at the moment. The interaction target is clear enough to implement without additional product input.
