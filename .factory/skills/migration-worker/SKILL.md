---
name: migration-worker
description: Handles data migration from global single-user storage to per-user storage for the Firebase Auth multi-user migration.
---

# Migration Worker

NOTE: Startup and cleanup are handled by `worker-base`. This skill defines the WORK PROCEDURE.

## When to Use This Skill

Features that migrate existing global data to per-user storage:
- Moving global channel subscriptions to per-user S3 paths
- Moving global conversations, highlights to per-user S3 paths
- Moving global preferences to per-user Firestore documents
- Creating migration CLI commands or admin endpoints
- Ensuring idempotency and data integrity

## Required Skills

None.

## Work Procedure

1. **Read feature context**: Read the feature description, preconditions, expectedBehavior, and fulfills. Read `.factory/library/architecture.md` for data storage layout.

2. **Read the spec**: Read `.specs/multi-user-workspace-with-firebase-auth-and-google-ready-sso.md` sections on migration behavior and data model split.

3. **Snapshot before migration**: Before any mutation, create a snapshot/inventory of the data being migrated. This enables rollback and verification.

4. **Write failing tests first (RED)**:
   - Add backend tests that assert migration behavior: correct destination paths, idempotency, canonical content preservation.
   - Run `cd backend && cargo test` — new tests should FAIL.

5. **Implement migration logic (GREEN)**:
   - Create a migration CLI subcommand or admin endpoint in the Rust backend.
   - Migration must be idempotent: running twice produces identical results with no duplicates.
   - Use S3 list + copy operations for S3 data (user-channel-subscriptions/, user-conversations/, user-highlights/, user-video-states/).
   - Use Firestore read + write for preferences (dastill_preferences/{uid}).
   - NEVER modify or delete canonical content (channels/, transcripts/, summaries/, video-info/, search-sources/, dastill_videos).
   - The operator UID should be resolved from the first email in OPERATOR_EMAIL_ALLOWLIST.

6. **Verify data integrity**:
   - Compare pre-migration snapshot with post-migration state.
   - Verify canonical content checksums are unchanged.
   - Verify per-user data is complete and correct.
   - Run migration a second time and verify counts are unchanged (idempotency).

7. **Run all validators**:
   ```
   cd backend && cargo test
   cd frontend && bun test tests
   cd frontend && bunx svelte-check --tsconfig ./tsconfig.json
   ```

8. **Commit**: `feat(migration): description`.

## Example Handoff

```json
{
  "salientSummary": "Implemented data migration CLI command that moves global subscriptions, conversations, highlights, and preferences to the first operator account. Migration is idempotent (verified by running twice with identical counts). Canonical content checksums unchanged. 325 backend tests pass.",
  "whatWasImplemented": "Added `cargo run --bin dastill -- migrate --operator-uid <uid>` CLI subcommand. Copies channel subscriptions from channels/ to user-channel-subscriptions/{uid}/. Copies conversations/ to user-conversations/{uid}/. Copies highlights/ to user-highlights/{uid}/. Copies preferences from dastill_preferences/user to dastill_preferences/{uid}. Pre-migration snapshot captured. Idempotency verified.",
  "whatWasLeftUndone": "",
  "verification": {
    "commandsRun": [
      { "command": "cd backend && cargo test", "exitCode": 0, "observation": "325 passed, 7 ignored — includes 7 new migration tests" },
      { "command": "cargo run --bin dastill -- migrate --operator-uid test-op-uid", "exitCode": 0, "observation": "Migrated 5 subscriptions, 3 conversations, 12 highlights, 1 preference doc" },
      { "command": "cargo run --bin dastill -- migrate --operator-uid test-op-uid", "exitCode": 0, "observation": "Second run: 0 new records created (idempotent)" },
      { "command": "sha256sum verification of canonical content", "exitCode": 0, "observation": "All canonical checksums match pre-migration snapshot" }
    ],
    "interactiveChecks": [
      { "action": "curl GET /api/channels with operator session", "observed": "Returns all 5 previously global channels under operator's subscriptions" },
      { "action": "curl GET /api/channels with fresh user session", "observed": "Returns only t3chat (seeded)" }
    ]
  },
  "tests": {
    "added": [
      {
        "file": "backend/tests/migration.rs",
        "cases": [
          { "name": "migration creates per-user subscriptions from global channels", "verifies": "Subscriptions migrate to correct S3 paths" },
          { "name": "migration is idempotent", "verifies": "Second run creates no duplicates" },
          { "name": "canonical content unchanged after migration", "verifies": "Global data integrity preserved" }
        ]
      }
    ]
  },
  "discoveredIssues": []
}
```

## When to Return to Orchestrator

- Per-user storage layer is not functioning (write failures)
- Cannot resolve the operator UID from OPERATOR_EMAIL_ALLOWLIST
- S3 access issues prevent reading/writing migration data
- Firestore access issues prevent preference migration
- Migration would need to modify canonical content (which is forbidden)
