# Tasks: YouTube Channel Subscription App

## Current State
Backend + frontend implemented. Transcript extraction now uses direct `summarize --extract` output (`md` + `txt`) without LLM markdown mode so transcript fetch works without `GEMINI_API_KEY`.

## Steps

### Phase 1: Backend Core
- [x] Create Cargo project with dependencies
- [x] Database module with migrations
- [x] Data models (Channel, Video, Transcript, Summary)
- [x] Channel CRUD handlers
- [x] YouTube RSS service (fetch videos)
- [x] Channel handle resolution

### Phase 2: Transcript Pipeline
- [x] `summarize` CLI wrapper service
- [x] Transcript storage/retrieval handlers
- [x] Lazy loading on GET request

### Phase 3: Summarization
- [x] Ollama HTTP client
- [x] Summary generation service
- [x] Summary storage/retrieval handlers

### Phase 4: Frontend Shell
- [x] SvelteKit + Tailwind setup
- [x] Color palette CSS (zen aesthetic)
- [x] API client layer
- [x] Route structure

### Phase 5: UI Components
- [x] ChannelCard, VideoCard components
- [x] TranscriptView with markdown
- [x] Toggle (transcript/summary)
- [x] ContentEditor (edit mode)

### Phase 6: Integration
- [x] Channel subscription flow
- [x] Channel avatar fallback + subscribe-time canonical avatar fetch
- [x] Video list with lazy load
- [x] Transcript/summary toggle view
- [x] Edit and save functionality
- [x] start_app.sh

## Decisions Made During Implementation
- Backend API uses `/api` routes with Turso (DB_URL/DB_PASS), Ollama via `OLLAMA_URL` + `OLLAMA_MODEL`, and summarize CLI via `SUMMARIZE_PATH`.
- `TranscriptService` now avoids `--markdown-mode llm` for transcript fetches and uses direct extraction (`--extract --format md` plus `--extract --format txt`) to prevent hard dependency on Gemini credentials.
- Frontend uses SvelteKit + Tailwind v4 with `@tailwindcss/vite` and markdown rendering via marked + isomorphic-dompurify.
- YouTube `videos.xml` may be Atom, so the backend now parses RSS first and falls back to Atom XML to avoid channel refresh failures.
- Channel avatar extraction now checks `og:image`, `thumbnailUrl`, and `image_src`, then falls back to embedded yt3 URLs from page data before defaulting in the UI.
