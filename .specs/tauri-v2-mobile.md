# Tauri v2 Mobile — Migration Spec for dAstIll

> Created: 2026-03-30

---

## TL;DR

Wrapping dAstIll in Tauri v2 is viable for Android today, rough for iOS. The single largest change is converting SvelteKit from SSR (`adapter-node`) to SPA (`adapter-static`) — everything else is additive. The Rust backend stays untouched and runs as a sidecar or remote service. The custom text-selection action bar (the original goal) is achievable via a small Kotlin plugin.

**Distribution strategy**: sideloading (direct APK install) first — no app store account, no review process, works immediately. Play Store is a later option once YouTube policy compliance allows it.

---

## 1. Architecture: What Changes vs What Stays

### Stays exactly as-is

- Rust backend (`axum`, port 3544) — no changes, runs as a remote/sidecar
- All frontend Svelte components, routing, CSS, state
- Firebase auth (Firebase JS SDK works fine inside WebView)
- `localStorage`, `sessionStorage`, `IndexedDB` — all supported in WebView
- Service worker — supported in Android System WebView; not supported on iOS WKWebView (offline caching needs an alternative for iOS)
- `fetch()`, `EventSource`, `ReadableStream` — all work in WebView
- `navigator.clipboard` — works in WebView with HTTPS origin
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
        └── apple/      ← generated Xcode project (committed)
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

# iOS targets (macOS only)
rustup target add aarch64-apple-ios x86_64-apple-ios aarch64-apple-ios-sim

# Tauri CLI
cargo install tauri-cli --version "^2"

# Android Studio: install from https://developer.android.com/studio
# Then via SDK Manager install:
#   - NDK 28.0.12674087  ← required for Play Store 16KB page alignment
#   - Build-Tools 34.0.0
#   - Android SDK Platform 34

# Required env vars (add to ~/.zshrc or ~/.bashrc)
export JAVA_HOME="/Applications/Android Studio.app/Contents/jbr/Contents/Home"
export ANDROID_HOME="$HOME/Library/Android/sdk"
export NDK_HOME="$ANDROID_HOME/ndk/28.0.12674087"
```

iOS additionally requires: macOS, full Xcode (not just CLI tools), Cocoapods (`brew install cocoapods`), and an Apple Developer account ($99/year).

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

Initialise mobile projects (one-time, commit the generated folders):
```bash
cargo tauri android init
cargo tauri ios init      # macOS only
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

## 6. Storage — What Works, What Needs Attention

| Storage | Android WebView | iOS WKWebView | Action needed |
|---|---|---|---|
| `localStorage` | ✅ | ✅ | None |
| `sessionStorage` | ✅ | ✅ | None |
| `IndexedDB` (workspace cache) | ✅ | ✅ | None |
| `navigator.clipboard` | ✅ | ✅ | None |
| Service Worker | ✅ | ❌ | iOS: disable SW registration; accept no offline caching for v1 |
| `EventSource` / SSE | ✅ | ✅ | None |
| `ReadableStream` (chat) | ✅ | ✅ | None |
| `<audio>` (TTS) | ✅ | ✅ | None |

iOS is the only platform where the service worker doesn't run. The practical impact is no offline caching on iOS — acceptable for v1.

---

## 7. Build Pipeline

### Development
```bash
# Android (starts Vite dev server + Android emulator/device)
cargo tauri android dev

# iOS (macOS only)
cargo tauri ios dev
```

### Sideload APK (primary distribution path)
```bash
# Debug build — no signing required, install immediately
cargo tauri android build -- --apk --debug

# Release build — requires keystore (see §9), better performance
cargo tauri android build -- --apk
```

Output: `src-tauri/gen/android/app/build/outputs/apk/{debug,release}/app-*.apk`

### Play Store AAB (future, once eligible)
```bash
cargo tauri android build -- --aab
```

### iOS IPA (future)
```bash
cargo tauri ios build
```

First build: 10–20 minutes (full Rust cross-compile for 4 Android targets). Subsequent builds with Rust cache: 3–5 minutes.

---

## 8. CI/CD (GitHub Actions)

