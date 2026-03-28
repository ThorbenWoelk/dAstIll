---
name: fullstack-auth-worker
description: Implements features spanning SvelteKit frontend and Rust Axum backend for the multi-user Firebase Auth migration.
---

# Fullstack Auth Worker

NOTE: Startup and cleanup are handled by `worker-base`. This skill defines the WORK PROCEDURE.

## When to Use This Skill

Features that span both the SvelteKit frontend and Rust Axum backend, including:
- Firebase Auth session management
- SvelteKit hooks and server routes
- Backend middleware and endpoint refactoring
- Frontend auth context and UI components
- Per-user data storage and endpoint scoping
- Client-side storage namespacing

## Required Skills

- `agent-browser` — for manual verification of auth flows and UI state changes. Invoke when verifying login/logout flows, anonymous vs authenticated UI, or any browser-visible behavior.

## Work Procedure

1. **Read feature context**: Read the feature description, preconditions, expectedBehavior, and fulfills from features.json. Read relevant validation contract assertions. Read `.factory/library/architecture.md` for system understanding.

2. **Read the spec**: Read `.specs/multi-user-workspace-with-firebase-auth-and-google-ready-sso.md` for the full specification of the multi-user migration.

3. **Read DESIGN.md** (for frontend work): Read `/Users/thorben.woelk/repos/dAstIll/DESIGN.md` for frontend engineering standards, design system tokens, file size limits (800 max, 500+ refactor candidate), Svelte 5 rune patterns, and clean code rules.

4. **Write failing tests first (RED)**:
   - Frontend tests: Add test file(s) under `frontend/tests/` using bun test. Write tests that assert the expected behavior from the feature description. Run `cd frontend && bun test tests` — tests should FAIL (red).
   - Backend tests: If the feature touches Rust code, add tests in the appropriate `backend/src/` module or `backend/tests/`. Run `cd backend && cargo test` — new tests should FAIL (red).

5. **Implement to make tests pass (GREEN)**:
   - Write the minimum code to make all new tests pass.
   - For Rust model changes: run `cd backend && cargo test` to regenerate ts-rs TypeScript bindings.
   - Follow existing patterns: handlers in `backend/src/handlers/`, DB in `backend/src/db/`, models in `backend/src/models.rs`.
   - For SvelteKit: use Svelte 5 runes (.svelte.ts), follow DESIGN.md conventions.

6. **Run all validators**:
   ```
   cd frontend && bun test tests
   cd backend && cargo test
   cd frontend && bunx svelte-check --tsconfig ./tsconfig.json
   cd frontend && bunx eslint .
   ```
   All must pass with zero errors.

7. **Manual verification**:
   - For API features: use curl to test endpoints with appropriate session cookies/headers.
   - For UI features: use `agent-browser` to verify visual state (invoke the `agent-browser` skill).
   - For auth features: start the Firebase Auth Emulator (`cd frontend && firebase emulators:start --only auth --project demo-dastill`), then test sign-in/sign-out flows.
   - Each manual check = one `interactiveChecks` entry with the full action and observed result.

8. **Commit**: Stage and commit changes with conventional commit format: `feat(scope): description`.

## Example Handoff

```json
{
  "salientSummary": "Implemented GET/POST/DELETE /auth/session with Firebase Admin session cookies. POST exchanges ID tokens for httpOnly cookies (7-day, SameSite Strict). GET returns {userId, authState, accessRole, email}. All 376 frontend tests pass, 318 backend tests pass, svelte-check and eslint clean. Verified via curl that invalid tokens get 401, valid tokens get cookie, DELETE clears cookie.",
  "whatWasImplemented": "Added frontend/src/routes/auth/session/+server.ts with GET/POST/DELETE handlers. POST calls createSessionCookie() with 5-min auth_time freshness check. GET calls verifySessionCookie() and returns auth context. DELETE revokes session and clears cookie. Updated hooks.server.ts to populate event.locals.auth from Firebase session cookie. Added OPERATOR_EMAIL_ALLOWLIST case-insensitive matching. Old HMAC cookies rejected.",
  "whatWasLeftUndone": "",
  "verification": {
    "commandsRun": [
      { "command": "cd frontend && bun test tests", "exitCode": 0, "observation": "376 tests passed across 48 files including 7 new auth tests" },
      { "command": "cd backend && cargo test", "exitCode": 0, "observation": "318 passed, 7 ignored" },
      { "command": "cd frontend && bunx svelte-check --tsconfig ./tsconfig.json", "exitCode": 0, "observation": "0 errors, 0 warnings" },
      { "command": "cd frontend && bunx eslint .", "exitCode": 0, "observation": "No issues" },
      { "command": "curl -X POST http://localhost:3543/auth/session -d '{\"idToken\":\"invalid\"}'", "exitCode": 0, "observation": "401 Unauthorized returned with JSON error body" }
    ],
    "interactiveChecks": [
      { "action": "agent-browser open http://localhost:3543/login && screenshot", "observed": "Login page shows Google sign-in button, no password field present" },
      { "action": "curl -b '__session=valid-cookie' http://localhost:3543/auth/session", "observed": "Returns {userId: 'abc', authState: 'authenticated', accessRole: 'user', email: 'test@example.com'}" }
    ]
  },
  "tests": {
    "added": [
      {
        "file": "frontend/tests/auth-session-route.test.ts",
        "cases": [
          { "name": "POST rejects invalid token with 401", "verifies": "Invalid Firebase ID tokens are rejected" },
          { "name": "POST creates session cookie from valid token", "verifies": "Valid tokens exchange for httpOnly session cookie" },
          { "name": "GET returns auth context from session", "verifies": "Session cookie resolves to correct auth state" },
          { "name": "DELETE clears session cookie", "verifies": "Logout clears the session" }
        ]
      }
    ]
  },
  "discoveredIssues": []
}
```

## When to Return to Orchestrator

- Feature depends on an API endpoint or data model that doesn't exist yet
- Requirements are ambiguous or contradictory
- A precondition feature is not actually complete (missing expected code/APIs)
- Backend or frontend tests fail in ways unrelated to this feature
- Firebase Auth Emulator is not accessible
- Cannot resolve how to split canonical vs per-user data for a specific endpoint
