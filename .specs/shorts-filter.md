# Spec: Shorts Filtering

## Problem
Channel video lists include YouTube Shorts mixed with long-form videos, which makes transcript/summary workflows noisy for users who only want standard videos.

## Goal
Allow users to hide Shorts from video lists using data persisted on each video record.

## Scope
- Add Shorts classification data to backend `Video` model and SQLite storage.
- Populate Shorts classification during channel refresh using YouTube URL resolution.
- Add API filtering support for including/excluding Shorts.
- Add UI toggle(s) so users can hide Shorts in Workspace and Download Queue pages.

## Out of Scope
- Perfect historical backfill of existing rows.
- Advanced duration-based categorization beyond Shorts vs non-Shorts.

## Approach
- Add `is_short: bool` to the shared `Video` shape.
- Use YouTube redirect/canonical behavior from `https://www.youtube.com/shorts/{id}` to classify Shorts.
- Persist `is_short` in `videos` table with default false and expose it in list/get endpoints.
- Add optional `include_shorts` query parameter to `GET /api/channels/{id}/videos` (default true).
- Add frontend toggle to request `include_shorts=false` when user wants Shorts hidden.

## Risks
- Classification adds one lightweight HTTP request per fetched video during refresh.
- Any YouTube redirect behavior changes could reduce classification accuracy; fallback is non-short.

## Validation
- Backend tests for DB filtering by `include_shorts`.
- Backend tests for Shorts URL classification helper.
- Run backend format/check/tests.
- Run frontend type/build checks.
