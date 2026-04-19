# Test Spec: Redesign Unification Verification

## Acceptance Criteria
- [ ] `/queue` route loads and renders the full-page queue.
- [ ] Navigating between sections (Workspace, Highlights, Queue, Chat) maintains the 3-column shell on desktop.
- [ ] Mobile bottom navigation bar is visible and fixed on all pages except `/mini`.
- [ ] Clicking a nav item on the mobile bottom bar correctly switches the section.
- [ ] Highlights Sidebar allows filtering by channel, matching Workspace behavior.

## Proof for the Current Increment
### Automated Checks (Playwright)
- `tests/redesign-shell.spec.ts`:
    - Renders Workspace, Highlights, and Queue at 1280px, asserting Sidebar is visible.
    - Renders all core sections at 375px, asserting `.mobile-bottom-nav` exists and is fixed.
    - Asserts `/queue` contains at least one processing, pending, or completed row.
    - Checks that `/highlights` with sidebar interaction correctly updates the content list.

### Manual Checks
- Verify swipe navigation (if applicable) doesn't conflict with bottom bar.
- Test safe area padding on iOS simulator (Tauri).
- Ensure "Add Source" drawer works from the unified Highlights sidebar.

## Edge Cases
- **Empty States:** The new `/queue` page should show a neutral "Nothing in the queue" state when empty.
- **Deep Links:** Navigating directly to a video from Highlights should correctly set up the Workspace shell.
- **Logged Out:** Mobile bottom nav should handle public/restricted routes appropriately (or be hidden).

## Observability or Failure Signals
- **Lighthouse Scores:** Monitor layout shift (CLS) and accessibility (ARIA labels for new nav buttons).
- **Error States:** Failed items in the queue must persist "Details" and "Retry" actions.
