# dAstIll Design System

This file (`design.md`) is the source of truth for this repository's frontend design system and frontend engineering standards.
Do not duplicate these rules in `AGENTS.md`; link here from there instead.

## Philosophy

Muted, zen, minimalistic. Content first, no decorative chrome. Prefer restraint over expressiveness - if something can be removed, remove it. No emojis anywhere.

**Borders and boxes**: Use as little as possible. Prefer whitespace, typography weight, and color contrast to create separation and hierarchy. Borders and background boxes around text or UI elements should be a last resort.

**No accent outline chrome**: Do not use decorative accent-colored line borders, outline rings, or thin red/orange strokes around cards, rows, pills, or spotlighted elements. The accent family may tint fills, text, or soft glows, but it must not appear as a visible perimeter stroke except when conveying a true semantic state such as danger or destructive confirmation.

---

## Visual Atoms

### Typography

- **Body**: [Manrope](https://fonts.google.com/specimen/Manrope) (system-ui fallback), `-webkit-font-smoothing: antialiased`
- **Headings / Serif Moments**: [Fraunces](https://fonts.google.com/specimen/Fraunces) (`font-variation-settings: "opsz" 72`, `letter-spacing: -0.02em`, `font-weight: 600`)
- **UI Labels / Tabs / Tooltips**: Uppercase, `font-weight: 700`, `letter-spacing: 0.05-0.08em`, `font-size: 10-11px`

### Color System

All colors are CSS custom properties (`var(--token)`). Never use hardcoded hex values.

**Semantic vs decorative**

- **Semantic colors** encode meaning or structure. Users should infer state or risk from them. Use them consistently: do not repurpose a semantic token for decoration, and do not use a decorative token to stand in for meaning (for example, never use `--accent` where `--danger` is required).
  - **Content and layers**: `--foreground`, `--soft-foreground`, `--background`, `--surface`, `--surface-strong`, and related surface/overlay tokens establish hierarchy and readability.
  - **Risk and destruction**: `--danger` (and any danger-derived tokens) only for destructive or high-risk actions and messaging.
  - **Structure when it carries meaning**: `--border`, `--border-soft` for separation that clarifies layout or grouping, not for ornament.

- **Decorative colors** set mood, brand, and motion. They may change with `data-color` or theme without changing what the UI *means*. The **accent family** (`--accent`, `--accent-soft`, `--accent-strong`, `--accent-wash`, `--color-swatch`, and other palette-derived tokens) is primarily decorative in hue; it still signals *interactivity* or *focus*, but the chosen palette is a visual preference, not a data encoding.
  - **Palettes** (`gold`, `ember`, `sage`, `ocean`, `sand`, `plum`): decorative swaps for the accent system.
  - **Atmosphere**: shell gradients, subtle washes, and logo wordmark treatment use decorative contrast; they must not be the only cue for errors, success, or destructive actions.

| Token               | Light Value                | Dark Value             | Role                 |
| ------------------- | -------------------------- | ---------------------- | -------------------- |
| `--background`      | `#faf9f6` (warm off-white) | `#111315` (near-black) | Page shell only      |
| `--foreground`      | `#1a1a1a`                  | `#f4efe9` (warm white) | Primary text         |
| `--surface`         | `#ffffff`                  | `#181b1f`              | Panels, cards        |
| `--soft-foreground` | `#5a5a5a` (mid-gray)       | `#b8b1aa` (warm taupe) | Secondary text       |
| `--accent`          | `#b5851f` (gold)           | `#f0c36a` (gold)       | Interactive emphasis (hue is decorative) |
| `--border`          | warm gray                  | cool-dark gray         | Structural separation (semantic when it clarifies layout) |
| `--danger`          | `#d25a5a`                  | `#ff8f8f`              | Destructive / risk (semantic) |

**Palettes**: `gold` (default), `ember`, `sage`, `ocean`, `sand`, `plum`. Each has light/dark variants auto-computed into `data-color` on `:root`.

**Monochrome subtrees**: a route may opt out of the palette entirely by re-aliasing both the accent family and the palette-mixed tokens (`--muted`, `--border`, `--border-soft`) at its shell selector. Do this at the scope boundary, never inside components, so the opt-out stays a one-file decision. `/mini` is the reference implementation.

### Spacing & Radius

- **Base Spacing**: `4px` (xs), `8px` (sm), `16px` (md), `24px` (lg), `32px` (xl)
- **Border Radius**: `8px` (sm), `12px` (md), `20px` (lg), `9999px` (full)
- **Standard**: `rounded-full` for pill buttons/tags, `--radius-md` for cards and panels.
- **4-point Grid Rule**: All layout spacing must land on 4px increments. Prefer `--space-*` tokens or Tailwind spacing utilities that resolve to 4px steps, and avoid fractional spacing utilities like `.5` unless there is a deliberate, documented exception.

### Icons

Icons are **minimal stroke glyphs** only. No emoji, no filled decorative pictograms, and no one-off SVGs inlined in feature components when an existing icon fits.

Prefer minimal monochrome icon controls over text labels for compact app chrome. Text remains appropriate inside forms, menus, empty states, and destructive/confirmation actions, but repeated toolbar commands should lead with the shared icon system and expose meaning through `aria-label` plus `[data-tooltip]` when needed.

**Location**: `frontend/src/lib/components/icons/` (Svelte components, one file per icon).

**Shape rules**

- `viewBox="0 0 24 24"`, `fill="none"`, `stroke="currentColor"` so color follows text (`--foreground`, `--soft-foreground`, `--accent`, etc.).
- `stroke-linecap="round"` and `stroke-linejoin="round"` for a consistent soft line look.
- Default `aria-hidden="true"`; pair with visible labels or `aria-label` on the control when meaning is not obvious from text alone.
- Optional props: `size`, `strokeWidth`, `className` / `class` (match existing components when adding new ones).

**Standard set** (reuse before adding)

| Component | Role |
| --- | --- |
| `ChevronIcon` | Disclosure, back/forward, expand/collapse (`direction`: left, right, down). |
| `CheckCircleIcon` | Read/handled state and circular completion actions. |
| `CheckIcon` | Success, selected, done. |
| `CloseIcon` | Dismiss, clear input. |
| `CopyIcon` | Copy to clipboard. |
| `ExternalLinkIcon` | Opens elsewhere / external URL. |
| `FilterIcon` | Filter or narrow list results; use for read/unread filters too. |
| `HighlighterIcon` | Highlights mode / annotation affordance. |
| `MenuIcon` | Navigation/options menu trigger; never use as the filter trigger. |
| `RefreshIcon` | Regenerate, refresh, retry. |
| `SearchIcon` | Search fields and search affordances. |
| `TrashIcon` | Delete / destructive remove. |

New icons should match this stroke style and live in the same folder so the UI stays visually one system.

---

## Component Design

### AppShell (Unified Layout)

All pages (Workspace, Queue, Highlights) must share the same `AppShell` structure:

1. **Header**: Logo (left) | Nav Pills (center) | Actions (right: Search toggle, Theme, Guide).
2. **Main Layout**: Max 3 slots: Navigation Sidebar | List Column | Detail View.
3. **Responsive**: Fixed header and bottom tab bar on mobile.

### Navigation

- **Page Nav (Header)**: Rounded pills, uppercase, tracking-wide.
- **Content Tabs**: `Toggle.svelte` (pill style). Avoid underline tabs.
- **Mobile Bottom Bar**: Shared app-level navigation for **Workspace | Queue | Highlights | Settings**.

### Shortcut Naming

- Shortcut hints must follow one clear grammar across the app.
- **Navigation** uses numbers: `Cmd/Ctrl + 1..6`.
- **Content modes / tabs** use mnemonic letters: for example `I`, `S`, `H`, `T`.
- **Inline actions** use symbols only. Do not assign letter shortcuts to action-row buttons when a symbol shortcut is available.
- Keep hint labels visually short. Prefer one-character hint chips over wordy badges.
- When adding a new shortcut, update both the visible hint and the actual keyboard handler in the same change.

---

## Interaction Model

- **Hover**: `--accent-wash` background + nudge color toward `--foreground`.
- **Active/Selected**: `--accent-soft` background + `--accent-strong` text.
- **Animations**: `fade-in` (500ms, translateY 10px → 0). Stagger increments of 80ms.
- **Tooltips**: `[data-tooltip]` attribute. 10px uppercase bold, fully opaque background. No transparency and no blur/filter effects.
- **Popups / Modals / Overlays**: Must be fully opaque surfaces. Do not use transparent backgrounds, frosted/glass effects, `backdrop-filter`, or `-webkit-backdrop-filter`.

### Opaque Overlay Rule (Strict)

- Every popup, popover, drawer, tour card, and modal must render with **opaque** colors only.
- Backdrops/scrims must also be opaque - no alpha colors (`rgba`, `/xx` opacity utility backgrounds, `transparent`, or color-mix results that introduce transparency).
- Use solid design tokens for these layers: `--surface`, `--surface-strong`, `--surface-overlay`, `--surface-overlay-strong`, `--tooltip-bg`.
- If a popup-style component needs depth, use spacing and solid tone contrast first; avoid translucency tricks.

### Overlay Layer Contract

- Do not introduce raw mobile overlay `z-index` values in feature components when the layer already belongs to the shared shell contract. Use the root overlay tokens in `frontend/src/app.css` instead.
- Treat `position: fixed` UI as **overlay-bearing**. It must not live under a transformed ancestor unless that anchoring is explicitly intended.
- If a mobile header, shell, drawer, or panel uses animation, prefer opacity-only entry animation when a descendant popup/popover/drawer must stay viewport-anchored.
- Any new mobile top-bar popup or drawer needs one Playwright assertion that tap/click makes the overlay visible above the browse/content shell.

### Filter Controls

Use `FilterIcon` for every filter trigger across the app, including compact read/unread controls such as "hide read". Do not use search icons for filters, and do not use a menu/burger icon as the direct filter trigger.

Filter triggers must behave as status indicators:

- **Idle**: minimal monochrome stroke icon with a 44x44px touch target on mobile.
- **Active**: add visible weight with a numeric badge or equivalent dot when only one filter can be active. Prefer a count badge when multiple filter dimensions can be active.
- **Focus/Press**: use the standard hover/focus background treatment without changing layout size.

Selection surfaces:

- Use a dropdown/popover for simple desktop filters with one to three groups.
- Use a drawer or sheet for complex mobile filters with many categories. Simple mobile filters may use a compact popover when it remains easy to dismiss.
- Use radio buttons or mutually exclusive menu items for one-of-many choices; use checkboxes only when multiple values can be selected at once.
- Prefer live filtering for small local lists and desktop flows. Use batch apply/reset controls only when mobile space or expensive queries make live updates costly.

Active feedback and reset:

- Never hide the fact that filters are on. Use a badge on the trigger and visible chips or concise status text near the filtered list when space allows.
- Always provide a clear/reset action for active filters.
- Empty states caused by filters must say that the current filters produced no results and provide a clear filters action.

---

## Mobile-First Patterns

### CSS Breakpoint Rule

Write base styles for mobile. Use `@media (min-width: 640px)` to add desktop enhancements. Never use `max-width` media queries for responsive layout.

### One Codebase, Two Sizes

Breakpoints change **layout**, never **behavior**. A feature must work identically on mobile and desktop or it should not ship.

- Do not branch on viewport width in JavaScript (`isMobile`, `matchMedia` feature flags, route forks). CSS owns the size story.
- Same components, same state, same event handlers at every breakpoint. Desktop is a CSS-only re-layout of the mobile tree (flex direction swap, grid row↔column, `display: none` for chrome that belongs to one size).
- When a mobile affordance (bottom bar, bottom sheet) has no desktop home, hide it with a media query and let the remaining controls (keyboard, sidebar, inline header actions) cover the intent. Do not duplicate logic into a desktop-only component.
- Axis-sensitive effects (e.g. `scrollIntoView`) read the computed CSS (flex direction, container orientation) rather than the viewport. The CSS remains the source of truth.

The mini reader is the reference implementation - see [docs/features/mini-reader.md](./docs/features/mini-reader.md).

### Bottom Bar

Primary mobile actions go in a fixed bottom bar (`position: fixed; bottom: 0`). Use `z-index: var(--z-mobile-tab-bar)` and respect `env(safe-area-inset-bottom)`. Hide on desktop with `@media (min-width: 640px) { display: none }`. All touch targets must be 44px minimum.

### Bottom Sheet

Secondary selections (channel pickers, filter groups) use an opaque bottom sheet that slides up from the bottom. The sheet has a drag handle, opaque `--surface` background, and `--surface-overlay-strong` backdrop. Max height `60dvh`. Dismiss via backdrop tap, Escape key, or explicit close button. Do not use native `<select>` dropdowns or hamburger menus for these surfaces on mobile.

### Swipe Navigation

Use the `swipeNavigation` action (`frontend/src/lib/mini/use-swipe-navigation.ts`) for horizontal swipe between content items. Default threshold: 60px. Ignores swipes starting within 40px of the left edge (iOS back gesture). Rejects diagonal swipes. Does not interfere with vertical scrolling.

### Skeleton Loading

Use content-shaped skeleton screens (matching the layout of the content being loaded) over spinner or pulse animations. Skeleton elements use `background: var(--muted)` with `animation: pulse-subtle`.

### Safe Area

All fixed-position UI (top bars, bottom bars, sheets) must respect device safe areas using `env(safe-area-inset-top)` and `env(safe-area-inset-bottom)`. Use `max()` to combine with standard padding: `padding-bottom: max(var(--space-sm), env(safe-area-inset-bottom))`.

---

## Design Debt (P0/P1)

Key priorities:

1. [ ] **Unify Shells**: Align Highlights and Queue pages with the 3-column Workspace shell.
2. [ ] **Common Mobile Nav**: Implement a single, fixed bottom tab bar across all pages.
3. [ ] **Tab Parity**: Standardize all tab-like controls to use the rounded pill style.
4. [ ] **Header Consistency**: Lock the logo/nav/actions layout across all pages.

---

## Engineering Standards

### File Limits

- Max line count per file should be **800**. If a file exceeds this, it must be modularized.
- For frontend files, **500+ lines** is already a refactor candidate even if it is still below the hard limit. Treat that as a prompt to look for natural seams before adding more code.

### Svelte State Management

- When a Svelte component or `.svelte.ts` controller exposes setter methods or action methods for reactive state, treat those methods as the only valid write path. Do not mutate the backing `$state` variable directly from alternate code paths.
- Keep side-effectful state transitions centralized. If changing a value must also sync the URL, invalidate cache, emit analytics, or notify a parent, that logic belongs in the setter/action, not in scattered direct assignments.
- Keep UI/domain state in its canonical type across the app. Only translate it to transport/API shapes at the boundary where the request is made.

### Frontend Clean Code Rules

- Keep `.svelte.ts` controllers/store modules to a single concern. If one file mixes filter state, CRUD flows, preview loading, and route sync, split those into focused modules with an explicit context or API.
- Prefer extracting render-only Svelte components before moving more behavior into state modules. If the same markup pattern appears in multiple branches, create a presentational component and pass callbacks/data in.
- In Svelte 5, prefer snippet props and `{@render ...}` over legacy `<slot>` APIs in new code. Do not introduce deprecated slot patterns during refactors.
- Do not put TypeScript type annotations or casts directly inside template event expressions when avoidable. Move non-trivial handlers into the `<script>` block and type them there.
- When a child component needs to cooperate with parent-owned focus or element refs, use an explicit prop/callback contract rather than duplicating ownership of the ref.
- Repeated UI sections should be extracted with the smallest useful surface area. Keep parent components responsible for route-specific orchestration and children responsible for rendering.
- When a component grows because it handles multiple list modes or layouts, split by mode-specific content blocks rather than keeping large `if/else` trees in one file.
- Treat duplicated state representations as a code smell. One domain concept should have one canonical representation through the UI layer.
- After refactoring large frontend files, rerun `prettier`, `svelte-check`, `eslint`, targeted unit tests, and the staged pre-commit hook before considering the cleanup verified.

### Testing

#### Two layers, two jobs

| Layer | Runner | What it proves | What it misses |
|-------|--------|---------------|----------------|
| Unit (`tests/`) | `bun test` | Logic correctness - offsets, transforms, data mutations | Whether the component actually renders the output |
| E2E (`e2e/`) | `playwright test` | Real DOM: elements present, visible, interactive | Fine-grained logic edge cases |

Neither layer substitutes for the other. The highlights regression - marks not rendering - is the canonical example: every utility function was tested, but no test verified that `<mark class="reader-highlight">` elements appeared in the article DOM.

#### When each layer is required

Write a **unit test** when:

- A pure function transforms, filters, or maps data (offsets, ranges, merging, sorting)
- A bug was caused by incorrect logic - pin the input/output contract

Write an **E2E test** when:

- A feature is visible in the DOM: an element appears, disappears, or changes state
- A data-to-DOM pipeline exists: server data → component prop → rendered element
- A regression was a rendering/wiring failure - the element was absent or wrong

#### Rendering regression rule

Any feature whose correctness is observable in the DOM must have at least one E2E assertion that checks for that element.

Examples:

- Highlights → assert `mark.reader-highlight` is visible inside the article
- Sidebar counts → assert the count badge text matches data
- Floating toolbar → assert the action container appears on text selection

When fixing a rendering bug, add the E2E test first so it fails before the fix, then fix, then confirm it passes.

#### Responsive regression rule

Mobile-first + media queries means a base rule applies at every width until a breakpoint overrides it. Editing base CSS can silently break desktop (or vice versa). The defense is a screenshot assertion at each breakpoint.

Any route or component with a desktop re-layout (a `@media (min-width: 640px)` or `@media (min-width: 960px)` block that changes structure) must have a Playwright spec that:

- Renders the page at a mobile viewport (e.g. `375 × 812`) and asserts a defining element is visible.
- Renders the page at a desktop viewport (e.g. `1280 × 900`) and asserts the desktop-only element is visible (and/or the mobile-only chrome is hidden).

Example checks for the mini reader: bottom bar visible at 375px; hidden at 1280px. Summary strip flex-row at 375px; flex-column sidebar at 1280px. Desktop sidebar is scrollable (internal scroll, not page scroll).

When a CSS change touches a breakpoint block, run the responsive spec before committing. If you zero a base padding, remove a width, or change a flex direction — assume you broke the other breakpoint until the spec says otherwise.

#### Running tests locally

```bash
# Unit tests
cd frontend && bun test tests

# E2E (requires running app on port 3543)
cd frontend && bunx playwright test

# E2E headed (watch it run)
cd frontend && bunx playwright test --headed
```
