# App Dark Mode

**Linear:** none

## Problem

The product frontend is currently light-only while the docs already support an appearance toggle. That leaves the app inconsistent with the docs experience and prevents users from switching to a dark theme for longer reading and browsing sessions.

## Goal

Add dark mode to the product frontend with behavior that mirrors the docs appearance handling: the app should respect an existing preference on load, fall back to the system color scheme when no explicit preference exists, and expose a light/dark switch in the app UI.

## Requirements

- The app supports both light and dark themes.
- Theme application happens early enough to avoid a visible flash of the wrong theme on load.
- The theme behavior mirrors the docs pattern: explicit preference is persisted, and the default follows the system color scheme when unset.
- A shared theme toggle is available in the product frontend header across the main app surfaces.
- Core app surfaces remain readable and visually coherent in both themes.
- The app updates browser theme metadata appropriately for the active theme.

## Non-Goals

- Changing the docs frontend.
- Introducing a third user-facing theme mode selector beyond the docs-style light/dark switch.
- Redesigning the app layout or navigation beyond the changes required to place the toggle and support dark mode.

## Design Considerations

- Use the existing semantic CSS variable approach and extend it with a `.dark` token set rather than duplicating route-level styles.
- Keep theme state handling small and testable through pure helpers plus a thin DOM integration layer.
- Prefer a shared theme toggle component over route-specific copies, even if the current headers are duplicated.

## Open Questions

- None at the moment. The requirement to mirror the docs behavior is specific enough to implement directly.
