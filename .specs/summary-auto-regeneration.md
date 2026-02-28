# Spec: Automatic Summary Regeneration for Low Quality

## Problem
Summary quality is now evaluated asynchronously, but low-scoring summaries are only labeled and not automatically improved.

## Goals
- Automatically re-queue summary regeneration when evaluated quality is below `7/10`.
- Use existing workers to orchestrate the loop: generate -> evaluate -> optionally regenerate.
- Prevent infinite regeneration loops with a bounded retry policy.

## Non-Goals
- Manual user controls for retry policy.
- Historical storage of every generation/evaluation revision.

## Backend Requirements
- Persist an auto-regeneration attempt counter per summary.
- Queue regeneration when score `< 7` and attempts remain.
- Bypass summary cache when regeneration is requested by orchestration.
- Stop auto-regeneration once the configured attempt limit is reached.
- Keep summary availability intact even when regeneration/evaluation fails.

## Verification
- Add backend tests for retry counter behavior and regeneration decision logic.
- Run format, clippy (`-D warnings`), tests, and release build locally.
