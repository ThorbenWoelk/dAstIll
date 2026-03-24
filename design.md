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

| Token | Light Value | Dark Value | Role |
|---|---|---|---|
| `--background` | `#faf9f6` (warm off-white) | `#111315` (near-black) | Page shell only |
| `--foreground` | `#1a1a1a` | `#f4efe9` (warm white) | Primary text |
| `--surface` | `#ffffff` | `#181b1f` | Panels, cards |
| `--soft-foreground` | `#5a5a5a` (mid-gray) | `#b8b1aa` (warm taupe) | Secondary text |
| `--accent` | `#d33c2a` (ember) | `#ff8e79` (ember) | Active states, icons |
| `--border` | warm gray | cool-dark gray | Structure borders |
| `--danger` | `#d25a5a` | `#ff8f8f` | Destructive actions |

**Palettes**: `ember` (default), `sage`, `ocean`, `sand`, `plum`. Each has light/dark variants auto-computed into `data-color` on `:root`.

### Spacing & Radius
- **Base Spacing**: `4px` (xs), `8px` (sm), `16px` (md), `24px` (lg), `32px` (xl)
- **Border Radius**: `8px` (sm), `12px` (md), `20px` (lg), `9999px` (full)
- **Standard**: `rounded-full` for pill buttons/tags, `--radius-md` for cards and panels.

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

---

## Design Debt (P0/P1)
Refer to [ux-visual-audit.md](file:///Users/thorben.woelk/repos/dAstIll/specs/ux-visual-audit.md) for detailed fixes. Key priorities:
1. [ ] **Unify Shells**: Align Highlights and Queue pages with the 3-column Workspace shell.
2. [ ] **Common Mobile Nav**: Implement a single, fixed bottom tab bar across all pages.
3. [ ] **Tab Parity**: Standardize all tab-like controls to use the rounded pill style.
4. [ ] **Header Consistency**: Lock the logo/nav/actions layout across all pages.
