# dastill-mini Test Spec

## Acceptance criteria

- The maintenance landing page shows a sign-in or continue CTA into `dastill-mini`.
- A signed-in user can open `/mini` and see only their subscribed channels in the selector.
- Selecting a channel loads summaries for that channel and displays one active summary at a time.
- Previous and next controls move through the channel summary list in deterministic order.
- A signed-in user can mark a summary as read from `dastill-mini`.
- Read status persists through the S3-backed per-user video state path and is visible again on reload.
- The new planning artifacts exist under `.specs/dastill-mini/` and `.adr/`.

## Automated checks

- `cd frontend && bun run check`
- `cd frontend && bun run lint`
- `cd frontend && bun run test`
- `cd backend && cargo check`
- `cd backend && cargo test`

## Manual checks

- Open the landing page in maintenance mode and verify the sign-in CTA leads to login and then `/mini`.
- Sign in with a user that has channel subscriptions and verify the dropdown shows only those channels.
- Open a summary, move to next and previous summaries, and verify the content updates without leaving the page.
- Mark a summary as read, refresh the page, and verify the read state remains visible.
- Verify empty-state behavior for a user with no subscriptions or no available summaries.

## Edge cases

- A subscribed channel with zero summaries.
- A summary whose `video-info` row is missing optional display fields like thumbnail or channel name.
- A user with read state on videos that are no longer present in the summary list.
- Signed-out access to `/mini`.

## Observability or failure signals

- `/api/mini/*` requests returning S3 or serialization errors.
- The mini route falling back to the workspace bootstrap path.
- Read-state writes succeeding locally but not reappearing after reload.
