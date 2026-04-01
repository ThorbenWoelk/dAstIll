# Tasks: Firestore Query Scalability

## Current State
Audit complete. Hot request paths still rely on `load_all_videos()` full-collection reads, especially in channel browsing and chat-related flows. Terraform already manages Firestore field-index policy, but the query indexes needed to remove those scans are not yet part of the defined design.

## Steps
- [x] Create spec and task files for Firestore query scalability.
- [ ] Inventory all request-path `load_all_videos()` consumers and rank them by user-facing impact.
- [ ] Define the required Firestore indexes and query shapes in Terraform for the first migration pass.
- [ ] Define replacement query helpers for channel paging, channel snapshot support, chat suggestions, and recent-activity flows.
- [ ] Define the migration boundary for remaining offline, admin, or stats callers that can stay scan-backed for now.
- [ ] Define performance, correctness, and parity verification for the new indexed read paths.

## Decisions Made During Implementation
- This pass starts with hot user-facing request paths first.
- Existing endpoint paths and response payload shapes remain unchanged.
- Firestore indexes are acceptable where they materially reduce request-path scans and latency risk.
