# Tasks: Tauri Android Native Google Auth With App Callback

## Current State

The Android shell launches and can run in guest mode. Google sign-in inside the embedded WebView is blocked by Google policy, and the partial browser handoff work is not a release-quality solution. The remaining work is to replace that with native Android auth plus an app callback.

## Steps

- [ ] Choose and document the Android-native Google auth mechanism to use.
- [ ] Define the callback mechanism that returns control to the app automatically after sign-in.
- [ ] Add the Android-native auth dependencies and plugin bridge in `src-tauri/gen/android/`.
- [ ] Add the frontend-to-native auth bridge contract for Tauri Android.
- [ ] Define and wire the required Google client ID or equivalent Firebase-native auth configuration input.
- [ ] Implement sign-in success, cancellation, and error propagation from native Android into the frontend.
- [ ] Finish Firebase auth in the app and confirm the existing backend bearer-token path works unchanged afterward.
- [ ] Implement native-aware logout cleanup.
- [ ] Verify local emulator flow end to end under `./start_app.sh`.
- [ ] Update README and docs with the final Android auth workflow and required setup.

## Decisions Made During Implementation

- The remaining Android auth work must use a real callback back into the app, not a browser-only dead end.
- The final mobile auth flow must preserve the existing Firebase-based app and backend auth model.
- Guest mode remains supported and is not blocked on native Google auth completion.
