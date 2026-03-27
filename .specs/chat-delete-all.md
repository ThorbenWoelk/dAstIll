# Chat Delete All

**Linear:** n/a

## Problem

The chat UI only supports deleting one conversation at a time. When a user wants to clear their chat history, they have to repeat the same destructive action for every thread in the sidebar.

## Goal

Users can remove their entire chat history from the chat UI in one explicit action, with clear confirmation and immediate UI feedback once the history is gone.

## Requirements

- The chat sidebar exposes a bulk delete action when conversations exist.
- Triggering the bulk delete action requires an explicit confirmation step before any chat data is removed.
- Confirming bulk delete removes every stored conversation and clears the active thread in the chat workspace.
- After bulk delete completes, the chat route shows the empty-state experience instead of a stale thread.
- The implementation includes automated coverage for the new delete-all behavior and the rendered UI affordance.

## Non-Goals

- Adding undo / restore for deleted conversations.
- Changing single-conversation delete behavior beyond keeping it compatible with the new bulk action.
- Reworking the broader chat layout or navigation.

## Design Considerations

- A dedicated backend delete-all endpoint keeps the operation simple for the client and avoids chaining many per-conversation deletes from the browser.
- The destructive action should reuse the existing confirmation modal pattern so the risk profile stays obvious and consistent with single-delete.
- The sidebar already owns conversation management controls, so the bulk action should live there rather than in the message composer or header.

## Open Questions

- None. This implementation assumes the bulk action should be available to the same users who can already delete single conversations.
