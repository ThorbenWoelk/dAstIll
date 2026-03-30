# Tauri v2 Android — Migration Spec for dAstIll

> Created: 2026-03-30

---

## TL;DR

Wrapping dAstIll in Tauri v2 for Android. The single largest change is converting SvelteKit from SSR (`adapter-node`) to SPA (`adapter-static`) — everything else is additive. The Rust backend stays untouched. The custom text-selection action bar (the original goal) is achievable via a small Kotlin plugin.

**Distribution strategy**: sideloading (direct APK install) first — no app store account, no review process, works immediately. Play Store is a later option once YouTube policy compliance allows it.

---

## 1. Architecture: What Changes vs What Stays

### Stays exactly as-is

- Rust backend (`axum`, port 3544) — no changes, runs as a remote service
- All frontend Svelte components, routing, CSS, state
- Firebase auth (Firebase JS SDK works fine inside Android System WebView)
- `localStorage`, `sessionStorage`, `IndexedDB` — all supported in WebView
- Service worker — supported in Android System WebView
- `fetch()`, `EventSource`, `ReadableStream` — all work in WebView
- `navigator.clipboard` — works in WebView
- `<audio>` element (TTS player) — works natively
- Tailwind, all CSS — no changes

### Must change: SSR → SPA

This is the only structural rewrite. Tauri has no Node.js runtime — it serves a static bundle from disk. SvelteKit's `adapter-node` (current setup) is incompatible. Everything that runs on the server side must move.

| What breaks | Impact | Resolution |
|---|---|---|
| `adapter-node` | Build fails | Switch to `adapter-static` |
| `+layout.server.ts` (auth session) | Runtime failure | Move to client-only Firebase SDK flow |
| `+page.server.ts` files | Runtime failure | Audit each; convert to `load()` client functions |
| `src/routes/auth/*` server routes | Not reachable | Firebase client SDK handles auth directly |
| `hooks.server.ts` | Not executed | Move any session logic to client |
| `$env/dynamic/public` | Build-time only in static | Replace with `$env/static/public` or build-time vars |

The frontend already uses the Firebase JS SDK for client-side auth (`auth-state.svelte.ts`). The server routes primarily exchange Firebase ID tokens for session cookies. In Tauri, skip the cookie-based session entirely — keep the Firebase ID token in memory / `localStorage` and send it as `Authorization: Bearer <token>` to the Rust backend, which already validates Firebase tokens server-side.

### Additive: Tauri shell layer

A new `src-tauri/` directory sits alongside the existing `frontend/`:

```
dAstIll/
├── frontend/           ← unchanged SvelteKit source
├── backend/            ← unchanged Rust backend
└── src-tauri/          ← new: Tauri shell
    ├── Cargo.toml
    ├── tauri.conf.json
    ├── capabilities/
    └── gen/
        └── android/    ← generated Gradle project (committed)
```

---

## 2. Toolchain Setup

### One-time developer machine setup

```bash
# Rust (if not already installed)
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Android targets
rustup target add aarch64-linux-android armv7-linux-androideabi \
  i686-linux-android x86_64-linux-android

# Tauri CLI
cargo install tauri-cli --version "^2"

# Android Studio: install from https://developer.android.com/studio
# Then via SDK Manager install:
#   - NDK 28.0.12674087
#   - Build-Tools 34.0.0
#   - Android SDK Platform 34

# Required env vars (add to ~/.zshrc or ~/.bashrc)
export JAVA_HOME="/Applications/Android Studio.app/Contents/jbr/Contents/Home"
export ANDROID_HOME="$HOME/Library/Android/sdk"
export NDK_HOME="$ANDROID_HOME/ndk/28.0.12674087"
```

---

## 3. SvelteKit Conversion (SSR → SPA)

**`frontend/svelte.config.js`** — swap adapter:
```js
import adapter from '@sveltejs/adapter-static';

export default {
  kit: {
    adapter: adapter({ fallback: 'index.html' }),
  }
};
```

**`frontend/src/routes/+layout.ts`** — disable SSR globally:
```ts
export const ssr = false;
```

