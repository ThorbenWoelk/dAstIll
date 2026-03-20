# Tasks: Security hardening

## Current State
The security hardening spec has been implemented end to end. Deployment now wires the new secrets and proxy env vars, the backend Cloud Run service is no longer public, the deploy workflow is deployment-only, and CI validation/audits now run in dedicated `Checks` and `Security` workflows. Local validation has passed for backend/frontend compile, tests, and audits. The only remaining audit note is an upstream RustSec maintenance warning on `fxhash` via `scraper`.

## Steps
- [x] Read `security-hardening.md` and map the required work onto the current frontend/backend architecture.
- [x] Add frontend session gating and same-origin API proxying so the browser stops calling the backend directly.
- [x] Add backend auth middleware and operator-route guards.
- [x] Remove user-derived prompt/title fields from telemetry.
- [x] Split side-effecting GET endpoints into read-only GET plus explicit POST triggers.
- [x] Update frontend API callers and streaming logic to use the new API contract.
- [x] Restrict backend CORS to configured origins.
- [x] Add request rate limiting for baseline and expensive endpoints.
- [x] Add dependency audit steps to CI and document the new auth/runtime config.
- [x] Run backend/frontend validation and capture results.

## Decisions Made During Implementation
- The current browser-to-backend direct topology is not compatible with making the backend private at the platform layer without first introducing a frontend proxy.
- The implementation therefore moves the browser to an authenticated frontend session and same-origin proxy, while the backend now trusts only proxy-authenticated requests and separates operator-only routes with dedicated middleware.
- Read-only `GET` routes now return stored data only; the frontend explicitly triggers transcript, summary, and video-info generation via new `POST .../ensure` endpoints.
- CI runs `cargo audit` plus `bun audit --production`; the docs audit intentionally scopes to production dependencies so the VitePress dev-server `esbuild` advisory does not block deploys for the static production site.
- CI responsibilities are split so deploy no longer installs toolchains or runs audits on the release path; `Checks` handles compile/test/build gates and `Security` handles dependency audits on PRs, pushes to `main`, and a weekly schedule.
- Terraform now removes public invoke access from the backend and grants invoke access only to the frontend service account. The frontend/docs Cloud Run services remain publicly reachable at the platform edge for now; the frontend itself is protected by the new app-session gate, and moving the browser-facing services behind Google identity would require a separate IAP/front-door change.
