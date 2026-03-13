# Start App Docs Integration

**Linear:** none

## Problem

`start_app.sh` currently starts only the product frontend and backend. The docs frontend remains a separate manual step even though the product already links to the local docs URL by default.

## Goal

Make `start_app.sh` start and supervise the docs frontend alongside the product frontend and backend so a single local startup command brings the full development environment online.

## Requirements

- `start_app.sh` starts the docs frontend in both attached and detached modes.
- The script cleans up old docs processes on the configured docs port before starting.
- The script waits for the docs frontend to become reachable before reporting readiness.
- Startup output and service logs clearly include the docs process and local docs URL.
- Local development documentation reflects that `./start_app.sh` now brings up docs too.

## Non-Goals

- Changing the production docs deployment.
- Reworking the docs frontend itself.
- Adding new startup flags beyond what is needed for the existing attached and detached modes.

## Design Considerations

- Keep the script structure aligned with the existing backend/frontend management so the docs process behaves consistently in attached and detached modes.
- Preserve configurable ports through environment variables rather than hardcoding the docs port in multiple places.
- Keep readiness checks HTTP-based, consistent with the current script behavior.

## Open Questions

- None at the moment. The required startup behavior is specific enough to implement directly.
