# ADR: Unified Redesign Shell and Navigation

## Status
Proposed

## Context
The application's interface has evolved through several prototypes, resulting in fragmented shells for Workspace, Highlights, and Chat. The `redesign-template.html` introduced a cohesive "Stone" aesthetic with a standardized 3-column shell on desktop and a fixed bottom bar on mobile. 

## Decision
We will standardize all top-level application sections (Workspace, Highlights, Queue, Chat) to use a single `WorkspaceShell` structure. This shell will include a unified desktop navigation rail and a global mobile bottom navigation bar. 

Specifically:
- **Queue** will move from a popover to a full route (`/queue`) with structural parity.
- **Highlights** will be refactored to support the sidebar snippet for better list management.
- **Navigation** will follow a strict 4-item mobile-first model: Workspace, Queue, Highlights, Chat.

## Alternatives Considered
- **Maintain popover-only Queue:** Rejected because users need a stable place to monitor long-running background tasks (transcription/summary) and manage failures.
- **Native header navigation on mobile:** Rejected in favor of the bottom bar to improve one-handed reachability on modern devices.

## Consequences
- **Positive:** Improved navigational consistency and professional visual "finish." Better monitoring for background operations.
- **Negative:** Increased initial implementation effort. Some existing section-specific layouts (e.g., Highlights) will need refactoring to fit the 3-column model.
- **Performance:** Slight increase in shared layout weight due to the unification.

## Follow-ups
- Update the Feature Guide (Tour) to reflect the new navigation layout.
- Refactor Vocabulary to a secondary preference rather than a top-level nav item if space is tight.
