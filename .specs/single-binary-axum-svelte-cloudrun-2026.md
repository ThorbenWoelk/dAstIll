# Frontend → Firebase Hosting

## Context

- **Problem**: The frontend is a pure static SPA (Svelte 5, adapter-static) served by an nginx container on Cloud Run. This is operational overhead with no benefit: it pays for a Cloud Run service, adds a cold-start path, requires Docker builds on every frontend change, and forces a hardcoded CORS origin in the backend config. Firebase Hosting already exists in the project (Terraform `firebase.tf`) and is the right tool for the job.
- **Goal**: Serve the frontend from Firebase Hosting. The frontend Cloud Run service (`dastill-frontend`) is decommissioned. API calls from the browser stay same-origin via a Firebase Hosting rewrite rule that proxies `/api/**` to the backend Cloud Run service — eliminating CORS entirely in production.
- **Linear**: —

---

## Key Facts (from codebase audit)

- All Axum routes already have the `/api` prefix — no backend route changes needed.
- `firebase.json` exists but only contains auth emulator config — needs a `hosting` section.
- No `.firebaserc` in the repo yet.
- `VITE_API_BASE` is baked into the Docker image at build time and currently set to the full backend Cloud Run URL.
- `BACKEND_CORS_ALLOWED_ORIGINS` in `deploy.yml` is hardcoded to the Cloud Run frontend URL.
- `terraform/firebase.tf` derives `firebase_frontend_host` from `google_cloud_run_v2_service.frontend.uri`. This local must be updated to the Firebase Hosting domain after migration.
- `terraform/iam.tf` contains a `frontend_sa` service account with Cloud Run invoker + Firebase Admin bindings. These can be removed (static SPA on Firebase Hosting needs no service account).
- `terraform/cloud_run.tf` contains `google_cloud_run_v2_service.frontend` and its public IAM binding — both removed in Phase 4.
- Firebase Hosting rewrites proxy at the network edge; from the browser's perspective requests to `/api/**` are same-origin. No `Origin` header is sent on same-origin requests, so the backend CORS layer is not triggered in production. Localhost CORS origins remain valid for local dev.

---

## Implementation Plan

- [ ] **Phase 1: Firebase Hosting config**
  - Add `hosting` block to `frontend/firebase.json`:
    - `public: "build"` (Svelte static output dir)
    - SPA fallback: `{ "source": "**", "destination": "/index.html" }`
    - API proxy rewrite before the SPA fallback: `{ "source": "/api/**", "run": { "serviceId": "dastill-backend", "region": "europe-west3" } }`
    - Add `headers` rule for static asset cache-control (`**/*.js`, `**/*.css` → `Cache-Control: public,max-age=31536000,immutable`; `index.html` → `Cache-Control: no-cache`)
  - Create `.firebaserc` at repo root pointing to `$PROJECT_ID`.
  - Move `firebase.json` from `frontend/` to repo root (Firebase CLI expects it at root alongside `.firebaserc`). Update any path references.

- [ ] **Phase 2: Frontend build config**
  - Change `VITE_API_BASE` from the full backend URL to an empty string `""`. Relative fetch paths (`/api/...`) work naturally with Firebase Hosting rewrite in production and with the Vite dev proxy in local dev.
  - Verify `frontend/src/lib/api-client.ts` handles an empty `VITE_API_BASE` correctly (it normalises by stripping trailing slash — an empty string produces `""`, which prepends nothing to paths like `/api/health`). Add a unit test if not covered.
  - Remove `PUBLIC_CONTACT_EMAIL` and any other build-arg env vars that were only needed by the nginx/Cloud Run setup if they are no longer used.

- [ ] **Phase 3: CI/CD — deploy workflow**
  - Add Firebase CLI to the deploy job: `npm install -g firebase-tools` (or use `w9jds/firebase-action`).
  - Replace the "Build and Push Frontend Image" + "Deploy Frontend to Cloud Run" steps with:
    1. Build frontend static files: `bun install --frozen-lockfile && bun run build` with `VITE_API_BASE=""` and the other `PUBLIC_*` build args (Firebase config still needed).
    2. Deploy to hosting: `firebase deploy --only hosting --project $PROJECT_ID --token $FIREBASE_TOKEN` (or use WIF-based gcloud auth which is already set up).
  - Remove the Docker build step, the artifact registry push, and the Cloud Run deploy step for the frontend.
  - Update `BACKEND_CORS_ALLOWED_ORIGINS` in the backend deploy step: replace the Cloud Run frontend URL with the Firebase Hosting domain (`https://$PROJECT_ID.web.app`) — or remove it entirely since same-origin rewrites make it unreachable in production. Keep localhost origins for local dev via the existing defaults in `config.rs`.
  - Remove "Resolve backend and docs URLs" steps that exist solely for passing `VITE_API_BASE` into the Docker build.
  - The `deploy-frontend` job no longer needs to `needs: [deploy-backend, deploy-docs]` (it needed the URLs only for `VITE_API_BASE`). It can run independently after `checks`.

