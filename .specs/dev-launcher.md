# Spec: dAstIll Dev Launcher (.app)

## Goal
Provide a one-click macOS launcher app for local development that starts dAstIll and opens it in a browser.

## Scope
- Add an installer script that creates `dAstIll Dev.app` in `~/Applications`.
- Launcher starts `start_app.sh` detached and opens the frontend URL.
- Include a custom icon bundled into the `.app`.
- Keep the setup idempotent (safe to re-run installer).

## Out of Scope
- Packaging a self-contained production release app.
- Code signing, notarization, or DMG creation.

## Acceptance Criteria
- Running installer script creates `~/Applications/dAstIll Dev.app`.
- Launching the app starts backend + frontend using repo-local scripts.
- Browser opens `http://localhost:3543`.
- App has a custom icon.
