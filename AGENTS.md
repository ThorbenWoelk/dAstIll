# Design & Style Guide

## Philosophy

Muted, zen, minimalistic. No decorative chrome. Let content breathe. Prefer restraint over expressiveness - if something can be removed, remove it. No emojis anywhere.

**Borders and boxes**: use as little as possible. Prefer whitespace, typography weight, and color contrast to create separation and hierarchy. Borders and background boxes around text or UI elements should be a last resort - only when there is no other way to communicate structure or state.

## Typography

- Body: **Manrope** (system-ui fallback), `-webkit-font-smoothing: antialiased`
- Headings / serif moments: **Fraunces** (`font-variation-settings: "opsz" 72`, `letter-spacing: -0.02em`, `font-weight: 600`)
- UI labels, tabs, tooltips: uppercase, `font-weight: 700`, `letter-spacing: 0.05-0.08em`, `font-size: 10-11px`
- Logo wordmark: `font-bold tracking-tighter` — A and I rendered in `--soft-foreground`, rest in `--color-swatch`

## Color System

All colors are CSS custom properties. Never hardcode hex values in components — always use variables.

### Core tokens (light / dark)

| Token               | Light                      | Dark                   |
| ------------------- | -------------------------- | ---------------------- |
| `--background`      | `#faf9f6` (warm off-white) | `#111315` (near-black) |
| `--foreground`      | `#1a1a1a`                  | `#f4efe9` (warm white) |
| `--surface`         | `#ffffff`                  | `#181b1f`              |
| `--surface-strong`  | `#ffffff`                  | `#1d2126`              |
| `--soft-foreground` | `#5a5a5a` (mid-gray)       | `#b8b1aa` (warm taupe) |
| `--muted`           | warm gray wash             | dark gray wash         |
| `--border`          | warm gray                  | cool-dark gray         |
| `--border-soft`     | softer warm gray           | softer dark gray       |
| `--danger`          | `#d25a5a`                  | `#ff8f8f`              |

### Accent / palette (swappable)

The accent color is set by `data-color` on `:root`. Five palettes, each with light + dark variants:

| Palette           | Light accent | Dark accent |
| ----------------- | ------------ | ----------- |
| `ember` (default) | `#d33c2a`    | `#ff8e79`   |
| `sage`            | `#4a8a5c`    | `#7ac88e`   |
| `ocean`           | `#2a7ab5`    | `#6db8e8`   |
| `sand`            | `#a68a5b`    | `#d4b882`   |
| `plum`            | `#8b5cb4`    | `#c49aeb`   |

Derived accent tokens (auto-computed via `color-mix`): `--accent-soft`, `--accent-strong`, `--accent-wash`, `--accent-wash-strong`, `--accent-border-soft`, `--panel-surface`, `--color-swatch`.

### Usage rules

- **Primary text**: `--foreground`
- **Secondary / supporting text**: `--soft-foreground`
- **Interactive accent**: `--accent` (links, active states, icons)
- **Subtle accent backgrounds**: `--accent-wash`, `--accent-soft`
- **Borders**: available (`--border`, `--border-soft`, `--accent-border-soft`) but use sparingly - prefer spacing and surface contrast over divider lines
- **Panels / cards**: `--surface`, `--panel-surface` - avoid adding a border on top of a surface background; the surface color already creates separation
- **Never** use raw `--background` as a surface — it's for the page shell only

## Spacing & Radius

```
--space-xs: 4px   --radius-sm: 8px
--space-sm: 8px   --radius-md: 12px
--space-md: 16px  --radius-lg: 20px
--space-lg: 24px  --radius-full: 9999px
--space-xl: 32px
```

Prefer `rounded-full` for pill buttons/tags, `--radius-md` for cards and panels.
All layout spacing should sit on a 4px grid. Prefer `--space-*` tokens or Tailwind spacing utilities that resolve to 4px steps, and avoid fractional `.5` spacing utilities unless the exception is intentional.

## Interactive States

- Hover: `--accent-wash` background + nudge toward `--foreground`
- Active/selected: `--accent-soft` background + `--accent-strong` text + `--accent`/25 border
- Focus ring: `ring-2 ring-[var(--accent)]/40`, `ring-offset` uses `--background`
- Transitions: `200ms`, easing `cubic-bezier(0.16, 1, 0.3, 1)`
- Disabled: `opacity-50`, no pointer events

## Buttons

- Ghost/icon: `rounded-full`, `h-8 w-8`, `--soft-foreground` color, hover `--accent-wash` background - no border
- Pill/label: `rounded-full`, borderless by default; add `--accent-border-soft` border only when needed for legibility against ambiguous backgrounds; uppercase 11px bold tracking
- Destructive: `--danger` family tokens only
- Never add a border to a button just for decoration

## Surfaces & Depth

- Body: radial gradient from `--shell-gradient-start` → `--shell-gradient-end` → `--background`
- Subtle noise grain texture via `body::before` SVG filter
- Frosted glass: `backdrop-filter: blur(10px)` + `--surface-frost` background
- Shadows: `--shadow-soft` (soft lift), `--shadow-strong` (modals/popovers)

## Animation

- Entrance: `fade-in` (500ms, translateY 10px → 0, ease-out spring)
- Stagger classes: `.stagger-1` (80ms), `.stagger-2` (160ms), `.stagger-3` (240ms)
- Respect `prefers-reduced-motion` — all durations collapse to 0.01ms

## Scrollbars

Thin (4px), `--border` thumb, transparent track, `--soft-foreground` on hover.

## Tooltips

`[data-tooltip]` attribute. 10px uppercase bold, `--tooltip-bg` (dark glass) with blur, hidden on touch devices.

---

# Dev Tools

## Log Inspection

- **Logfire**: `logfire logs` - inspect application traces and structured logs
- **GCloud**: `gcloud logging read` - inspect Cloud Run deployment logs
- **GitHub Actions**: `gh run list` / `gh run view <id>` - inspect CI/CD pipeline logs

## Validation Execution

- Do not chain formatting, typecheck, tests, and build into one long `&&` command.
- Run validators step-by-step as separate commands and report progress after each step so long-running checks do not appear stuck and can be interrupted safely.
