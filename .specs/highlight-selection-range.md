# Highlight Selection Range

**Linear:** none

## Problem

Sometimes saving a highlight captures less text than the user visibly selected in the transcript/summary reader.

## Goal

Map DOM selections to the article's raw text offsets reliably so saved highlights cover the full intended selection.

## Requirements

- Saving a highlight must preserve the full selected text across inline formatting boundaries.
- The selection offset calculation must not depend on whitespace-normalizing DOM stringification that can shorten the computed range.
- Existing highlight creation and rendering behavior must continue to work for transcript and summary content.
- The fix must be covered by automated frontend tests.

## Non-Goals

- Changing highlight storage format.
- Redesigning the selection tooltip UI.
- Changing highlight rendering or deletion behavior outside the offset fix.

## Design Considerations

- The bug is likely in client-side DOM range to text offset translation, not backend persistence.
- The safest fix is to centralize selection-to-offset calculation in a utility that can be unit tested directly against DOM ranges.
