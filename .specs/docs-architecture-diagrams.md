# Docs Architecture Diagrams

## Problem

The documentation explains the system architecture and major data flows in prose, but it
does not visualize the runtime boundaries or how content moves between frontend, backend,
workers, and storage. That makes it harder to validate the architecture quickly and
harder for readers to map the written descriptions onto the actual system.

## Goal

Add explicit, accurate architecture and data-flow diagrams to the VitePress docs so the
docs UI shows the system boundaries and major processing flows directly on the relevant
pages.

## Requirements

- Add diagram support that renders correctly in the current VitePress docs UI.
- Document the main runtime architecture with a diagram that reflects the current codebase.
- Document the primary frontend/backend and persistence data flows with diagrams that
  reflect the current codebase.
- Keep the diagrams colocated with the existing architecture and flow pages instead of
  creating a parallel docs structure.
- Verify the docs build succeeds after the diagram changes.

## Non-Goals

- Reworking the overall docs information architecture or sidebar structure.
- Changing product, backend, or infrastructure behavior.
- Producing exhaustive low-level sequence diagrams for every endpoint or worker path.

## Design Considerations

- The docs should use one consistent diagram mechanism so authors do not need multiple
  rendering conventions.
- Diagrams should be small enough to stay readable inside the VitePress content column
  and in both light and dark themes.
- The written prose should remain the source of nuance, while diagrams make boundaries
  and flow direction obvious at a glance.

## Open Questions

- None at the moment. The relevant pages and the documentation goal are clear.
