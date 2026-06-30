# ADR: dastill-mini uses object-store-backed user video state for read tracking

## Status

Accepted

## Context

The main workspace currently resolves read-state operations through the canonical video lookup path, which depends on the SQL-backed video catalog. `dastill-mini` is intended to remain useful when the app is in maintenance mode and the main workspace stack is unavailable or intentionally paused.

## Decision

`dastill-mini` reads and writes summary read status through the existing object-store-backed per-user video state store instead of relying on a SQL-backed acknowledged flag as the source of truth.

## Consequences

- Read status remains available when the mini reader is operating without the workspace bootstrap path.
- The mini reader can use the same per-user state surface across maintenance and normal app modes.
- The SQL-backed acknowledged field becomes, at most, a compatibility surface rather than the canonical write dependency for read state.
