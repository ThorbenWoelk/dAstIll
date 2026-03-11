# Shareable View URL State

**Linear:** none

## Problem

The app currently keeps workspace selection state only in localStorage. That makes the current view sticky for one browser, but it cannot be shared as a link, reopened reliably on another machine, or restored from a copied URL.

## Goal

Selected channel, selected video, content tab, and active filters should be encoded in the URL so the current app view can be copied and shared at any time. Opening a shared URL should restore that view without depending on localStorage.

## Requirements

- The main workspace route `/` must reflect the selected channel, selected video, content tab, video type filter, and acknowledged filter in the query string.
- Opening `/` with those query parameters must restore the corresponding view, with URL state taking precedence over localStorage state.
- The download queue route `/download-queue` must reflect the selected channel and active queue tab in the query string.
- Opening `/download-queue` with those query parameters must restore the corresponding queue view, with URL state taking precedence over localStorage state.
- Invalid or stale query parameter values must be ignored safely and fall back to the existing selection resolution behavior.
- URL updates must happen automatically as the user changes the relevant view state.
- Existing workspace persistence behavior in localStorage must continue to work as a fallback when no explicit URL state is present.
- Automated tests must cover URL parsing/serialization and the restored-state precedence rules.

## Non-Goals

- No changes to mobile-only panel state such as the temporary channels/videos/content drawer tabs.
- No back/forward history redesign beyond keeping the current URL in sync.
- No backend API changes.

## Design Considerations

- Prefer small pure helpers for parsing and serializing URL state so the route components only orchestrate state application.
- Keep query parameter names short but readable because they are user-visible share links.
- Avoid introducing full navigation loops when syncing URL state; updating the current history entry is sufficient for this scope.

## Open Questions

- None at the moment.
