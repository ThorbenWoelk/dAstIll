# Spec: Mobile Workspace Tab Bar Bottom Anchor

## Goal

Restore the workspace mobile panel navigation (`Channels`, `Videos`, `Content`) to a bottom-anchored control instead of a top-mounted bar.

## Scope

- Update the shared mobile tab bar layout styles used by the workspace route.
- Ensure the mobile shell reserves bottom space for the fixed tab bar instead of top space.
- Preserve desktop layout and the existing mobile interaction behavior.

## Non-Goals

- Redesigning the tab bar visuals or changing labels/actions.
- Altering the shared section navigation used for workspace, queue, and highlights pages.
- Refactoring unrelated mobile shell spacing outside the tab bar regression.

## Acceptance Criteria

- On mobile widths, the workspace tab bar is fixed to the bottom edge of the viewport.
- The tab bar respects the bottom safe-area inset and no longer occupies the top edge.
- Workspace content is padded so the bottom tab bar does not cover interactive content.
- Frontend check, test, and build commands pass after the fix.
- The deployed `main` branch includes the fix and the production deploy workflow succeeds.
