# Three Pane Transcript Workspace

**Linear:** N/A

## Problem

The current UI stacks videos and transcript content in the same main column, which slows channel browsing and makes transcript-first reading less direct.

## Goal

Present a clear workspace with channels on the left, selected video transcript in the middle, and videos for the selected channel on the right, with an intuitive transcript availability filter.

## Requirements

- Desktop layout uses three panes:
- Left pane: channels list and follow controls
- Middle pane: selected video transcript/summary content
- Right pane: videos list for the selected channel
- Videos pane includes filter controls for transcript availability with options for:
- available transcripts only
- unavailable transcripts only
- all videos
- Filter behavior updates the visible video list immediately and keeps selection coherent.
- Existing content actions remain functional:
- load transcript/summary
- edit and save transcript/summary
- channel refresh and pagination
- Responsive behavior remains usable on smaller screens.

## Non-Goals

- Backend API changes for new filter endpoints
- New transcript generation logic
- Visual rebrand or design system replacement

## Design Considerations

- Keep the existing visual language and component patterns to minimize regression risk.
- Place transcript availability controls in the videos pane header where users decide what to browse.
- Default to transcript-ready videos for a transcript-first workflow.

## Open Questions

- None for current scope.
