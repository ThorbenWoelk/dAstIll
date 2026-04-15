# Tauri Android

## Purpose

dAstIll includes a Tauri v2 shell for Android under `src-tauri/`.

The Android app:

- uses the static frontend build from `frontend/`
- talks directly to the Rust backend
- sends Firebase bearer tokens in `Authorization` headers
- uses a mobile-auth handoff so Google sign-in can complete in the system browser instead of the embedded Android WebView
- replaces the custom transcript selection create-toolbar with Android native `Highlight` and `Correct` actions

## Install The CLI

Install the Tauri CLI once:

```bash
cargo install tauri-cli --version "^2"
```

If the command is missing, you can always use:

```bash
bunx @tauri-apps/cli@latest <command>
```

Examples:

```bash
bunx @tauri-apps/cli@latest dev
bunx @tauri-apps/cli@latest android dev
```

## Local Tooling

You need:

- Android Studio
- Java 17+
- Android SDK
- Android NDK
- Rust Android targets

Typical setup:

```bash
rustup target add aarch64-linux-android armv7-linux-androideabi \
  i686-linux-android x86_64-linux-android

export JAVA_HOME="/Applications/Android Studio.app/Contents/jbr/Contents/Home"
export ANDROID_HOME="$HOME/Library/Android/sdk"
export NDK_HOME="$ANDROID_HOME/ndk/28.2.13676358"
```

Check that a device or emulator is available:

```bash
adb devices
```

## Development Flow

From the repo root:

```bash
./start_app.sh
```

The Tauri Android shell is opt-in. To have `./start_app.sh` launch it after the local services are healthy:

```bash
START_APP_MOBILE=1 ./start_app.sh
```

If you want to run it yourself instead, use:

```bash
cargo tauri android dev
```

When `.github/runtime-mode.env` is set to `APP_RUNTIME_MODE=maintenance`, `./start_app.sh` still starts the backend and serves the maintenance/minimal frontend mode so `dastill-mini` remains usable locally.

This assumes the backend is reachable locally and the Android app can call it through the configured `VITE_API_BASE`.

For local Android development, keep frontend build values in the shared/local frontend env files rather than in `start_app.sh`.

The frontend also has a Tauri Android dev fallback for `http://tauri.localhost`: when `VITE_API_BASE` is unset there, it uses `http://127.0.0.1:3544`, which matches the `adb reverse` port forwarding set up by `./start_app.sh`.

## Auth Handoff

Google blocks sign-in inside the Android WebView used by Tauri. The current app handles that by:

1. creating a short-lived `/api/auth/mobile-handoff` session on the backend
2. opening `/login?mobileBrowserAuth=1&handoffSession=<id>` in the system browser with a completion secret in the URL fragment
3. completing Google sign-in in the browser
4. posting the Google tokens back to the handoff session with the completion secret
5. redeeming the completed handoff once from the Android shell with a separate redeem secret, then finishing Firebase sign-in locally

The browser no longer polls or fetches reusable Google tokens over `GET`. The redeem step is one-shot and bound to the creator of the handoff.

If your browser-hosted login page lives on a different origin than the Tauri-loaded frontend, set `PUBLIC_BROWSER_AUTH_BASE_URL` (or `VITE_BROWSER_AUTH_BASE_URL`) in the frontend env so the mobile shell opens the correct browser origin for the handoff.

## Smoke Checklist

Verify these in order:

1. The app launches without a blank screen.
2. Anonymous mode works on first load.
3. Workspace data loads from the backend.
4. A transcript opens successfully.
5. Text selection inside the transcript shows native Android actions `Highlight` and `Correct`.
6. Tapping `Highlight` creates a highlight.
7. Tapping `Correct` opens the vocabulary correction flow.
8. Tapping an existing highlight still exposes delete behavior.
9. Google sign-in works.
10. Queue, highlights, chat, and workspace navigation still work.

## Build APKs

Debug APK:

```bash
cargo tauri android build -- --apk --debug
```

Release APK:

```bash
cargo tauri android build -- --apk
```

APK output:

```text
src-tauri/gen/android/app/build/outputs/apk/
```

Install a debug APK manually:

```bash
adb install -r src-tauri/gen/android/app/build/outputs/apk/debug/app-debug.apk
```

## CI

The repository includes [`.github/workflows/android.yml`](../.github/workflows/android.yml).

It:

- resolves the deployed backend/docs URLs
- resolves Firebase frontend build values from Secret Manager
- builds the Android app
- uploads the release APK as a workflow artifact

Required GitHub secrets:

- `ANDROID_KEY_ALIAS`
- `ANDROID_KEY_PASSWORD`
- `ANDROID_KEYSTORE_B64`
