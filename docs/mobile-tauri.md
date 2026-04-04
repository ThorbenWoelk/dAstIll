# Tauri Android

## Purpose

dAstIll includes a Tauri v2 shell for Android under [`src-tauri/`](../src-tauri).

The Android app:

- uses the static frontend build from `frontend/`
- talks directly to the Rust backend
- sends Firebase bearer tokens in `Authorization` headers
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
cargo tauri android dev
```

This assumes the backend is reachable locally and the Android app can call it through the configured `VITE_API_BASE`.

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