**`frontend/src/lib/api-client.ts`** — add token injection:
```ts
// getIdToken() from Firebase JS SDK, called before each request
const token = await auth.currentUser?.getIdToken();
headers['Authorization'] = `Bearer ${token}`;
```

The server session flow (`POST /auth/session` → cookie) becomes: keep the Firebase ID token in memory, refresh it when needed, and pass it as `Authorization: Bearer <token>` to the Rust backend. The Rust backend needs to accept this header instead of (or in addition to) the session cookie.

**Env vars** — `$env/dynamic/public` is runtime-server-only. Replace with build-time `$env/static/public` and pass values at build time via a `.env` file baked into the static build.

**Remove or stub**: `frontend/src/routes/auth/+server.ts`, `frontend/src/hooks.server.ts`, all `+*.server.ts` files. Each one needs a quick audit — most are thin auth wrappers that become no-ops in client-only mode.

---

## 4. Tauri Shell Setup

After the SvelteKit conversion, initialise the Tauri shell from the repo root:

```bash
cargo tauri init
# frontendDist: ../frontend/build
# devUrl: http://localhost:5173
# beforeDevCommand: cd frontend && bun run dev
# beforeBuildCommand: cd frontend && bun run build
```

**`src-tauri/tauri.conf.json`** (key sections):
```json
{
  "productName": "dAstIll",
  "version": "1.0.0",
  "identifier": "com.dastill.app",
  "build": {
    "frontendDist": "../frontend/build",
    "devUrl": "http://localhost:5173",
    "beforeDevCommand": "cd frontend && bun run dev",
    "beforeBuildCommand": "cd frontend && bun run build"
  },
  "app": {
    "security": {
      "csp": "default-src 'self'; connect-src 'self' https://*.googleapis.com https://*.firebaseio.com https://your-backend.run.app"
    }
  }
}
```

Initialise the Android project (one-time, commit the generated folder):
```bash
cargo tauri android init
```

---

## 5. Custom Text Selection Plugin (the original goal)

A Kotlin plugin that hooks into the Android WebView to replace the native selection action bar with our own items.

**`src-tauri/gen/android/app/src/main/java/com/dastill/app/MainActivity.kt`**:
```kotlin
class MainActivity : TauriActivity() {
    override fun onCreate(savedInstanceState: Bundle?) {
        registerPlugin(SelectionPlugin::class.java)
        super.onCreate(savedInstanceState)
    }

    override fun onWebViewCreate(webView: WebView) {
        webView.setCustomSelectionActionModeCallback(object : ActionMode.Callback {
            override fun onCreateActionMode(mode: ActionMode, menu: Menu): Boolean {
                menu.add(Menu.NONE, 1, Menu.NONE, "Highlight")
                    .setShowAsAction(MenuItem.SHOW_AS_ACTION_ALWAYS)
                menu.add(Menu.NONE, 2, Menu.NONE, "Correct")
                    .setShowAsAction(MenuItem.SHOW_AS_ACTION_IF_ROOM)
                return true
            }
            override fun onPrepareActionMode(mode: ActionMode, menu: Menu) = false
            override fun onActionItemClicked(mode: ActionMode, item: MenuItem): Boolean {
                return when (item.itemId) {
                    1 -> { webView.evaluateJavascript("window.__tauri_selection_highlight()", null); mode.finish(); true }
                    2 -> { webView.evaluateJavascript("window.__tauri_selection_correct()", null); mode.finish(); true }
                    else -> false
                }
            }
            override fun onDestroyActionMode(mode: ActionMode) {}
        })
        super.onWebViewCreate(webView)
    }
}
```

**Frontend side** — register handlers once at startup:
```ts
// frontend/src/lib/native-selection.ts
export function registerNativeSelectionHandlers(
  onHighlight: (text: string) => void,
  onCorrect: (text: string) => void
) {
  if (typeof window === 'undefined') return;
  (window as any).__tauri_selection_highlight = () => {
    const text = window.getSelection()?.toString() ?? '';
    onHighlight(text);
  };
  (window as any).__tauri_selection_correct = () => {
    const text = window.getSelection()?.toString() ?? '';
    onCorrect(text);
  };
}
```

