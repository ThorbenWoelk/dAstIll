# Gap Scan Storage Reconcile

## Status
Accepted

## Context

The gap-scan worker can discover historical videos whose Firestore metadata rows are missing even though their transcript and summary artifacts already exist in object storage. The current insert path creates those videos as `pending/pending`, which makes the normal workspace treat them as not ready and can leave channel/video views effectively empty until background workers revisit them.

The desired behavior is to backfill Firestore metadata without regressing content readiness that has already been materialized in storage.

## Decision

- Reconcile initial transcript and summary statuses for newly inserted Firestore video rows against existing storage objects.
- Only apply this reconciliation on first insert; preserve existing Firestore processing state for existing rows.
- Add regression coverage for newly inserted videos whose transcript and summary artifacts already exist.

## Consequences

- Gap-scan metadata backfills become idempotent with respect to already-generated content.
- Historical videos with stored artifacts appear as ready immediately after Firestore backfill.
- Insert cost for brand-new videos includes lightweight storage existence checks.
