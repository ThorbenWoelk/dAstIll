# Tasks: Follow Channels - Enso Web App

## Current State
MVP complete. Web app functional with River view (video feed) and Sanctuary view (transcript reader/editor). All 180 tests passing. Styling with Zen-inspired Enso design working.

## Steps

### Phase 1: Project Setup
- [x] Set up Svelte 5 frontend project with Vite in `frontend/` directory
- [x] Configure Tailwind CSS 4 with Enso color palette (muted red, off-white, charcoal)
- [x] Set up typography (Merriweather for body, Inter for UI)
- [x] Create Python FastAPI backend in `src/api.py` to expose existing functionality as REST API
- [x] Update `start_app.sh` to run both frontend and backend

### Phase 2: Backend API
- [x] Create FastAPI app with CORS configuration
- [x] Implement `/api/channels` endpoints (list, add, remove, subscribe)
- [x] Implement `/api/videos` endpoints (list by channel, get transcript, get summary)
- [x] Implement `/api/transcripts/{video_id}` endpoint (get/update transcript)
- [x] Implement `/api/summaries/{video_id}` endpoint (get/update summary, trigger generation)
- [ ] Add WebSocket endpoint for real-time status updates (rate limits, new videos)

### Phase 3: Frontend - River View (Home Feed)
- [x] Create main layout with minimal navigation header
- [x] Implement channel subscription input (handle/URL/channel ID)
- [x] Create VideoCard component with channel name, status badge, video ID
- [x] Implement video feed with 3-column grid (newest to oldest)
- [x] Add rate limit banner with calm messaging
- [x] Style with Enso design principles (muted colors, clean typography, card shadows)
- [ ] Implement infinite scrolling with lazy loading

### Phase 4: Frontend - Sanctuary View (Reader/Editor)
- [x] Create reader layout with back navigation and title
- [x] Implement Transcript/Summary toggle (pill-shaped switch with muted red highlight)
- [x] Create content display area with optimal readability (750px max-width, Merriweather font)
- [x] Add hover-to-edit functionality with pencil icon
- [x] Implement inline editor with save functionality
- [x] Add loading animation for pending AI summaries
- [x] Implement keyboard shortcuts (Tab to toggle, Esc to go back, Cmd+S to save)

### Phase 5: AI Integration
- [x] Connect Ollama integration to summary endpoint
- [ ] Implement summary generation queue with status tracking
- [ ] Add real-time progress updates via WebSocket
- [x] Handle summary generation failures gracefully

### Phase 6: Polish & Cleanup (Future Work)
- [ ] Update README.md with new web interface documentation
- [ ] Add proper markdown rendering (currently showing raw markdown)
- [ ] Ensure responsive design for mobile
- [ ] Add loading states and error handling polish
- [ ] Clean up old CLI-only code

## Decisions Made During Implementation
- Kept Python backend (FastAPI) instead of Rust - existing Python services work well and are fully tested (180 tests)
- API placed in `src/api.py` rather than separate `backend/` folder for simpler structure
- Used Tailwind CSS 4 with @tailwindcss/vite plugin for styling
- Transcript editing updates the full markdown file (not just transcript section)
- Summary extraction parses `## Summary` section from transcript markdown files
- Color palette: cream (#FAF9F6), muted red (#B85C4C), charcoal (#333333)

## What's Working
- River view showing 98+ videos from 5 channels
- Video cards with channel names, status badges (Ready/Downloaded)
- Channel subscription via handle input
- Sanctuary view with transcript/summary toggle
- Keyboard shortcuts for navigation
- Full transcript content display
- Summary display when available
- Edit functionality (hover to reveal edit button)

## What Needs Polish
- Markdown rendering (currently raw text)
- Infinite scroll pagination
- WebSocket for real-time updates
- Mobile responsive design
