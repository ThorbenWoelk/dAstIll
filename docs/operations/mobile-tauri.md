# Android Operations

This runbook covers local Android tooling, launch commands, APK builds, CI artifacts, and smoke
checks for the Tauri Android shell.

## Tooling

Install the Tauri CLI once:

```bash
cargo install tauri-cli --version "^2"
```

If the binary is unavailable, use the package runner:

```bash
bunx @tauri-apps/cli@latest android dev
```

Install local Android tooling:

- Android Studio
- Java 17+
- Android SDK
- Android NDK
- Rust Android targets

Rust targets:

```bash
rustup target add aarch64-linux-android armv7-linux-androideabi \
  i686-linux-android x86_64-linux-android
```

Set `JAVA_HOME`, `ANDROID_HOME`, and `NDK_HOME` in your shell profile or local environment.

Check device or emulator visibility:

```bash
adb devices
```

## Local Run

From the repo root, start the local stack:

```bash
./start_app.sh
```

The Android shell is opt-in. Set the mobile launch flag in your shell before running
`./start_app.sh` when you want the script to launch Android after local services are healthy.
Launch flag names live in `backend/.env.example`.

Manual Android launch:

```bash
cargo tauri android dev
```

Keep Android-facing frontend build values in the shared/local frontend env files. Do not add them to
`start_app.sh`.

## Local Connectivity

`./start_app.sh` configures local port forwarding for the Android shell. The frontend has a local
Android fallback origin for `http://tauri.localhost`.

When browser-based sign-in needs a different origin than the Tauri-loaded frontend, set the
browser-auth origin in the frontend env. Use `frontend/.env.example` for the current key name.

## Smoke Check

Verify these after local launch or APK install:

1. The app launches without a blank screen.
2. Anonymous mode works on first load.
3. Workspace data loads from the backend.
4. A content item opens.
5. Sign-in completes.
6. Queue, highlights, chat, and workspace navigation load.

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

The Android workflow is [`.github/workflows/android.yml`](../../.github/workflows/android.yml).

It:

- resolves deployed backend and docs URLs
- resolves Firebase frontend build values from Secret Manager
- builds the Android app
- uploads the release APK as a workflow artifact

The workflow needs Android signing secrets configured in GitHub.
