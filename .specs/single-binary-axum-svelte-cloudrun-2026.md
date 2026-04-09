# Static Web Hosting Migration (Firebase Hosting, backend stays on Cloud Run)

## Recommendation

Move the static Svelte frontend and the static docs site off Cloud Run and onto Firebase Hosting.

Do not combine that migration with a blanket `/api/**` Firebase Hosting rewrite to the backend.

That rewrite pattern looks attractive for CORS removal, but it is a bad fit for this app right now because the product already uses long-lived streaming endpoints. Firebase Hosting rewrites to Cloud Run are subject to a 60-second request timeout, while this backend explicitly supports multi-minute chat streams.

The clean version of this migration is:

- Firebase Hosting serves static frontend assets.
- Firebase Hosting serves static docs assets.
- The backend remains a standalone Cloud Run service.
- The frontend continues to call the backend directly via its public base URL.
- CORS remains enabled for the frontend origin and localhost dev.

This removes the frontend container, the docs container, both image build/push paths, and both static-site Cloud Run services.

---

## Why this version

### What gets better

- Static assets move to a CDN-backed product that is designed for SPAs.
- The repo no longer builds and deploys an nginx image just to serve `frontend/build/`.
- The repo no longer builds and deploys an nginx image just to serve VitePress docs.
- The frontend no longer has Cloud Run cold-start behavior.
- The docs site no longer has Cloud Run cold-start behavior.
- The frontend no longer needs a service account or Cloud Run IAM surface.
- Docs no longer need a Cloud Run runtime surface either.
- Static-site deploys become simpler and faster.

### What does not get magically better

- Backend costs do not change.
- CORS does not disappear in this first migration.
- Firebase Hosting is not automatically cheaper than today in every traffic shape.

### Why the original same-origin rewrite plan is too risky

- The frontend already uses streamed responses and SSE-like long-lived flows.
- The backend chat path is intentionally configured for streams that can last up to 30 minutes.
- Firebase Hosting rewrites to Cloud Run have a 60-second timeout.

That means the original "proxy all `/api/**` through Hosting" plan can break chat and status streams even if normal JSON endpoints work.

---

## Cost Reality

This migration probably makes sense, but mostly for architecture and delivery quality, not because it is guaranteed to slash cost.

### Current frontend cost model

Today the static-site costs come from:

- Cloud Run frontend request/CPU/RAM usage
- Cloud Run docs request/CPU/RAM usage
- Cloud Build time for frontend image builds
- Cloud Build time for docs image builds
- Artifact Registry storage for frontend images
- Artifact Registry storage for docs images

Relevant official pricing as checked on April 9, 2026:

- Cloud Run request-based services include a free tier of 2 million requests/month, 180,000 vCPU-seconds/month, and 360,000 GiB-seconds/month.
- Cloud Build includes 2,500 free build-minutes/month, then charges per build minute.
- Artifact Registry includes 0.5 GB free storage, then charges per GB-month.

### Firebase Hosting cost model

Relevant official pricing as checked on April 9, 2026:

- Hosting includes 10 GB storage free.
- Hosting includes 360 MB/day data transfer free.
- After that, Hosting charges $0.026/GB stored and $0.15/GB transferred.

### Practical interpretation for this repo

- The current built frontend is small: roughly 3.1 MB in `frontend/build/`.
- The docs site is also a plain static build produced by VitePress.
- If every visit were a completely cold load, the Hosting free transfer tier would cover roughly 100 to 120 full loads/day.
- In practice, immutable JS/CSS chunks are CDN-cached and browser-cached, so real transfer per repeat user should be lower than a full cold load.
- The current frontend and docs Cloud Run services also may already sit mostly inside free tiers if traffic is modest.

### Bottom line on cost

- If traffic is low to moderate, the direct dollar difference may be small either way.
- If traffic grows, Hosting is still a better product fit for static assets, but bandwidth becomes the main cost driver.
- The strongest reasons to do this migration are operational simplification, CDN delivery, and removing an unnecessary runtime, not a guaranteed big savings number.

---

## Target Architecture

### Phase 1 target

