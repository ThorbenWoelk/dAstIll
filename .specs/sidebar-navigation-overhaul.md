# Spec: Sidebar Navigation Overhaul

## Goal

Make sidebar channel and video navigation feel lightweight across workspace and channel-overview flows by preserving preview UI state, avoiding unnecessary reloads, and only fetching more data when navigation truly needs it.

## Problems

- Clicking a video from a channel other than the currently selected one can trigger a full selected-channel list reload before the content panel finishes switching.
- Navigating from the channel overview sidebar into the workspace loses expanded per-channel preview state, so previously opened channel sections collapse.
- Per-channel preview lists do not share enough transient state across routes, which creates avoidable fetches and visual churn.

## Requirements

- Cross-channel video selection must reuse already loaded sidebar preview data when possible instead of eagerly reloading the full selected-channel list.
- Expanded per-channel preview state must survive navigation between the channel overview route and the main workspace route within the same browsing session.
- Sidebar preview fetching must stay incremental: preview lists load previews by default and only escalate to full channel loads when needed for the active interaction.
- The resulting behavior must be covered by targeted automated tests.

## Implementation Notes

- Introduce a shared in-memory preview session/cache for per-channel sidebar collections so route remounts can restore expansion state and already loaded preview rows.
- Seed selected-channel cache/state from clicked sidebar preview rows before triggering route/content navigation.
- Keep route helpers pure where possible so the navigation policy can be tested cheaply.