Android CI runs on `ubuntu-latest` (cheap). iOS CI requires `macos-latest` ($0.08/min vs $0.008/min for Linux — 10× more expensive).

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
      - run: cargo tauri android build -- --aab
        env:
          NDK_HOME: ${{ env.ANDROID_SDK_ROOT }}/ndk/28.0.12674087
      - uses: actions/upload-artifact@v4
        with:
          name: android-aab
          path: src-tauri/gen/android/app/build/outputs/bundle/release/*.aab
```

> Note: `tauri-action@v0` mobile support is experimental and doesn't set up the SDK/NDK. The manual workflow above is the reliable path.

### New GitHub secrets needed

| Secret | Value |
|---|---|
| `ANDROID_KEY_ALIAS` | Keystore alias |
| `ANDROID_KEY_PASSWORD` | Keystore password |
| `ANDROID_KEYSTORE_B64` | `base64 -i upload-keystore.jks` |
| `APPLE_CERTIFICATE_B64` | (iOS only) Distribution cert |
| `APPLE_CERTIFICATE_PASSWORD` | (iOS only) |
| `APPLE_PROVISIONING_PROFILE_B64` | (iOS only) |
| `APPLE_API_KEY_ID` | (iOS only) App Store Connect API key |
| `APPLE_API_ISSUER` | (iOS only) |

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
| NDK 28 signing compliance | $0 | Already required for sideload builds |

### If iOS is added later

| Item | Cost | Notes |
|---|---|---|
| Apple Developer Program | $99/year | Required for any iOS distribution |
| GitHub Actions iOS CI | ~$0–15/month | `macos-latest` is free for public repos; private repos burn quota 10× faster than Linux — fine for infrequent release builds |

---

## 10. Deployment to Devices

### Android — Sideloading (primary path, no store account needed)

**Step 1 — Enable unknown sources on your device**
Settings → Apps → Special app access → Install unknown apps → allow your file manager or browser.

**Step 2 — Build the APK**
```bash
# Debug: no signing setup needed, install immediately
cargo tauri android build -- --apk --debug

# Release (better perf): requires keystore first (one-time, see below)
cargo tauri android build -- --apk
```

**Step 3 — Install on device** (pick any method):

| Method | Command / Steps |
|---|---|
| ADB over USB | `adb install app-debug.apk` |
| ADB over Wi-Fi | `adb connect <device-ip>:5555` then same |
| File transfer | Copy APK via USB/cloud, tap to install on device |
| Local server | `python3 -m http.server` in build folder, open URL on device |

**Keystore setup** (one-time, needed for release APK):
```bash
keytool -genkey -v -keystore ~/dastill.jks -keyalg RSA \
  -keysize 2048 -validity 10000 -alias dastill
```
Then reference it in `src-tauri/gen/android/app/build.gradle.kts` signing config.

**Updating the app** — build a new APK and install over the existing one. Android keeps app data between installs as long as the signing key is the same.

---

### Android — Development mode (hot reload)
```bash
cargo tauri android dev   # connects to device/emulator with live reload
```

---

### Android — Play Store (future, once YouTube policy compliant)

1. Generate signed AAB: `cargo tauri android build -- --aab`
2. Create Google Play developer account ($25 one-time)
3. Upload to Play Console → internal testing track → staged rollout

---

### iOS (future)

**Development** (device must be registered in Apple Developer portal):
```bash
cargo tauri ios dev       # deploys to registered device via Xcode
```

**Distribution** — TestFlight (beta) or App Store ($99/year Apple Developer account required).

```bash
xcrun altool --upload-app --type ios --file "$APPNAME.ipa" \
  --apiKey $APPLE_API_KEY_ID --apiIssuer $APPLE_API_ISSUER
```

---

## 11. What Needs to be Written / Rewritten

### Net new (~medium effort)
- `src-tauri/` — Tauri shell config, `Cargo.toml`, capabilities
- `MainActivity.kt` — ActionMode plugin (30–50 lines of Kotlin)
- `frontend/src/lib/native-selection.ts` — JS bridge handlers
- `.github/workflows/android.yml` — Android CI workflow (produces APK artifact)
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
- Existing web deployment (the Tauri app and PWA coexist — same codebase, different build targets)

---

## 12. Risks and Caveats

| Risk | Severity | Mitigation |
|---|---|---|
| SSR → SPA conversion touches auth flow | High | Must be tested thoroughly; session handling changes meaningfully |
| Android WebView version variance on old devices | Medium | Set `minSdkVersion 26` (Android 8+); affects ~3% of devices |
| iOS CI is painful to set up | Medium | Out of scope for now; add later |
| WKWebView on iOS has no service worker | Low | Out of scope for now |
| `tauri-action` mobile support is experimental | Low | Use the manual workflow; it's 50 lines of YAML |
| Play Store 16KB page alignment (NDK 28) | Low | NDK 28 required anyway; non-issue if set up from day one |
| Web PWA and Tauri app must stay in sync | Ongoing | Same frontend codebase; single source of truth |

---

## 13. Recommended Phasing

### Phase 1 — Sideload on your own device (1–2 weeks)
1. Convert SvelteKit to SPA mode + auth refactor
2. Set up Tauri shell, local Android build
3. Write ActionMode Kotlin plugin
4. Generate signing keystore
5. Build release APK, install via ADB or file transfer
6. Done — app works on your device, zero store involvement

### Phase 2 — CI-produced APK artifact (1 week)
1. Android CI workflow — produces a signed APK on every push
2. Download artifact from GitHub Actions and sideload without a local build environment
3. Useful for sharing with other devices/testers without needing the full toolchain

### Phase 3 — Play Store (when YouTube policy allows)
1. Google Play developer account ($25 one-time)
2. Switch CI to produce AAB
3. Upload to Play Console → internal testing → staged rollout

### Phase 4 — iOS (optional, when needed)
1. Apple Developer account ($99/year)
2. iOS build locally + TestFlight
3. App Store submission

The web PWA continues to work throughout — Tauri is purely additive to the existing deployment.
