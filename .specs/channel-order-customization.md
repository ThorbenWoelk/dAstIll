# Spec: Custom Channel Ordering in Left Sidebar

## Problem
Channels in the left sidebar follow backend-added order only, so users cannot customize their preferred browsing sequence.

## Goals
- Let users reorder channels directly in the left sidebar.
- Persist custom order across reloads.
- Keep existing channel selection and workspace state behavior intact.

## Non-Goals
- Cross-device/server-synced ordering.
- Additional backend APIs or schema changes.

## Approach
- Implement drag-and-drop reordering in the channel list UI.
- Persist ordered channel id list in existing workspace localStorage snapshot.
- Reconcile persisted order with fetched channels (ignore stale IDs, append new channels).

## Verification
- Frontend type checks pass.
- Frontend production build passes.
- Manual behavior: reorder persists after reload.
