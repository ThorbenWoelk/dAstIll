# Spec: Firestore Cost Reduction

## Goal
Reduce Firestore billing by minimizing storage/index overhead and optimizing read operations.

## Context
The application is already highly optimized by using S3 for most of the large datasets. However, `dastill_videos` and `dastill_preferences` are still in Firestore and could benefit from further optimization.

## Proposed Changes

### 1. Index Exemptions (Terraform)
Firestore's default behavior of indexing every field leads to unnecessary storage and write costs. We will exempt the following fields:

- **`dastill_videos`**:
  - `thumbnail_url` (already exempted)
  - `quality_score` (already exempted)
  - `retry_count` (already exempted)
  - `title` (already exempted)
  - `channel_id` (Should we exempt this? It's used in composite indexes but also in single-field filters.)
  
- **`dastill_preferences`**:
  - `vocabulary_replacements`
  - `channel_order`
  
- **`dastill_tts_stats`**:
  - `sample_count`
  - `total_words`
  - `total_duration_secs`

### 2. Read Batching (Rust)
In `search_vector_candidates`, metadata for unique video IDs is fetched one-by-one. We will switch to a batched fetch (`by_id_in()`).

### 3. Review logic
Review `fs_insert_video` and other write-heavy areas to ensure we are not doing redundant reads or writes.

## Tasks
- [ ] Implement index exemptions in `terraform/firestore.tf`
- [ ] Implement batched metadata fetching in `backend/src/db/search.rs`
- [ ] Run `terraform plan` to verify index removals
- [ ] Run `cargo test` to ensure stability
