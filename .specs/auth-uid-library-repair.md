# Auth UID Library Repair

## Status
Accepted

## Context

The app stores channel subscriptions, per-video user state, and preference documents by Firebase UID rather than email. After the GCP/Firebase project migration, `thorben.woelk@gmail.com` now authenticates as UID `0KRDh7vsaBXG3vP0Yu172O3lx9o1`, but the expected multi-channel library remains under orphaned UID `SuyMuuAOBGd7kIJ1lUUc8i6f2eE2`, which no longer exists in the current Firebase Auth project.

This leaves the signed-in user seeing only the single Theo subscription that was created under the new UID.

## Decision

- Repair the live user data by copying the orphaned UID-scoped library records into the current Firebase UID for `thorben.woelk@gmail.com`.
- Migrate:
  - S3 `user-channel-subscriptions/<uid>/`
  - S3 `user-video-states/<uid>/`
  - Firestore `dastill_preferences/<uid>`
- Keep the migration additive and non-destructive for the current UID.

## Consequences

- The current signed-in account regains the expected channel list, read state, and channel order.
- Existing new-UID records are preserved or merged rather than deleted.
- Longer-term product work may still be needed if the app should reconcile Firebase UID changes automatically.
