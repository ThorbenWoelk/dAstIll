# Mini Reader

`/mini` is the project's reference implementation for text-forward reading. It is also the reference implementation for two architectural rules from [DESIGN.md](https://github.com/ThorbenWoelk/dAstIll/blob/main/design.md):

1. Breakpoints change **layout**, never **behavior**.
2. A route may opt out of the palette by re-aliasing tokens at its shell boundary.

This page explains how those rules land in code so other routes can follow the same pattern.

## Single tree, two shapes

The whole surface is one Svelte tree:

```
mini-shell
├── MiniTopBar
├── mini-main
│   ├── MiniSummaryStrip        (hidden when there is no active summary)
│   └── mini-article-pane       (scrolls; holds MiniArticle OR MiniEmptyState)
├── MiniBottomBar               (position: fixed)
└── MiniChannelSheet            (position: fixed, conditionally open)
```

There is no `isDesktop` flag, no `matchMedia` subscription, no branch that renders a different tree on desktop. The same state object (`MiniReaderState`), the same handlers, and the same components are mounted at every viewport size.

### What changes at `@media (min-width: 960px)`

- `.mini-main` flips from `flex-direction: column` (strip on top, pane below) to `flex-direction: row` (strip on left, pane on right).
- `MiniSummaryStrip` flips from a horizontal card rail to a vertical sidebar - `flex-direction: column`, vertical overflow, full-width cards, a right-edge hairline.
- `MiniBottomBar` sets `display: none`. Its intents are covered by keyboard shortcuts (arrows / `j` / `k` / `r`), by clicking items in the now-visible rail, and by the channel picker that lives in the top bar on desktop.
- `MiniChannelSheet` repositions from a bottom-anchored sheet to a centered modal (same markup, same logic, different `align-items` + `border-radius` + animation).
- `MiniTopBar` reveals a channel trigger that was always in the DOM but `display: none` on mobile.
- `MiniArticle` widens its max-width and drops the bottom padding that previously reserved space for the fixed mobile bar.

### The one axis-sensitive effect

`MiniSummaryStrip` calls `scrollIntoView` to keep the active card visible. Horizontal and vertical rails need different axis hints. The component does not ask the viewport - it reads the rail's own computed `flex-direction`:

```ts
const vertical = getComputedStyle(stripRef).flexDirection === "column";
el.scrollIntoView({
  behavior: "smooth",
  block: vertical ? "center" : "nearest",
  inline: vertical ? "nearest" : "center",
});
```

CSS remains the source of truth for orientation. The effect reads it, never replicates it.

### Pull refresh belongs to the scroll pane

The page shell is fixed-height so the top bar, summary strip, and bottom bar stay steady while reading. That means browser-native pull-to-refresh is not reliable on mobile: the article pane, not the document, owns vertical scroll.

`use-pull-refresh.ts` handles a downward touch gesture only when `.mini-article-pane` is already at `scrollTop === 0`. It ignores interactive targets, rejects horizontal gestures, and calls `MiniReaderState.refreshReader()` with a cache bypass so the reload fetches fresh `/api/mini` data.

## Monochrome by token override

The rest of the app has a palette system (`ember`, `sage`, `ocean`, `sand`, `plum`) swapped via `data-color` on `:root`. The mini reader is deliberately palette-proof: its visual identity is exactly one pencil, on every device, in every theme.

This is done with a single CSS block on the shell, not with component edits:

```css
.mini-shell {
  --accent: var(--foreground);
  --accent-strong: var(--foreground);
  --accent-soft: color-mix(in srgb, var(--foreground) 8%, var(--surface));
  --accent-wash: color-mix(in srgb, var(--foreground) 6%, var(--surface));
  --accent-wash-strong: color-mix(
    in srgb,
    var(--foreground) 12%,
    var(--surface)
  );
  --muted: color-mix(in srgb, var(--foreground) 7%, var(--background));
  --border: color-mix(in srgb, var(--foreground) 18%, var(--background));
  --border-soft: color-mix(in srgb, var(--foreground) 9%, var(--background));
}
```

The accent family is neutralized to foreground tones. The palette-mixed tokens (`--muted`, `--border`, `--border-soft`) are redefined against `--foreground` and `--background` instead of `--accent-soft`, because their default definitions carry an accent tint that would otherwise leak in even after neutralizing accent itself.

Components inside the shell keep using `--accent*` and `--muted` normally. They do not know they are inside a monochrome scope. Mixing this pattern into another route is a one-block change on that route's shell.

## Rules of thumb

- Keep `/mini` as small as possible. It is a reading surface, not a second
  workspace.
- Add a feature only when it does not increase maintenance effort, or when it can
  fail without disturbing the core loop: pick channel, read summary, mark read,
  move on.
- If a feature would only be usable on mobile or only on desktop, it is not a feature for `/mini`. Either generalize it, or move it elsewhere.
- If a CSS-only re-layout is not enough to make a screen feel right on desktop, the information architecture is probably wrong - revisit it before reaching for JS branching.
- Keep state in `$lib/mini/*.svelte.ts` controllers, not in the page. The page orchestrates; it does not own state.
- Keep accent references in component CSS even inside the monochrome subtree. The scope does the neutralizing; components stay portable.