- `PROJECT_ID.web.app` serves the static SPA from Firebase Hosting.
- Docs are served from Firebase Hosting as well.
- The frontend uses a configured backend base URL for API calls.
- The backend stays on Cloud Run at its current service URL.
- `BACKEND_CORS_ALLOWED_ORIGINS` includes the Firebase Hosting origin and local dev origins.
- Docs remain a separate static site, but no longer on Cloud Run.

### Explicit non-goal for this migration

- No Firebase Hosting rewrite for `/api/**` in this phase.

### Optional future phase

Only after production validation, consider Firebase Hosting rewrites for short-lived non-streaming endpoints. Keep streaming endpoints direct unless you can prove they stay comfortably under Hosting's timeout constraints.

### Docs URL strategy

Use one of these, explicitly:

- Preferred: a separate Hosting site or custom domain for docs, such as `docs.example.com`
- Acceptable: the default Hosting subdomain for docs if a separate custom domain is not needed yet

Do not force docs under the same Hosting site unless you explicitly want a path-based information architecture like `/docs`.

---

## Repo Facts That Shape The Plan

- `frontend/src/lib/api-client.ts` already handles an empty or absent `VITE_API_BASE` safely, but it also supports a full backend URL.
- `frontend/vite.config.ts` already proxies `/api` in local dev.
- The frontend currently depends on streamed API responses for chat and on `EventSource` for search status.
- The backend chat streaming path is intentionally configured for long-running streams.
- `frontend/firebase.json` currently contains emulator/auth config only and is not in the repo root where Hosting config should live.
- `docs/` is a static VitePress site currently packaged into its own nginx container.
- The deploy workflow currently resolves both backend and docs URLs dynamically before building the frontend image.
- `PUBLIC_DOCS_URL` is still required by the frontend, so frontend deploy independence requires either resolving docs URL directly inside the frontend job or moving docs to a stable configured URL source.

---

## Implementation Plan

- [ ] **Phase 1: Root Firebase config for static hosting**
  - Move Firebase config to repo root.
  - Merge the existing auth emulator config into a root `firebase.json`.
  - Add Hosting config for the app with:
    - `public: "frontend/build"`
    - SPA fallback to `/index.html`
    - cache headers for immutable built assets
    - `index.html` set to `Cache-Control: no-cache`
  - Add Hosting config for docs with:
    - `public: "docs/.vitepress/dist"`
    - standard static asset caching
    - no SPA fallback unless docs navigation actually requires one
  - Add `.firebaserc` at repo root pointing to the existing Firebase project.
  - Do not add a `/api/**` Cloud Run rewrite in this phase.

- [ ] **Phase 2: Frontend build stays static, API stays direct**
  - Keep the frontend static build.
  - Keep `VITE_API_BASE` as an explicit backend origin for production builds.
  - Continue using the existing Vite dev proxy for local dev.
  - Add or update a unit test around API URL resolution so production direct-origin and local relative-path behavior are both covered.
  - Keep `PUBLIC_DOCS_URL` unless docs are given a stable custom domain or another stable source.

- [ ] **Phase 3: CI/CD switch from container deploy to Hosting deploy**
  - Remove the frontend Docker build/push/deploy steps from `.github/workflows/deploy.yml`.
  - Remove the docs Docker build/push/deploy steps from `.github/workflows/deploy.yml`.
  - Build frontend static files in CI with Bun.
  - Build docs static files in CI with Bun.
  - Deploy Hosting from the repo root using Firebase CLI with the existing Google auth flow.
  - Use ADC/WIF-based auth already established in the workflow, not `FIREBASE_TOKEN`.
  - Frontend deploy may run after `checks` without `needs: deploy-backend` or `needs: deploy-docs`, as long as the job can resolve the current backend URL and configured docs URL itself.
  - Prefer making the docs URL stable as part of this migration so the frontend no longer depends on deploy-time discovery of a docs Cloud Run URL.

- [ ] **Phase 4: Backend config changes**
  - Update backend deploy env so `BACKEND_CORS_ALLOWED_ORIGINS` includes:
    - `https://PROJECT_ID.web.app`
    - any production custom frontend domain if used
    - existing localhost dev origins
  - Do not remove CORS entirely.