This completely replaces the floating bottom toolbar on Android with OS-native action bar items. The existing toolbar in `TranscriptView.svelte` remains the fallback for web/desktop — detect Tauri with `'__TAURI_INTERNALS__' in window` to conditionally hide it.

---

## 6. Storage — All Good on Android

Every storage API used by the app works in Android System WebView with no changes needed:

| Storage | Android WebView | Notes |
|---|---|---|
| `localStorage` | ✅ | None |
| `sessionStorage` | ✅ | None |
| `IndexedDB` (workspace cache) | ✅ | None |
| `navigator.clipboard` | ✅ | None |
| Service Worker | ✅ | None |
| `EventSource` / SSE | ✅ | None |
| `ReadableStream` (chat) | ✅ | None |
| `<audio>` (TTS) | ✅ | None |

---

## 7. Build Pipeline

### Development (hot reload on device)
```bash
cargo tauri android dev   # connects to device/emulator with live reload
```

### Sideload APK (primary distribution path)
```bash
# Debug build — no signing required, install immediately
cargo tauri android build -- --apk --debug

# Release build — requires keystore (see §10), better performance
cargo tauri android build -- --apk
```

Output: `src-tauri/gen/android/app/build/outputs/apk/{debug,release}/app-*.apk`

### Play Store AAB (future, once eligible)
```bash
cargo tauri android build -- --aab
```

First build: 10–20 minutes (full Rust cross-compile for 4 Android targets). Subsequent builds with Rust cache: 3–5 minutes.

---

## 8. CI/CD (GitHub Actions)

Android CI runs on `ubuntu-latest` — covered by the free tier, no multiplier.

**`.github/workflows/android.yml`**:
```yaml
jobs:
  android:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: actions/setup-java@v4
        with: { java-version: '17', distribution: 'temurin' }
      - uses: android-actions/setup-android@v3
      - run: sdkmanager "ndk;28.0.12674087" "build-tools;34.0.0"
      - uses: dtolnay/rust-toolchain@stable
        with:
          targets: aarch64-linux-android,armv7-linux-androideabi,i686-linux-android,x86_64-linux-android
      - uses: swatinem/rust-cache@v2
        with: { workspaces: './src-tauri -> target' }
      - uses: oven-sh/setup-bun@v2
      - run: bun install --frozen-lockfile
        working-directory: frontend
      - name: Write signing config
        run: |
          cd src-tauri/gen/android
          echo "keyAlias=${{ secrets.ANDROID_KEY_ALIAS }}" > keystore.properties
          echo "password=${{ secrets.ANDROID_KEY_PASSWORD }}" >> keystore.properties
          base64 -d <<< "${{ secrets.ANDROID_KEYSTORE_B64 }}" > $RUNNER_TEMP/keystore.jks
          echo "storeFile=$RUNNER_TEMP/keystore.jks" >> keystore.properties
      - run: cargo tauri android build -- --apk
        env:
          NDK_HOME: ${{ env.ANDROID_SDK_ROOT }}/ndk/28.0.12674087
      - uses: actions/upload-artifact@v4
        with:
          name: android-apk
          path: src-tauri/gen/android/app/build/outputs/apk/release/*.apk
```

> Note: `tauri-action@v0` mobile support is experimental and doesn't set up the SDK/NDK. The manual workflow above is the reliable path.

### New GitHub secrets needed

| Secret | Value |
|---|---|
| `ANDROID_KEY_ALIAS` | Keystore alias |
| `ANDROID_KEY_PASSWORD` | Keystore password |
| `ANDROID_KEYSTORE_B64` | `base64 -i dastill.jks` |

---

## 9. Costs

### Sideload-only (current path)

| Item | Cost | Notes |
|---|---|---|
| Tauri itself | Free | Open source (Apache 2 / MIT) |
| Android signing keystore | Free | Self-generated with `keytool` |
| GitHub Actions Android CI | ~$0 | `ubuntu-latest`; free tier covers it |
| **Total** | **$0** | |

