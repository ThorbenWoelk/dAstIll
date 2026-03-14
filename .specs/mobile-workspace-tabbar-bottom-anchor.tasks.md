# Tasks: Mobile Workspace Tab Bar Bottom Anchor

## Current State

The workspace mobile footer bar is restored to the bottom, and the conflicting mobile `Workspace / Queue / Highlights` selector now renders inline in the header on pages that already have a dedicated bottom tab bar. Frontend checks, tests, and build passed, and mobile screenshots confirm the workspace and queue footer bars no longer overlap the section selector.

## Steps

- [x] Identify the component and CSS rule that moved the workspace mobile tab bar to the top.
- [x] Restore bottom anchoring and bottom shell spacing for the mobile workspace tab bar.
- [x] Run frontend formatting, checks, tests, and build verification.
- [ ] Verify the tab bar position on a mobile viewport and deploy the fix to `main`.

## Decisions Made During Implementation

- The fix should target the shared mobile tab bar CSS instead of the `WorkspaceMobileTabBar.svelte` markup, because the component structure already renders a dedicated bottom navigation container.
- Pages that already ship with a dedicated bottom mobile tab bar should render the section selector inline in the header on mobile instead of stacking a second floating bottom control above the footer bar.
