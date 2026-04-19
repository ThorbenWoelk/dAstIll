# PRD: Redesign Unification

## Problem
The dAstIll redesign is currently fragmented. While the `redesign-template.html` presents a unified vision, the implementation has gaps:
- The **Queue** only exists as a transient popover, lacking a dedicated monitoring and management interface.
- **Mobile navigation** is inconsistent across sections (Workspace vs. Highlights vs. Chat).
- **Highlights** and **Vocabulary** lack the 3-column structural parity of the Workspace.
- **Appearance settings** are limited to mode and basic color, missing the expanded palette and accessibility options in the template.

## Goal
Unify the application's core shells and navigation systems to match the "Stone" aesthetic and structural model defined in the redesign template.

## Current Increment
**Phase 1: Shell & Navigation Parity**
- Implement the dedicated `/queue` page.
- Unify the `WorkspaceShell` to support the `/highlights` and `/queue` sections with 3-column parity.
- Deploy the global fixed mobile bottom navigation bar.

## Clear Deliverable
A unified app shell where navigating between Workspace, Highlights, and Queue feels seamless, using the same navigation rails and a consistent mobile bottom bar.

## Non-Goals
- Real-time collaborative features.
- Redesigning the underlying API or data models.
- Implementing the "Deep Search" logic (this spec focuses on the UI/Shell).

## Users or Actors
- **Power Users:** Need the desktop 3-column layout to manage large libraries.
- **Mobile Users:** Rely on the fixed bottom bar for one-handed navigation between core app functions.

## Requirements

### 1. Operations Queue Page (`/queue`)
- 3-column layout (Nav Rail | Channel/Source List | Queue Detail).
- Rows for:
    - **Processing:** Active transcript/summary extraction with "Cancel" affordance.
    - **Pending:** Queued items with status label.
    - **Error:** Failed items with "Details" and "Retry" actions.
    - **Completed:** Line-through items with "Done" status.

### 2. Global Mobile Bottom Navigation
- Fixed at `bottom: 0`.
- Targets: **Read (Workspace)**, **Queue**, **Saved (Highlights)**, **Chat**.
- Respects safe areas and uses opaque surfaces (no transparency).

### 3. Structural Parity
- `/highlights` updated to use `sidebar` snippet for channel-based filtering, matching the Workspace.
- Consistent header height and logo/action alignment across all routes.

## Risks and Open Questions
- **Risk:** Mobile bottom bar might conflict with section-specific bottom sheets or drawers.
- **Question:** Should "Vocabulary" be a top-level nav item or moved into settings? (Template suggests 4 core items).
