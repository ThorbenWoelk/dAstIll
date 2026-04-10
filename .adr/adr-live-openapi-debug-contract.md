# ADR: Live OpenAPI Document For Local Backend Debugging

## Decision

Use the running backend's `/api/openapi.json` endpoint as the source of truth for local Postman debugging.

The checked-in `backend/openapi.postman.yaml` file may still exist as a snapshot or export artifact, but it is not the primary contract for local debugging.

## Context

The repo already keeps the real backend contract in code:

- Axum route registration in `backend/src/main.rs`
- request and response DTOs in `backend/src/models.rs`
- a handwritten frontend API client plus generated TypeScript bindings

The previous Postman YAML file was manually maintained and had already drifted behind the actual router surface. That made it useful as a rough reference, but unreliable for debugging the live backend.

## Alternatives considered

- Keep maintaining `backend/openapi.postman.yaml` by hand.
- Remove OpenAPI support entirely and debug the backend with raw requests only.
- Generate only a checked-in YAML snapshot without exposing a live OpenAPI endpoint.

## Consequences

- Postman can import a contract that matches the running backend instead of a stale file.
- Backend route and DTO changes now require corresponding OpenAPI annotations to keep the live document useful.
- A checked-in YAML snapshot can still be kept later for review or sharing, but drift should be treated as a secondary artifact problem, not a local-debugging problem.

## Follow-ups

- Add a dedicated OpenAPI workflow if the repo still wants a checked-in YAML snapshot kept current.
- Consider exporting the live OpenAPI document to YAML automatically instead of hand-editing `backend/openapi.postman.yaml`.
