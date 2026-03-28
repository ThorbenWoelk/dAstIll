# Spec: Mobile Highlights Integration

## Goal

Make highlights work on mobile devices where the native Android text selection toolbar overlays the app's custom selection toolbar.

## Implementation Details

- Detect mobile devices via media query (`(max-width: 1023px)`).
- On mobile, instead of a floating tooltip above the selection, show a fixed bottom bar with highlight actions.
- This allows both the native selection menu (to copy, share, etc.) and our actions to be visible and usable.

## Changes Made (Mar 2026)

### Fixed Mobile Toolbar Interaction

1. **Container Boundary Bug Fix**: The `handlePointerDown` event handler was clearing the tooltip when clicking outside the container element. On mobile, the fixed-position toolbar sits outside the container, so taps on toolbar buttons would clear the tooltip before the click could register.

   **Fix**: Added check to skip clearing tooltip if the pointerdown target is within `.text-action-toolbar`:
   ```typescript
   const target = event.target;
   if (target instanceof Element && target.closest(".text-action-toolbar")) {
     return;
   }
   ```

2. **Touch Capture**: Added `touch-action: none` CSS to the `.text-action-toolbar` class to prevent Android's touch handling from interfering with pointer events on the toolbar buttons.

3. **Pointer Event Handling**: Changed from `onmousedown` to `onpointerdown` on the toolbar container for consistent handling across touch and mouse input. The container now calls `preventDefault()` and `stopPropagation()` to ensure pointer events are captured before Android's Action Mode can intercept them.

4. **Accessibility**: Added `role="group"` and `aria-label="Text selection actions"` to the toolbar container for screen reader compatibility.

## Root Cause

The issue was a combination of:
1. The tooltip being cleared when tapping the toolbar (container boundary check)
2. Android's Action Mode intercepting touch events on overlay elements
3. Missing `touch-action: none` to allow custom touch handling

## Verification

- Resize browser to mobile width and select text in TranscriptView.
- Compare with desktop behavior.
- On Android device: after text selection, the native Action Mode appears at top, and the app's toolbar should be tappable at bottom.
