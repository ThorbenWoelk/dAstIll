# Feature: Mobile Interaction Accessibility

Priority: **P0**

## Outcome

Make the mobile UI operable with touch first:

- critical actions are available without hover
- interactive controls meet touch-target expectations
- text entry does not trigger iOS zoom regressions
- user-readable text remains legible on phone-sized screens
- the feature guide remains tappable and visible on mobile viewport changes

## Source Of Truth

Use [design.md](../design.md) for:

- touch target expectations
- typography rules
- opaque overlay/popover rules
- E2E requirements for DOM-visible regressions

Do not restate design-token, color, or typography system details here unless the repo source of truth changes.

## Consolidated Scope

This feature replaces the intent of:

- `01-touch-targets.md`
- `03-mobile-accessible-actions.md`
- `04-ios-input-zoom.md`
- `05-hover-to-touch.md`
- `06-mobile-typography.md`
- `09-feature-guide-mobile.md`

## Requirements

### 1. Touch Targets

On screens below `lg`, user-tappable controls must expose a touch target of at least `44x44px`.

Apply this to:

- nav and sidebar controls
- content action buttons
- chat action buttons
- modal, drawer, and guide controls
- audio player transport and seek affordances

If a visual control must stay small, expand its hit area structurally instead of relying on visual size alone.

### 2. No Hover-Only Critical Actions

Critical actions must be discoverable and usable on touch devices without hover:

- channel deletion
- chat conversation rename/delete
- chat message copy
- any other destructive or management action that is currently hover-gated

Touch-safe alternatives can be:

- always-visible mobile actions
- overflow menus
- action sheets
- long-press flows

Choose the smallest pattern that preserves clarity and matches the current UI language.

### 3. Mobile Input Usability

Text inputs, textareas, and selects used on mobile must avoid iOS auto-zoom regressions.

Acceptance:

- no mobile input used in normal flows renders below a safe focus size
- radio/checkbox rows are tappable as rows, not just on the native control
- date inputs remain readable on mobile

### 4. Mobile Legibility

User-readable text on mobile must be reviewed and normalized where it currently falls below practical readability.

Acceptance:

- no essential text ships at obviously unreadable micro-sizes
- interactive labels and buttons remain readable without zoom
- metadata styling still respects the product’s restrained visual language

### 5. Feature Guide On Mobile

The feature guide must remain visible and tappable on mobile:

- reposition on `visualViewport` changes
- avoid overflow on narrow screens
- expose touch-safe guide navigation controls

## Acceptance Criteria

- A touch user can complete key management flows without relying on hover.
- Mobile inputs no longer trigger the known focus-zoom regression in Safari-class browsers.
- All key visible controls in mobile paths are comfortably tappable.
- No DOM-visible mobile regression ships without at least one Playwright assertion that proves the intended UI appears.

## Verification

Required:

- `bun install --frozen-lockfile`
- `bun run format:check`
- `bun run lint`
- `bun run check`
- `bun run test`
- `bun run build`
- `bun audit --production`

Required Playwright coverage:

- at least one E2E check for mobile-visible destructive/management actions
- at least one E2E check for mobile-visible chat action affordances
- at least one E2E check for feature-guide rendering/tappability if the guide DOM changes

Manual/mobile QA:

- iOS Safari or equivalent real-device/simulator check for input focus zoom
- narrow-phone check for guide positioning and tap targets
