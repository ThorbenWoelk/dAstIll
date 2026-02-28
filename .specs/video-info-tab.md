# Spec: Video Info Tab Next to Transcript and Summary

## Problem
The workspace only exposes transcript and summary views. Users cannot quickly inspect full video metadata such as full title, description, and related details.

## Goals
- Add an `info` tab next to `transcript` and `summary`.
- Show rich video details in that tab: full title, description, URL, channel info, publish date, and available engagement/runtime metadata.
- Keep transcript/summary editing behavior unchanged.

## Non-Goals
- Editing metadata in the app.
- Persisting a full metadata cache in SQLite.

## Backend Changes
- Add `GET /api/videos/{id}/info` endpoint.
- Add YouTube service method to fetch and parse watch-page metadata.
- Provide resilient fallback fields from DB video row when fetch/parsing is partial.

## Frontend Changes
- Extend content-mode toggle to include `info`.
- Add API client + types for video info payload.
- Render info panel in content area when `info` mode is selected.
- Disable content editor controls in `info` mode.

## Verification
- Backend: format, clippy (warnings denied), tests, release build.
- Frontend: check and build.
