---
name: rust-backend-worker
description: Implements Rust backend performance optimizations (parallelization, caching, headers)
---

# Rust Backend Worker

NOTE: Startup and cleanup are handled by `worker-base`. This skill defines the WORK PROCEDURE.

## When to Use This Skill

Features involving Rust backend changes: S3 operation parallelization, read cache granularity, API response headers, query deduplication, and related performance improvements.

## Required Skills

None.

## Work Procedure

1. **Read the feature description** carefully. Understand preconditions, expected behavior, and verification steps.

2. **Read `.factory/library/environment.md`** for deployment constraints (1 vCPU, 512Mi Cloud Run). All changes MUST work within these limits.

3. **Read `.factory/library/architecture.md`** for current backend data layer patterns.

4. **Investigate existing code** before changing anything. Read the files you will modify and their callers/callees. Understand the full call chain.

5. **Write tests (RED where practical)**:
   - Add test cases in the appropriate test module (inline `#[cfg(test)]` module or separate test file)
   - Tests should verify the specific behavior described in the feature's `expectedBehavior`
   - For pure logic (cache eviction, header mapping, data transforms): write failing tests first, then implement (strict RED-GREEN)
   - For service-dependent/integration code (S3 operations, full handler flows): write tests alongside implementation when deterministic failing-first tests are impractical, but still ensure comprehensive test coverage
   - For parallelization: test that results are correct and complete (order-independent)
   - For cache: test that granular eviction works (mock the Store trait if needed)
   - For headers: test that Cache-Control headers are set on responses
   - Run `cargo test` and confirm tests pass

6. **Implement the changes (GREEN)**:
   - Make the minimal changes needed to pass the tests
   - For parallelization: use `tokio::task::JoinSet` with `tokio::sync::Semaphore` for bounded concurrency
   - Semaphore default should be 8-16 (suitable for 1 vCPU / 512Mi)
   - For cache: replace `read_cache.clear()` with targeted eviction methods
   - Preserve all existing API contracts (request/response shapes)
   - Run `cargo test` and confirm ALL tests pass (new and existing)

7. **Run full validation**:
   - `cargo check` (type checking)
   - `cargo test` (all tests)
   - `cargo build` (verify it compiles for release)

8. **Manual verification** (if the feature involves API-observable changes like headers):
   - Start the backend: `cd backend && cargo run`
   - Use `curl -v` to verify headers or response behavior
   - Stop the backend after verification

## Example Handoff

```json
{
  "salientSummary": "Parallelized load_all with JoinSet + Semaphore(12) bounded concurrency. Added 4 tests covering parallel fetch correctness, semaphore bounding, empty prefix, and error propagation. cargo test passes (23 tests total).",
  "whatWasImplemented": "Replaced sequential loop in db/helpers.rs load_all with tokio::task::JoinSet spawning concurrent S3 GetObject calls, bounded by Semaphore(12). Added configurable MAX_CONCURRENT_S3_OPS constant. Updated bulk_insert_videos to use the same pattern for PutObject calls.",
  "whatWasLeftUndone": "",
  "verification": {
    "commandsRun": [
      {"command": "cargo test", "exitCode": 0, "observation": "23 tests passed, 0 failed. New tests: test_load_all_parallel, test_load_all_bounded_concurrency, test_load_all_empty, test_load_all_error_propagation"},
      {"command": "cargo check", "exitCode": 0, "observation": "No warnings or errors"},
      {"command": "cargo build", "exitCode": 0, "observation": "Compiles successfully"}
    ],
    "interactiveChecks": []
  },
  "tests": {
    "added": [
      {"file": "backend/src/db/helpers.rs", "cases": [
        {"name": "test_load_all_parallel", "verifies": "load_all returns all objects correctly when fetched in parallel"},
        {"name": "test_load_all_bounded_concurrency", "verifies": "Semaphore limits concurrent S3 operations"},
        {"name": "test_load_all_empty", "verifies": "load_all with empty prefix returns empty vec"},
        {"name": "test_load_all_error_propagation", "verifies": "S3 errors propagate correctly from parallel tasks"}
      ]}
    ]
  },
  "discoveredIssues": []
}
```

## When to Return to Orchestrator

- Feature depends on a frontend change that hasn't been made yet
- S3 SDK behavior is unexpected (e.g., rate limiting, credential issues)
- Existing tests fail for reasons unrelated to this feature
- The read cache architecture needs a fundamental redesign beyond targeted eviction
- Memory usage concerns that need architectural discussion
