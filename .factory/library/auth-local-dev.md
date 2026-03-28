# Local auth verification notes

- Current local runtime does not match `.factory/services.yaml` exactly:
  - backend needs `PORT=3544 cargo run --bin dastill`
  - frontend needs `bun run dev -- --port 3543`
  - Firebase Auth Emulator starts with `cd frontend && firebase emulators:start --only auth --project demo-dastill`
- For browser checks, the app auto-connects to the Auth Emulator in dev even when `FIREBASE_AUTH_EMULATOR_HOST` is unset.
- If popup-based emulator sign-in is flaky under automation, you can still validate authenticated session exchange by creating an emulator Google ID token via `accounts:signInWithIdp` and POSTing that `idToken` to `/auth/session`.
