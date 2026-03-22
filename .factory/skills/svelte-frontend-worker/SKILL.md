---
name: svelte-frontend-worker
description: Implements SvelteKit frontend performance optimizations (SSR, code splitting, optimistic UI, service worker)
---

# SvelteKit Frontend Worker

NOTE: Startup and cleanup are handled by `worker-base`. This skill defines the WORK PROCEDURE.

## When to Use This Skill

Features involving SvelteKit frontend changes: SSR data loading, component extraction, code splitting, optimistic UI patterns, service worker caching, font optimization, cache invalidation, polling consolidation.

## Required Skills

- `agent-browser` — for manual verification of UI behavior, page loads, and visual regression checks. Invoke after implementation to verify user-visible changes.

## Work Procedure

1. **Read the feature description** carefully. Understand preconditions, expected behavior, and verification steps.

2. **Read `.factory/library/environment.md`** for deployment constraints and stack details.

3. **Read `.factory/library/architecture.md`** for current frontend patterns.

4. **Read `.factory/library/user-testing.md`** for testing surface details and known limitations (especially: `networkidle` does NOT work due to SSE connections).

5. **Investigate existing code** before changing anything. Read the files you will modify, their imports, and their consumers. For component extraction: map all state variables, functions, and reactive declarations that need to move.

6. **Write failing tests first (RED)**:
   - Add test cases in `frontend/tests/` following existing patterns (`.test.ts` files)
   - Use `fake-indexeddb` for IDB tests (already in devDependencies)
   - For API/cache changes: test cache invalidation behavior, request deduplication
   - For optimistic UI: test that state updates before promise resolves
   - For backoff: test increasing delay intervals
   - Run `bun test` and confirm new tests FAIL

7. **Implement the changes (GREEN)**:
   - Make the minimal changes needed to pass tests
   - For SSR: add `+page.server.ts` or `+page.ts` load functions, use SvelteKit's data loading
   - For component extraction: move code into `$lib/workspace/` or `$lib/components/` following existing patterns
   - For optimistic UI: update state immediately, fire request, revert on error with error toast
   - For service worker: implement proper caching strategies in `static/sw.js`
   - Preserve all existing functionality — no visual or behavioral regressions
   - Follow the design system in AGENTS.md (no emojis, use CSS custom properties, etc.)
   - Run `bun test` and confirm ALL tests pass

8. **Run full validation**:
   - `bun test` (all unit tests pass)
   - `npm run check` (svelte-check type checking)
   - `npm run format:check` (formatting)
   - `bun run build` (production build succeeds)

9. **Manual verification with agent-browser**:
   - Start the app: `cd /Users/thorben.woelk/repos/dAstIll && ./start_app.sh --detach`
   - Wait for health: `sleep 15` then check `curl -sf http://localhost:3543`
   - Use agent-browser to navigate to affected routes
   - Verify the specific changes (SSR content visible, fonts correct, components render, optimistic updates immediate, etc.)
   - Capture screenshots as evidence
   - Stop the app: `cd /Users/thorben.woelk/repos/dAstIll && ./end_app.sh`
   - IMPORTANT: Use `sleep` or `wait --selector`/`wait --text` instead of `wait --load networkidle`

10. **Check for regressions**: Navigate to all main routes (/, /highlights, /download-queue, /chat) to verify nothing is broken. Check console for JS errors.

## Example Handoff

```json
{
  "salientSummary": "Extracted workspace sidebar into WorkspaceSidebar.svelte ($lib/workspace/), moved 12 state variables and 8 functions. Added dynamic import for GuideTour component. bun test passes (142 tests), svelte-check clean, build produces 5 JS chunks. Verified via agent-browser: sidebar renders with channels, video list loads on selection, search works, mobile layout intact.",
  "whatWasImplemented": "Extracted WorkspaceSidebar.svelte from +page.svelte with channel list, video list, filters, and channel input. Created WorkspaceContentPanel.svelte for content tabs and display. Added lazy() wrapper for GuideTour. Main +page.svelte reduced from 2200 to 800 lines. Build output now produces 5 distinct JS chunks.",
  "whatWasLeftUndone": "",
  "verification": {
    "commandsRun": [
      {"command": "bun test", "exitCode": 0, "observation": "142 tests passed, 0 failed"},
      {"command": "npm run check", "exitCode": 0, "observation": "svelte-check clean, no errors or warnings"},
      {"command": "npm run format:check", "exitCode": 0, "observation": "All files formatted correctly"},
      {"command": "bun run build", "exitCode": 0, "observation": "Build succeeds, 5 JS chunks produced"}
    ],
    "interactiveChecks": [
      {"action": "Navigate to / and verify sidebar renders with channels", "observed": "Sidebar shows 4 channels with thumbnails, input field visible at top"},
      {"action": "Select channel and verify video list loads", "observed": "12 videos load in list, thumbnails and titles visible"},
      {"action": "Switch content tabs (Transcript, Summary, Info)", "observed": "All tabs switch correctly, content updates without errors"},
      {"action": "Check mobile viewport (375px width)", "observed": "Bottom nav visible, sidebar overlay opens on tap, back button works"},
      {"action": "Navigate to /highlights, /download-queue, /chat", "observed": "All routes load correctly, no console errors"}
    ]
  },
  "tests": {
    "added": [
      {"file": "frontend/tests/workspace-sidebar.test.ts", "cases": [
        {"name": "channel list renders from provided data", "verifies": "Sidebar component displays channels correctly"},
        {"name": "video list updates on channel selection", "verifies": "Selecting a channel triggers video list refresh"}
      ]}
    ]
  },
  "discoveredIssues": []
}
```

## When to Return to Orchestrator

- Component extraction requires changing backend API contracts
- SSR data loading hits authentication/proxy issues that need architectural decisions
- Service worker changes conflict with existing PWA registration
- Existing tests fail for reasons unrelated to this feature
- The feature depends on backend changes that haven't been made yet
- Build fails after changes and the cause is unclear
