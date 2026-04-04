# Tauri Android Native Google Auth With App Callback

## Problem

The current Android login path is not shippable.

- Google blocks OAuth inside the embedded Android WebView with `403 disallowed_useragent`.
- A browser-only workaround is not enough because users need to return to the app automatically after authentication.
- The current mobile shell must support authenticated use, not just guest mode.

The core issue is that Android auth is still modeled like a web flow. For the Tauri Android shell, sign-in needs to be treated as a native Android capability with an explicit callback back into the running app.

## Goal

Implement Android-native Google sign-in for the Tauri mobile shell so that:

- the app does not use embedded WebView Google auth
- the app does not strand the user in the browser
- sign-in completes and returns control to the app automatically
- the shell ends in the same authenticated Firebase state as the web app
- the existing backend bearer-token model keeps working unchanged after sign-in

## Requirements

- Android sign-in must use a native Android-safe Google auth mechanism rather than Firebase web popup auth inside the WebView.
- The auth flow must return to the app via a real callback path after the user finishes Google sign-in.
- The callback must resume the Tauri Android app even if the app was backgrounded during sign-in.
- The frontend must end in an authenticated Firebase session after the callback completes.
- The backend must continue to receive Firebase bearer tokens through the existing request path.
- The Android shell must expose clear success, cancellation, and failure states to the frontend.
- Logout must clear both Firebase auth state and any Android-native credential session state relevant to repeat sign-in.
- The implementation must work in local emulator development using the existing `./start_app.sh` workflow.
- The implementation must define the required Android/Firebase configuration inputs explicitly, including any Google client IDs, SHA fingerprints, deep-link or app-link requirements, and local-development assumptions.
- The implementation must preserve guest mode for users who do not sign in.
- The implementation must not require a browser-only manual “navigate back to the app” step.

## Non-Goals

- Reworking desktop or browser auth.
- Replacing Firebase auth with another auth provider.
- Solving Play Store release, app signing distribution, or store metadata in the same pass.
- Reworking transcript selection or other mobile-shell UX outside auth.
- Building iOS native auth in this scope.

## Design Considerations

- Prefer Android-native Google auth plus an explicit app callback over another browser workaround. The browser workaround is already proving fragile.
- The callback path must be deterministic. A deep link, app link, or equivalent native callback is acceptable, but it must be owned as part of the auth design, not left as an external browser behavior.
- The final app-auth state should remain Firebase-based so the backend bearer-token model and frontend auth-dependent logic do not need a second auth system.
- The native bridge should expose a narrow contract to the frontend: start auth, receive completion or error, and optionally clear native state on logout.
- Local development should remain one-command oriented. If emulator/browser callback routing needs `adb reverse`, deep-link config, or explicit hostnames, that must be documented and automated where possible.
- Android configuration must be treated as first-class repo state. If native auth needs a web client ID or `google-services.json` equivalent, the required source of truth must be made explicit in docs and env handling.

## Open Questions

- Which native Android sign-in path should be used in this repo: Credential Manager / Sign in with Google, Google Identity Services, or another Firebase-supported Android-native flow?
- Should the callback return directly with a Google credential to the Android plugin and let native code finish Firebase auth, or should native code only return the Google token and let the frontend finish Firebase auth?
- What exact configuration source will carry the required Google web or server client ID for local development and CI?
- Does the final callback use a custom deep-link scheme, Android app links, or a plugin-managed deep-link event path?