### When Play Store becomes an option

| Item | Cost | Notes |
|---|---|---|
| Google Play Store | $25 one-time | Per developer account |

---

## 10. Deployment to Devices

### Step 1 — Enable unknown sources on your device
Settings → Apps → Special app access → Install unknown apps → allow your file manager or browser.

### Step 2 — Build the APK
```bash
# Debug: no signing setup needed, install immediately
cargo tauri android build -- --apk --debug

# Release (better perf): requires keystore first (one-time, see below)
cargo tauri android build -- --apk
```

### Step 3 — Install on device (pick any method)

| Method | How |
|---|---|
| ADB over USB | `adb install app-debug.apk` |
| ADB over Wi-Fi | `adb connect <device-ip>:5555` then same |
| File transfer | Copy APK via USB/cloud drive, tap to install on device |
| Local server | `python3 -m http.server` in build folder, open URL on device and tap download |

### Keystore setup (one-time, needed for release APK)
```bash
keytool -genkey -v -keystore ~/dastill.jks -keyalg RSA \
  -keysize 2048 -validity 10000 -alias dastill
```
Then reference it in `src-tauri/gen/android/app/build.gradle.kts` signing config.

**Updating the app** — build a new APK and install over the existing one. Android keeps app data between installs as long as the signing key stays the same.

---

## 11. What Needs to be Written / Rewritten

### Net new
- `src-tauri/` — Tauri shell config, `Cargo.toml`, capabilities
- `MainActivity.kt` — ActionMode plugin (30–50 lines of Kotlin)
- `frontend/src/lib/native-selection.ts` — JS bridge handlers
- `.github/workflows/android.yml` — CI workflow (produces signed APK artifact)
- Android signing keystore (one-time local setup)
- Auth refactor: remove server session, use Bearer token

### Modified (not rewritten)
- `frontend/svelte.config.js` — swap adapter (2 lines)
- `frontend/src/routes/+layout.ts` — add `ssr = false` (1 line)
- `frontend/src/lib/api-client.ts` — inject Bearer token (~15 lines)
- `frontend/src/lib/components/TranscriptView.svelte` — hide bottom toolbar when running in Tauri (`'__TAURI_INTERNALS__' in window`)

### Deleted
- `frontend/src/hooks.server.ts`
- `frontend/src/routes/auth/+server.ts`
- `frontend/src/routes/+layout.server.ts` (or gutted to client-only)
- `frontend/src/routes/+page.server.ts` files (audit each)

### Unchanged
- Every Svelte component, store, style, and utility
- The entire Rust backend
- Firebase configuration
- Terraform / GCP infrastructure
- Existing web deployment (Tauri app and PWA coexist — same codebase, different build targets)

---

## 12. Risks and Caveats

| Risk | Severity | Mitigation |
|---|---|---|
| SSR → SPA conversion touches auth flow | High | Must be tested thoroughly; session handling changes meaningfully |
| Android WebView version variance on old devices | Medium | Set `minSdkVersion 26` (Android 8+); affects ~3% of devices |
| `tauri-action` mobile support is experimental | Low | Use the manual workflow; it's 50 lines of YAML |
| Web PWA and Tauri app must stay in sync | Ongoing | Same frontend codebase; single source of truth |

---

## 13. Phasing

### Phase 1 — Working on your own device (1–2 weeks)
1. Convert SvelteKit to SPA mode + auth refactor
2. Set up Tauri shell, local Android build
3. Write ActionMode Kotlin plugin
4. Generate signing keystore
5. Build release APK, install via ADB or file transfer

### Phase 2 — CI-produced APK artifact (1 week)
1. Android CI workflow — produces a signed APK on every push
2. Download artifact from GitHub Actions and sideload without a local build environment
3. Useful for sharing with other devices without needing the full toolchain locally

### Phase 3 — Play Store (when YouTube policy allows)
1. Google Play developer account ($25 one-time)
2. Switch CI output from APK to AAB
3. Upload to Play Console → internal testing → staged rollout
