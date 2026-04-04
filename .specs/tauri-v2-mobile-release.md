# Tauri v2 Android — Current State Release

## Summary

This artifact records the current checkpoint of the Tauri Android migration work.

The repo now has:

- SPA-converted frontend structure for Tauri compatibility
- backend bearer-token auth support for direct browser or mobile-shell calls
- generated `src-tauri/` shell and Android Gradle project
- one-command local startup that can launch the Android emulator and shell
- partial Android auth experimentation

## What Works

- `./start_app.sh` starts backend, frontend, docs, and can start the Android shell when the emulator is available.
- The frontend builds with `adapter-static`.
- The backend compiles and tests with the bearer-token auth path.
- The Tauri shell compiles locally.
- Guest mode remains the intended fallback path.

## What Is Not Release-Ready

- Google sign-in in the Android shell is not finished.
- Embedded WebView auth is blocked by Google policy.
- Browser-handoff auth is not a final solution and should be replaced with native Android auth plus callback.
- The original Android native text-selection replacement did not land in a stable supported form and currently falls back to the in-app behavior.

## Risks

- Android auth remains the largest unresolved blocker for a user-facing release.
- Local dev can be confused by stale or duplicate service processes; lifecycle scripts must remain the source of truth.
- Mobile-shell behavior and browser behavior now share more code but still diverge in auth and callback handling.

## Recommended Next Step

Implement the follow-up spec in `.specs/tauri-android-native-google-auth.md` and stop iterating on browser-only auth workarounds.