- [ ] **Phase 5: Terraform cleanup**
  - Remove the frontend Cloud Run service from `terraform/cloud_run.tf`.
  - Remove the docs Cloud Run service from `terraform/cloud_run.tf`.
  - Remove the frontend public IAM binding.
  - Remove the docs public IAM binding.
  - Remove the frontend service account and frontend-specific IAM grants from `terraform/iam.tf`.
  - Remove the docs service account and docs-specific IAM grants from `terraform/iam.tf` if nothing else uses them.
  - Remove frontend secret-access bindings that only existed for the frontend runtime container.
  - Remove docs runtime bindings that only existed for the docs runtime container.
  - Keep Firebase project/web-app/auth resources that are still needed by the browser app.
  - Add Terraform-managed Hosting site resources only if you need multiple Hosting sites or want Terraform to explicitly own them.
  - Run `terraform plan` and verify that the destroys are limited to the frontend/docs Cloud Run runtime surfaces and their unused IAM/service-account pieces.

- [ ] **Phase 6: Cleanup**
  - Delete `frontend/Dockerfile`.
  - Delete `frontend/nginx.conf`.
  - Delete `docs/Dockerfile`.
  - Delete `docs/nginx.conf`.
  - Update deployment docs to describe Hosting-based frontend deploys.
  - Update deployment docs to describe Hosting-based docs deploys.
  - Remove any frontend/docs image or artifact references that are no longer used.

- [ ] **Phase 7: Optional follow-up investigation, not part of this migration**
  - Evaluate whether a split routing model is worth it:
    - static frontend on Hosting
    - short-lived JSON endpoints optionally behind Hosting rewrites
    - streaming endpoints remain direct to Cloud Run
  - Only do this if the added complexity is worth the reduced CORS surface.

---

## Requirements

- [ ] `https://PROJECT_ID.web.app` serves the SPA successfully.
- [ ] Docs are served successfully from their chosen Hosting URL.
- [ ] Firebase Auth still works from the Hosting origin.
- [ ] The frontend can call the backend successfully using the configured backend origin.
- [ ] Chat streaming still works.
- [ ] Search status streaming still works.
- [ ] The frontend Cloud Run service is deleted.
- [ ] The docs Cloud Run service is deleted.
- [ ] The frontend CI job no longer builds or pushes a frontend container image.
- [ ] The docs CI job no longer builds or pushes a docs container image.
- [ ] Local development still works with the existing Vite proxy.
- [ ] `terraform plan` only removes the no-longer-used frontend/docs runtime resources.

---

## Verification Gates

- [ ] `bun install --frozen-lockfile`
- [ ] `bun run format:check`
- [ ] `bun run lint`
- [ ] `bun run check`
- [ ] `bun run test`
- [ ] `bun run build`
- [ ] Local Hosting smoke test for SPA routes
- [ ] Docs Hosting smoke test
- [ ] Production smoke test from `PROJECT_ID.web.app`
- [ ] Production smoke test from the docs Hosting URL
- [ ] Production chat stream smoke test
- [ ] Production search status stream smoke test
- [ ] `terraform plan` reviewed before apply

---

## Rejected For Now

- Firebase Hosting rewrite for all `/api/**`
  - Rejected because Hosting applies a 60-second timeout to rewritten Cloud Run requests, which is incompatible with this app's long-running stream behavior.

- Removing CORS entirely
  - Rejected because the backend remains on its own origin in this migration.

- Claiming meaningful guaranteed cost savings up front
  - Rejected because the current frontend may already fit mostly within Google free tiers, and Hosting cost becomes bandwidth-shaped at scale.

---

## Sources checked on April 9, 2026

- Firebase Pricing: Hosting storage and transfer pricing
- Firebase Hosting with Cloud Run docs: Hosting rewrite behavior and 60-second timeout
- Firebase Hosting cache docs: cookie stripping behavior for rewrites
- Cloud Run pricing: free tier and request-based pricing
- Cloud Build pricing
- Artifact Registry pricing
