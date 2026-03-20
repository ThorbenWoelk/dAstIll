# Spec: Channel reorder last-position drop

## Context

Dragging channels in the workspace sidebar can move a channel upward, but dropping a channel at the visual end of the list does not persist the new position after release.

## Goals

- Reproduce the failed "drop to last position" behavior in the channel reorder logic.
- Fix the drag-and-drop flow so dropping at the final visible insertion point appends the channel to the end of the custom order.
- Add regression coverage for the end-of-list drop path.

## Non-goals

- No redesign of channel management UI beyond the drop behavior needed for this bug.
- No changes to non-custom sort modes or filtered reorder behavior.

## Approach

1. Reproduce the issue in the pure reorder helpers and confirm the current logic cannot express an append target.
2. Add a failing regression test for moving a channel to the last position.
3. Update the reorder/drop helpers and sidebar interaction so the bottom insertion indicator on the final row maps to an append operation.
4. Re-run targeted frontend tests plus `bun run check` and `bun run build`.

## Validation

Frontend:

```bash
bun test frontend/tests/channel-workspace.test.ts frontend/tests/workspace-channels.test.ts
bun run check
bun run build
```
