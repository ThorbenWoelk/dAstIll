# Turso Search Index

## Problem

Keyword search currently depends on a local in-memory/full-local index model that is awkward to persist and rehydrate across Cloud Run restarts. We want a durable search store without continuing the S3 snapshot path that was previously being explored.

## Goal

Back the keyword search index with Turso via the Rust `libsql` SDK, using an embedded replica local file for fast reads and a Turso primary database for durable writes, while preserving the existing `FtsIndex` service boundary and current search behavior.

## Requirements

- Keyword search must continue to support ranked full-text retrieval over chunk text, video title, and optional section title.
- The backend must connect to Turso using official Rust `libsql` integration and environment-provided database URL/token.
- Production/runtime reads should use a local embedded replica file, while writes persist to the Turso primary database.
- The app must still be able to bootstrap/rebuild the search index from existing `search-bundles/` or `search-chunks/` data when the Turso-backed index is empty.
- Search projection resets must clear the Turso-backed keyword index as well as the existing derived S3 projection.
- Local development and tests must remain runnable without requiring a hosted Turso database.

## Non-Goals

- Changing semantic search storage or retrieval.
- Replacing the `search_chunks` / `search-bundles` projection, which remains the rebuild source of truth.
- Redesigning ranking beyond what is needed to express current keyword search in SQLite/libSQL FTS.
- Building a new deployment flow for Turso beyond the minimum config and runtime wiring needed here.

## Design Considerations

- Turso’s Rust SDK supports embedded replicas, which keeps local read latency low while persisting writes remotely.
- Keeping the `FtsIndex` interface stable contains the engine swap to one service plus startup/config wiring.
- The bootstrap path from existing projection data remains important for migration and disaster recovery, even after Turso becomes the durable keyword index.

## Open Questions

- Whether the app should trigger explicit `sync()` calls at selected lifecycle points in addition to a periodic replica sync interval.
- Whether a later follow-up should move the canonical search projection itself into Turso rather than only the keyword index.
