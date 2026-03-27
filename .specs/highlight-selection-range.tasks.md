# Tasks: Highlight Selection Range

## Current State
Selection offset mapping is fixed, and legacy summary highlights now resolve against normalized summary text. Targeted tests, frontend checks, build, and pre-commit validation have passed again.

## Steps
- [x] Inspect the highlight selection and rendering flow.
- [x] Write a failing regression test for partial saved selections.
- [x] Implement a DOM-range-to-text-offset fix.
- [x] Run targeted verification, frontend checks, and pre-commit validation.
- [x] Write a failing regression test for summary highlight resolution against normalized content.
- [x] Implement a backward-compatible summary highlight matching fix.
- [x] Re-run targeted verification, frontend checks, and pre-commit validation.

## Decisions Made During Implementation
- The fix will stay frontend-only unless verification shows backend truncation.
- Selection offsets are now derived from DOM boundary points rather than `Selection.toString()` lengths, which avoids browser-dependent whitespace normalization shortening saved highlights.
- Highlight range resolution now tolerates stored `Summary:` / `Transcript:` prefixes so older highlights still render after display-time content normalization removes those labels.
