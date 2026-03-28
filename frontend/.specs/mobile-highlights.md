# Spec: Mobile Highlights Integration

## Goal

Make highlights work on mobile devices where the native Android text selection toolbar overlays the app's custom selection toolbar.

## Implementation Details

- Detect mobile devices via media query (`(max-width: 768px)` or similar).
- On mobile, instead of a floating tooltip above the selection, show a fixed bottom bar with highlight actions.
- This allows both the native selection menu (to copy, share, etc.) and our actions to be visible and usable.

## Verification

- Resize browser to mobile width and select text in TranscriptView.
- Compare with desktop behavior.
