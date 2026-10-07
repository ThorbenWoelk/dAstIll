# dAstIll Design System: Edition

This file (`design.md`) is the source of truth for the frontend design system and frontend engineering standards.
Do not duplicate these rules in `AGENTS.md`; link here from there instead.

## Philosophy

The reader is a calm morning paper. One story at a time, newest first. When a story is read, it leaves the page and the next one takes its place.

- **Content first.** The headline, standfirst, and body carry the page. Chrome stays small and quiet.
- **Newspaper structure, not app chrome.** Separate things with rules (lines), whitespace, and type. No cards, shadows, rounded corners, or gradients.
- **One primary action per screen.** On the reader that is "Mark as read". Everything else is a quiet text link.
- **No emojis, no decorative icons.** Icons are minimal stroke glyphs and appear only where a word would be clumsy (refresh, external link, check).
- **Fewer requests is a feature.** The UI never polls, streams, or sends analytics. See [Running Cost For One Reader](./docs/operations/deployment.md#running-cost-for-one-reader).

---

## Visual Atoms

### Typography

Fonts are self-hosted through `@fontsource` packages and imported in `src/routes/+layout.svelte`.

| Role | Font | Use |
| --- | --- | --- |
| Serif (`--serif`) | Libre Caslon Text 400, 400 italic, 700 | Masthead title, headlines, standfirst (italic), body |
| Sans (`--sans`) | Libre Franklin 400, 600, 700 | Dateline, kickers, labels, bylines, buttons, the "At a glance" list |

- **Labels** (`.label`): sans, 11px, weight 700, uppercase, `letter-spacing: 0.12em`. Used for kickers, section names, and box headings.
- **Headlines**: serif 700, 32px on phones, 48px from 640px, tight leading (`1.05`-`1.1`), `text-wrap: balance`.
- **Standfirst**: serif italic, 19px on phones, 22px from 640px, `--ink-soft`.
- **Body**: serif 17px, `line-height: 1.65`, left-aligned, `hyphens: auto`. Do not justify; narrow columns make rivers.
- Body section headings (`## Key Points`) render as small uppercase sans labels, not large serif headings.

### Color

All colors are CSS custom properties on `:root` in `src/app.css`. Never hardcode a hex value in a component.

| Token | Newsprint (light) | Night edition (dark) | Role |
| --- | --- | --- | --- |
| `--paper` | `#f1ece1` | `#161513` | Page background |
| `--paper-raised` | `#f7f3ea` | `#1e1c19` | Inputs and notices |
| `--ink` | `#171513` | `#ece6da` | Primary text |
| `--ink-soft` | `#57524a` | `#b9b1a4` | Standfirst, secondary text |
| `--ink-faint` | `#6b655c` | `#9a9286` | Counts and meta lines |
| `--rule` | `#171513` | `#ece6da` | Structural rules (2px under the masthead, above boxes) |
| `--hairline` | `#d6cfc1` | `#38342e` | Light separators between list items and columns |
| `--kicker` | `#a23b2c` | `#e88a72` | Channel kicker, active section, focus ring |
| `--press` / `--press-ink` | ink on paper | paper on ink | The filled primary button |
| `--danger` | same as kicker | same as kicker | Destructive confirmations and errors |
| `--marker` | `#ecd78f` | `#5a4a22` | Highlighted passages, like a highlighter pen |

- Dark mode follows `prefers-color-scheme`. There is no in-app theme switch.
- Every text color must reach WCAG AA (4.5:1) on `--paper`. Check new tokens before adding them.
- The kicker red is the only accent. The marker yellow is only for highlights. Do not introduce another hue.

### Spacing

- 4px grid: `--space-1` (4) through `--space-8` (64). Use the tokens, not raw pixel values.
- Page column: `--column` (1120px), side padding `--space-4` on phones and `--space-7` from 640px.
- Touch targets are at least `--touch` (44px). The primary button is 52px tall on phones.
- `--tab-bar` (48px) is the phone tab bar's height without the safe area. `--tuck-ms` (200ms) is how fast the bars tuck away and come back.

### Rules and boxes

- 2px `--rule` under the masthead, above the "At a glance" box, and above "Also in this edition".
- 1px `--rule` under the section bar, above the phone read bar, and above the phone tab bar.
- 1px `--hairline` between list items, around the byline, and between body columns.
- No `border-radius`, no `box-shadow`, no translucent overlays.

### Icons

Minimal stroke glyphs in `src/lib/components/icons/`: `viewBox="0 0 24 24"`, `fill="none"`, `stroke="currentColor"`, round caps and joins, `aria-hidden="true"`. Icon-only buttons need an `aria-label` and a `title`.

Current set: `CheckIcon`, `RefreshIcon`, `ExternalLinkIcon`, `HighlightIcon`. Reuse before adding.

### App icon

Source: [`assets/dastill-icon.svg`](./assets/dastill-icon.svg). A kicker-red play button over a heavy ink rule and two `--ink-soft` summary lines, on a square `--paper` sheet. Sharp corners, no circle, no gradient. The platform rounds the corners where it wants to.

- Favicons (16 and 32px) drop the summary lines and keep only the play button and the rule, so they stay readable.
- `frontend/static/icon-512.png` feeds [`scripts/sync_tauri_icons.sh`](./scripts/sync_tauri_icons.sh) for the desktop and Android icons.

---

## Components

| Component | Job |
| --- | --- |
| `Masthead` | Dateline (long date from 640px, short date on phones) with status and actions, then the `dAstIll` title and the 2px rule. Every screen uses it. The page links (Front page, Highlights, Finished, Sections) are one `nav`: a tab bar fixed along the bottom on phones, quiet links in the dateline from 640px. The current page uses `aria-current="page"`: kicker color and a 2px kicker rule in the tab bar, bold in the dateline. The front page hides its own dateline link. The optional nav slot renders after the header so it can stick. |
| `SectionNav` | "Front page" plus one button per channel with unread counts. On phones it runs full width, scrolls sideways, keeps the active section in view, and sticks to the top of the screen. Wraps and centers from 640px, not sticky. The active section uses `aria-current="page"` and the kicker color. |
| `StoryPager` | "‹ Previous", "2 of 14", "Next ›" above the lead story. Moves through the section without marking anything read. A turn that is not possible is hidden but keeps its place. |
| `StoryArticle` | Kicker, headline, standfirst, byline, "At a glance" box, and body. Body is one column on phones and two columns from 960px. `entrance` says how a new story arrives: printed in place (`.print-in`) or turned in from the side it came from (`.turn-in-next`, `.turn-in-previous`). Given `onHighlight`, it shows a Highlight button under a text selection and marks saved highlights in `--marker`. Tapping a mark offers "Remove highlight". |
| `ReadBar` | "Mark as read" and, after a read, an "Undo" button. Sticky on phones, just above the tab bar, and at the screen edge while the tab bar is tucked away. Inline after the article from 960px. |
| `AlsoInEdition` | The next six stories in the section. Each one is a button that makes it the lead. Below the article on phones, a sticky right column from 960px. |
| `EditionNotice` | A non-blocking message (a failed request) with Dismiss. |
| `SignIn` | The signed-out page under the masthead. |

Shared button styles live in `src/app.css`: `.press` (filled primary) and `.text-button` (underlined quiet link). Components add layout only.

### Summary layout

`src/lib/edition/summary.ts` maps the generated summary format onto the page:

- `## Overview` becomes the standfirst. A long overview (over 400 characters) stays in the body and only its first sentence leads.
- `## At a glance` becomes the boxed list.
- Every other section is the body, in order.
- Summaries in another shape render whole as the body.

Summary markdown is model output. Always render it through `renderMarkdown` (`marked` plus DOMPurify). It fails closed to plain text when the sanitizer is unavailable.

---

## Interaction

- **Mark as read** is optimistic: the story leaves at once, then the request is sent. A failure restores it and shows a notice.
- The story after the finished one takes its place, so a reader who browsed to story 5 stays at story 5. After the last story in a section, the one before it leads.
- **Undo** restores the last finished story as the lead. Undo requests wait for the read request, so they never race.
- Keyboard: `R` marks the lead story read, `U` undoes, `→` or `J` turns to the next story, `←` or `K` to the previous one. Keys are ignored while typing in a field or with a modifier held.
- After reading, undoing, picking, or turning to a story, or choosing a section, the page scrolls to the top.
- The newly printed story fades and rises 6px (`.print-in`, 360ms). A story turned to slides in 40px from its side (240ms). Respect `prefers-reduced-motion`.

### Moving between stories

- The lead story is one page in its section, newest first. **Next** is the older story after it, **Previous** the newer one before it. Turning never marks anything read.
- Three ways to turn, at every size: a sideways swipe, the `StoryPager` buttons, and the arrow keys. The pager is also the accessible alternative to the swipe.
- **Swipe** (`src/lib/navigation/story-swipe.ts`, decisions in `swipe.ts`): touch only, so mouse drags still select text. The story follows the finger. A swipe past a quarter of the width (at most 120px) or a quick flick turns. With nothing that way, the story resists and settles back. Vertical scrolling and pinch zoom stay with the browser (`touch-action: pan-y pinch-zoom`).
- A swipe does not start within 24px of a screen edge (the system back gesture), while text is selected, or inside something that scrolls sideways.
- A single story page (`/stories/[id]`) has a "‹ Finished" (or "‹ Highlights", "‹ Back") control above the story, and a swipe to the right goes back too. Installed on a phone, the paper has no browser back button. Opened directly, the control links to the front page.
- Loading: show the stored edition at once. Without one, show a short italic line ("Printing today's edition…"), not a spinner.
- Empty and error states are centered italic text with one clear next action.

### Highlights

- Selecting text in a story shows a **Highlight** button just under the selection. It sits below, not above, so it never covers the phone's own copy menu.
- Highlights cover the standfirst, the "At a glance" box, and the body. The headline, byline, and buttons are marked `data-passage-skip` and are left out.
- A highlight is saved as its text plus up to 80 characters of context on each side (`src/lib/highlights/anchoring.ts`). It finds its place again by text, ignoring whitespace differences, and uses the context to pick between repeats. A highlight that no longer matches is simply not shown.
- Adding and removing are optimistic and roll back with a notice on failure.
- Highlight ids are strings. They are larger than JavaScript's safe integers.

---

## Mobile-First Patterns

### Breakpoints

Write base styles for phones. Add layout with `@media (min-width: 640px)` and `@media (min-width: 960px)`. Never use `max-width` media queries for layout.

| Width | Changes |
| --- | --- |
| base | One column. Tab bar along the bottom. Sticky section bar and read bar, both tucking with the tab bar. Rail below the article. Short dateline. |
| 640px | Page links in the dateline, no tab bar. Long dateline, larger title and headline, centered section bar that scrolls with the page, two-column glance list. |
| 960px | Rail beside the article, two-column body, read bar inline. |

### One codebase, two sizes

Breakpoints change layout, never behavior. Do not branch on viewport width in JavaScript. The same components, state, and handlers run at every size. CSS owns the size story.

### Tab bar and tucking

- Phones navigate with the thumb. The page links sit in a tab bar along the bottom. The section bar sticks to the top, and the read bar sits just above the tab bar.
- While the reader scrolls down, the tab bar and the section bar tuck away so the story has the screen. Scrolling up 16px, reaching the top 240px or the end of the page, keyboard focus in a bar, or moving to another page brings them back. The read bar never tucks.
- `src/lib/navigation/tuck.ts` follows the scroll at every size. The layout sets `--tucked` (0 or 1) and `--tab-bar-offset` on `.page`. Components read those variables, and their own CSS decides what tucks at which size.
- The page reserves the tab bar's height at the bottom, so tucking never shifts the content.
- `.page` clips sideways overflow (`overflow-x: clip`, which keeps sticky working), so a story swiped sideways never widens the page.

### Safe area

Fixed or sticky UI respects `env(safe-area-inset-top)` and `env(safe-area-inset-bottom)` with `max()`. The tab bar pads its bottom with the safe area; the read bar takes the safe area over while the tab bar is tucked away.

---

## Engineering Standards

### File limits

- Hard limit: 800 lines per file. Frontend files over 500 lines are a refactor candidate.

### State

- Reader state lives in `EditionReader` (`src/lib/edition/reader.svelte.ts`). Its methods are the only write path; components read fields and derived values.
- Keep pure logic in plain `.ts` modules (`stories.ts`, `summary.ts`, `printing.ts`, `navigation/swipe.ts`, `navigation/tuck.ts`) so `bun test` can cover it without the Svelte compiler.
- When a function returns `$state` or `$derived`, wrap it in getters and setters to keep reactivity.
- Keep UI types (`Story`, `Section`) separate from transport types (`MiniSummaryItem`). Convert at the API boundary.

### Requests

- Every backend call goes through `src/lib/api.ts`.
- No polling, no `EventSource`, no analytics. Reload on demand or when the tab returns after 30 minutes.
- Fan-out requests run at most four at a time.

### Svelte

- Use snippet props and `{@render}`; no legacy slots.
- DOM behavior that belongs to an element (like the story swipe) is an attachment (`{@attach}`), not a legacy action. Pass callbacks that read state when called, so the attachment is not torn down on every state change.
- Keep type annotations out of template expressions; move handlers into `<script>`.
- Extract a presentational component before adding branches to a large one.

---

## Testing

### Two layers, two jobs

| Layer | Runner | Proves |
| --- | --- | --- |
| Unit (`frontend/tests/`) | `bun test` | Pure logic: ordering, sections, summary splitting, request shape, error messages |
| E2E (`frontend/e2e/`) | `playwright test` | Real DOM: what renders, what disappears, layout at each breakpoint, sanitizing, swipes and tucking |

E2E specs mock the backend with `page.route` (`e2e/mock-backend.ts`) and sign in with the dev-only local session. Playwright starts the frontend dev server when nothing is running on port 3543. `e2e/navigation.spec.ts` swipes by dispatching touch `pointerdown`/`pointermove`/`pointerup` events, and scrolls in small steps so tucking sees a reader scrolling rather than a jump.

### When each layer is required

- A pure function that sorts, filters, maps, or parses data needs a unit test.
- Anything observable in the DOM needs at least one E2E assertion on that element.
- Anything that depends on a real DOM (for example HTML sanitizing) is tested in E2E, not with a simulated DOM.
- A layout that changes at a breakpoint needs an E2E check at a phone viewport (375 x 812) and a desktop viewport (1280 x 900).

### Running tests

```bash
cd frontend
bun run test        # unit
bun run test:e2e    # E2E (set PLAYWRIGHT_CHROMIUM_PATH to use a preinstalled Chromium)
```