- [ ] **Phase 4: Terraform — add Hosting, remove Cloud Run frontend**
  - In `terraform/firebase.tf`:
    - Add `google_firebase_hosting_site` resource (or use the default site `$PROJECT_ID.web.app`).
    - Update `firebase_frontend_host` local to the Firebase Hosting domain instead of the Cloud Run frontend URI: `"${var.project_id}.web.app"`.
    - The `firebase_authorized_domains` local already includes `${var.project_id}.web.app` — verify it stays in the list after removing the Cloud Run URI.
  - In `terraform/cloud_run.tf`:
    - Remove `google_cloud_run_v2_service.frontend` resource.
    - Remove `google_cloud_run_v2_service_iam_member.frontend_public`.
  - In `terraform/iam.tf`:
    - Remove `google_service_account.frontend_sa`.
    - Remove `google_secret_manager_secret_iam_member.frontend_secrets` (frontend_secret_ids map).
    - Remove `google_cloud_run_v2_service_iam_member.backend_frontend_invoker` (frontend SA → backend invoker).
    - Remove `google_project_iam_member.frontend_firebase_auth` (static SPA has no server-side Firebase Admin).
    - Remove `google_service_account_iam_member.sa_user_frontend`.
  - Remove `frontend` from the `cicd_secret_ids` merge in `iam.tf` if it only existed for the Cloud Run deploy SA.
  - Run `terraform plan` and verify only frontend Cloud Run and frontend SA resources are destroyed.

- [ ] **Phase 5: Cleanup**
  - Delete `frontend/Dockerfile` and `frontend/nginx.conf` (no longer needed).
  - Remove the frontend service from `detect_changed_components.sh` Cloud Run–specific logic if any.
  - Decommission the `dastill-frontend` Cloud Run service: `gcloud run services delete dastill-frontend --region europe-west3` (or let Terraform handle it via `terraform apply`).
  - Update any docs (e.g., `docs/operations/deployment.md`) that reference the Cloud Run frontend URL or Docker-based frontend deploy.

---

## Requirements

- [ ] `https://$PROJECT_ID.web.app` serves the SPA and the app is fully functional (auth, chat, search).
- [ ] `https://$PROJECT_ID.web.app/api/health` returns a 200 from the backend (Firebase Hosting rewrite is active).
- [ ] No `Access-Control-Allow-Origin` request/response headers present on API calls from the production frontend (same-origin — CORS not involved).
- [ ] `dastill-frontend` Cloud Run service is deleted; no billing accrues for it.
- [ ] Frontend deploy in CI no longer builds or pushes a Docker image.
- [ ] Frontend deploys independently of the backend (no `needs: deploy-backend` in the frontend job).
- [ ] Local dev still works: Vite dev server proxies `/api/**` to `localhost:3001` (existing `vite.config.ts` proxy config), no change needed.
- [ ] `terraform plan` after Phase 4 changes shows zero diff (Hosting resource exists, frontend Cloud Run + SA are gone).
- [ ] `bun run check` and `bun run test` pass after `VITE_API_BASE` change.

---

## Verification Gates

- [ ] **Unit test**: `api-client.ts` handles `VITE_API_BASE=""` — all API URLs resolve correctly as root-relative paths.
- [ ] **Local smoke test**: `firebase serve --only hosting` with the built `frontend/build/` dir; SPA routes work, `/api/**` returns the rewrite target (or 404 from backend — not a 404 from hosting).
- [ ] **Terraform plan**: no surprise destroys; only expected frontend Cloud Run + frontend SA removals.
- [ ] **Deploy dry run**: CI `deploy-frontend` job completes without Docker or Cloud Run steps.
- [ ] **Production smoke**: `https://$PROJECT_ID.web.app` loads, Firebase Auth works, at least one API call succeeds via the rewrite.
- [ ] **CORS verification**: DevTools Network tab shows no CORS preflight (`OPTIONS`) requests to `/api/**` from production.

---

## Non-Goals

- Changing the backend routing structure (routes already have `/api` prefix — no change needed).
- Migrating Firebase Auth configuration (emulator config, providers — untouched).
- Moving the docs service (remains a separate Cloud Run deployment).
- Adding SSR or edge functions — this stays a static SPA.
- Changing the Vite dev proxy configuration — local dev flow is unaffected.
