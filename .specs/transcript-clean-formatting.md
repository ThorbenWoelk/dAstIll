# Transcript Clean Formatting Action

## Problem
Transcript editing currently has no one-click way to clean transcript formatting, so users must manually reflow long blocks of text.

## Goal
Provide a minimal UI action in the transcript editor that sends transcript text to Ollama for formatting while guaranteeing the cleaned output preserves the same underlying transcript wording.

## Requirements
- Transcript editor includes a compact icon action for clean formatting while editing transcript content.
- Clicking the action sends current transcript draft text to backend formatting API powered by Ollama.
- Backend enforces text preservation by validating input vs output token sequence and falling back to original input if changed.
- Formatting action updates editor draft with returned content without auto-saving.
- Existing transcript/summary read and save behaviors remain unchanged.

## Non-Goals
- Replacing the summary generation workflow.
- Changing transcript extraction behavior.
- Introducing automatic background transcript reformatting.

## Design Considerations
Use existing `SummarizerService` Ollama integration to avoid duplicating client setup. Validation should happen server-side so all clients receive the same safety guarantee.

## Open Questions
- None.
