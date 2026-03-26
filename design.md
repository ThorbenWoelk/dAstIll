# dAstIll Design System

## Philosophy

Muted, zen, minimalistic. Content first, no decorative chrome. Prefer restraint over expressiveness - if something can be removed, remove it. No emojis anywhere.

**Borders and boxes**: Use as little as possible. Prefer whitespace, typography weight, and color contrast to create separation and hierarchy. Borders and background boxes around text or UI elements should be a last resort.

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
  - **Palettes** (`ember`, `sage`, `ocean`, `sand`, `plum`): decorative swaps for the accent system.
  - **Atmosphere**: shell gradients, subtle washes, and logo wordmark treatment use decorative contrast; they must not be the only cue for errors, success, or destructive actions.

| Token               | Light Value                | Dark Value             | Role                 |
| ------------------- | -------------------------- | ---------------------- | -------------------- |
| `--background`      | `#faf9f6` (warm off-white) | `#111315` (near-black) | Page shell only      |
| `--foreground`      | `#1a1a1a`                  | `#f4efe9` (warm white) | Primary text         |
| `--surface`         | `#ffffff`                  | `#181b1f`              | Panels, cards        |
| `--soft-foreground` | `#5a5a5a` (mid-gray)       | `#b8b1aa` (warm taupe) | Secondary text       |
| `--accent`          | `#d33c2a` (ember)          | `#ff8e79` (ember)      | Interactive emphasis (hue is decorative) |
| `--border`          | warm gray                  | cool-dark gray         | Structural separation (semantic when it clarifies layout) |
| `--danger`          | `#d25a5a`                  | `#ff8f8f`              | Destructive / risk (semantic) |

**Palettes**: `ember` (default), `sage`, `ocean`, `sand`, `plum`. Each has light/dark variants auto-computed into `data-color` on `:root`.

### Spacing & Radius

- **Base Spacing**: `4px` (xs), `8px` (sm), `16px` (md), `24px` (lg), `32px` (xl)
- **Border Radius**: `8px` (sm), `12px` (md), `20px` (lg), `9999px` (full)
- **Standard**: `rounded-full` for pill buttons/tags, `--radius-md` for cards and panels.
- **4-point Grid Rule**: All layout spacing must land on 4px increments. Prefer `--space-*` tokens or Tailwind spacing utilities that resolve to 4px steps, and avoid fractional spacing utilities like `.5` unless there is a deliberate, documented exception.

### Icons

Icons are **minimal stroke glyphs** only. No emoji, no filled decorative pictograms, and no one-off SVGs inlined in feature components when an existing icon fits.

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
| `CheckIcon` | Success, selected, done. |
| `CloseIcon` | Dismiss, clear input. |
| `CopyIcon` | Copy to clipboard. |
| `ExternalLinkIcon` | Opens elsewhere / external URL. |
| `HighlighterIcon` | Highlights mode / annotation affordance. |
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

---

## Design Debt (P0/P1)

Refer to [ux-visual-audit.md](file:///Users/thorben.woelk/repos/dAstIll/specs/ux-visual-audit.md) for detailed fixes. Key priorities:

1. [ ] **Unify Shells**: Align Highlights and Queue pages with the 3-column Workspace shell.
2. [ ] **Common Mobile Nav**: Implement a single, fixed bottom tab bar across all pages.
3. [ ] **Tab Parity**: Standardize all tab-like controls to use the rounded pill style.
4. [ ] **Header Consistency**: Lock the logo/nav/actions layout across all pages.
