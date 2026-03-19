# dAstIll UX Audit & Improvement Specs

Comprehensive audit across all pages (workspace, download-queue, highlights) on both desktop (>=1024px) and mobile (<1024px). Each finding includes the problem, affected components, and a concrete fix proposal.

---

## 1. Structural / Layout Issues

### 1.1 Monolith Page Component

**Problem:** `+page.svelte` is 1,800 lines with ~60 state variables, business logic, formatting orchestration, and template code all in one file. This makes reasoning about layout behavior extremely hard and leads to the "Frankenstein" feel because visual changes require wading through business logic.

**Fix:**
- Extract the remaining inline logic into dedicated stores/modules (e.g., `formatting-orchestrator.ts`, `content-loader.ts`, `video-selection.ts`).
- The page file should be <300 lines: import components, wire props, done.
- This is the single biggest enabler for all visual fixes below because it lets designers reason about layout in isolation.

### 1.2 Three-Column Grid Has No Defined Gutter or Internal Rhythm

**Problem:** The workspace grid `lg:grid-cols-[260px_300px_minmax(0,1fr)]` uses `lg:gap-0`, then each column applies its own inconsistent padding (`pl-2 pr-5`, `px-5`, `pl-6`). This causes:
- Channel sidebar left edge is indented differently from the video sidebar.
- Content panel left padding jumps by 6px compared to video sidebar right padding.
- On resize, columns feel like they shift independently.

**Fix:**
- Define a single layout primitive:
  ```
  lg:grid-cols-[260px_300px_minmax(0,1fr)] lg:gap-px
  ```
  with a consistent `px-4` inside each column (or `px-5` uniformly).
- Remove all per-column padding overrides. Use the grid gap as the only inter-column spacing.
- Wrap columns in a shared `divide-x divide-[var(--border-soft)]` instead of per-column `border-r` to guarantee alignment.

### 1.3 Inconsistent Sticky Column Heights

**Problem:** Both sidebars and the content panel use `lg:sticky lg:top-4 lg:h-[calc(100vh-4rem)]` but the header height is not a CSS variable. If the header wraps (e.g., long search bar on medium screens), content is clipped behind it.

**Fix:**
- Introduce `--header-height` measured dynamically (ResizeObserver on the header) or set to a fixed value.
- Change column heights to `lg:h-[calc(100dvh-var(--header-height)-var(--footer-height))]`.
- Use `dvh` for more reliable mobile viewport handling.

---

## 2. Header & Navigation

### 2.1 Header Wraps Awkwardly Between Breakpoints (768–1024px)

**Problem:** Between `sm` and `lg`, the header uses `flex-wrap` with `items-start` (mobile) vs `items-center` (desktop). At tablet widths (~800px), the search bar drops to a new row but the top row (logo + nav pills + theme toggle) becomes cramped. Logo, AI dot, guide icon, theme toggle, and section nav all fight for horizontal space.

**Fix:**
- At the `md` breakpoint, collapse SectionNavigation into its mobile dropdown mode (it already has this capability via `mobileMode="inline"`).
- Give the search bar `order-first w-full` on tablet so it sits above the brand row, reducing the cramped feel.

### 2.2 Duplicate Font Loading

**Problem:** `app.html` loads `Inter + Newsreader` via Google Fonts; `+layout.svelte` loads `Fraunces + Manrope`. The app only uses Fraunces and Manrope. The Inter/Newsreader request is a leftover that adds ~120KB of wasted fonts and a render-blocking request.

**Fix:**
- Remove the `Inter + Newsreader` `<link>` from `app.html`.
- Move the `Fraunces + Manrope` load into `app.html` (earlier in the document, with `<link rel="preload">`) so the layout shift is minimized.

### 2.3 "DASTILL" Brand Mark Feels Disconnected

**Problem:** The logo is plain bold uppercase text (`tracking-tighter`) with no visual weight to anchor the header. On mobile it's pushed to the far left with only a tiny 2.5px AI dot next to it. The overall effect is that the brand feels like an afterthought.

**Fix:**
- Add a small logomark (e.g., a stylized distillation flask icon, 24x24) to the left of the text. This creates an anchor point and makes the header feel intentional.
- Alternatively, set the brand in Fraunces (the serif font already loaded) to give it more personality and contrast with the UI font.

### 2.4 Search Bar Visual Weight is Disproportionate

**Problem:** On desktop, the search bar is 27rem wide and is the most visually prominent element in the header, yet search is a secondary action (the primary flow is channel -> video -> content). The `ask` badge, info icon, and spinner make it even heavier.

**Fix:**
- Reduce desktop width to `lg:w-[22rem]`.
- Replace the pill-shaped search bar with a more understated flat input that expands on focus (like Notion's search).
- Remove the always-visible `ask` badge; show it only after the user starts typing.

---

## 3. Channel Sidebar

### 3.1 Toolbar Icons Are Too Dense

**Problem:** The channel sidebar header packs "Channels" label + manage (trash) + search + sort into a tight row. All three icons are 7x7 circles with 13px SVGs. They look like a cluster of undifferentiated dots, especially on mobile where the sidebar gets full-width but the toolbar stays tiny.

**Fix:**
- Group the action icons into a single overflow menu (three-dot icon) that reveals manage/search/sort options. This reduces the header to: `Channels` label + overflow menu.
- On desktop with enough room, keep search and sort visible but hide manage behind the overflow.

### 3.2 Channel Input "Follow a channel..." Lacks Affordance

**Problem:** The input has no visible border or background -- just an underline. Users can easily miss that it's interactive. The `+` submit button appears disabled (gray circle) until text is entered, which makes the whole row look inert.

**Fix:**
- Give the input a subtle `bg-[var(--muted)]/30` background with `rounded-[var(--radius-sm)]` so it reads as a text field.
- Always show the `+` button in accent color (muted/low-opacity when empty, full when filled) so the row reads as an action area.

### 3.3 Channel Card Active Indicator is Tiny and Left-Aligned

**Problem:** The active channel gets a 2px-wide, 20px-tall accent bar on the absolute left. On a 260px sidebar, this is nearly invisible. Combined with the minimal `bg-[var(--surface)]` background, the active state doesn't pop.

**Fix:**
- Replace the thin bar with a full left-border approach: `border-l-2 border-[var(--accent)]` directly on the card.
- Add a slightly stronger background: `bg-[var(--accent-soft)]/40` for the active card.
- This makes the active state scannable at a glance.

### 3.4 Drag Handle Only Visible on Desktop, Not Communicated on Mobile

**Problem:** Drag handles (`lg:flex`) are hidden on mobile. The long-press drag is implemented but has no visual hint. Users won't discover channel reordering on mobile.

**Fix:**
- Add a small reorder icon or "hold to reorder" hint text at the top of the channel list when in custom sort mode and more than one channel exists.
- Alternatively, add an explicit "Reorder" mode toggle (similar to manage mode) that shows drag handles on mobile.

---

## 4. Video Sidebar

### 4.1 Video Cards Use Two Different Layout Densities

**Problem:** Channel cards are compact (avatar + name, ~44px tall). Video cards are tall (88px thumbnail + title + date + status icons, ~80px tall). The visual jump when switching from channels to videos on mobile is jarring.

**Fix:**
- Offer a compact video list mode (no thumbnails, just title + date + status) as default on mobile, with a toggle to show thumbnails.
- On desktop, keep the thumbnail layout but tighten it: reduce thumbnail to `w-[72px]` and font to `12px`.

### 4.2 "More" / "Explore History" Button is Ambiguous

**Problem:** The button at the bottom of the video list says "More" when there are more pages, then changes to "Explore History" when paginated results are exhausted. "Explore History" triggers a YouTube backfill -- a very different action -- but uses the same button style.

**Fix:**
- Make "Explore History" visually distinct: use an outlined/dashed border style with a clock/history icon to signal it's a different kind of action.
- Add a one-line explanation below: "Fetch older videos from YouTube".

### 4.3 Filter Button State is All-or-Nothing

**Problem:** The filter icon is either default (gray, 40% opacity) or fully styled (accent background, white icon). There's no indication of *which* filter is active without opening the menu.

**Fix:**
- When a filter is active, show a small badge/chip next to the filter icon (e.g., "Shorts" or "Unread") so users can see the active filter at a glance without opening the dropdown.

### 4.4 "Synced to ..." Text is Nearly Invisible

**Problem:** The sync depth indicator at the bottom uses `text-[11px] opacity-40`, making it almost unreadable.

**Fix:**
- Increase to `opacity-60` and add a small sync icon before the text to give it visual weight.

---

## 5. Content Panel

### 5.1 Content Mode Tabs + Action Buttons Compete for the Same Row

**Problem:** The content panel header has toggle tabs (transcript/summary/highlights/info) on the left and action buttons (format/revert/regenerate/youtube/edit/acknowledge) on the right. On smaller desktop widths or when multiple actions are visible, the row wraps and the actions land below the tabs with inconsistent alignment.

**Fix:**
- Pin the content mode tabs to a dedicated row.
- Move the action buttons into a second row or into a toolbar that appears below the tabs. This gives each concern its own horizontal space.
- On mobile, collapse the action buttons into a floating action bar pinned to the bottom of the content panel (above the tab bar), similar to how text editors handle toolbars.

### 5.2 Breadcrumb is Too Subtle

**Problem:** The breadcrumb `Channel > Video Title` uses `text-[12px] opacity-60`. It's the only wayfinding element in the content panel and it's barely visible.

**Fix:**
- Increase font to `text-[13px]`, remove the opacity reduction.
- Make the channel name a stronger color (soft-foreground without opacity hack).
- Add the channel thumbnail (16x16 rounded) before the channel name for visual anchoring.

### 5.3 Formatting Status Banner Takes Too Much Space

**Problem:** The formatting progress banner is a full-width bordered box with padding, which pushes the actual content down by ~80px. During the multi-turn formatting process, users see this banner for minutes.

**Fix:**
- Replace with a slim top-bar notification (like a toast pinned to the top of the content scroll area) that is ~32px tall with the pulsing dot + single line of text.
- This preserves content visibility while still communicating status.

### 5.4 Summary Quality Meta is Invisible

**Problem:** `WorkspaceSummaryMeta` uses `opacity-40` on the entire container, and the quality score, model name, and evaluator model are all at `text-[11px]`. This critical information is effectively hidden.

**Fix:**
- Remove the `opacity-40` from the container.
- Display the quality score as a prominent badge: `rounded-full bg-[var(--accent-soft)] px-3 py-1 text-[13px] font-bold`.
- Show the model name at normal opacity.
- Keep the eval note collapsed but make the "Show eval" toggle more prominent.

### 5.5 No Visual Distinction Between Transcript Render Modes

**Problem:** When a transcript switches from `plain_text` to `markdown` after formatting, the visual change is dramatic (headings appear, paragraphs reflow) but there's no indicator telling the user *why* the layout changed.

**Fix:**
- Add a small mode indicator chip above the content: "Plain text" or "Formatted" with a toggle to switch between views.
- This lets users understand and control the rendering.

---

## 6. Mobile-Specific Issues

### 6.1 Mobile Tab Bar + Footer Stacking is Confusing

**Problem:** On mobile, the footer moves to the top (`top: 0`) and the tab bar is at the bottom. The footer at the top contains only copyright + GitHub link, which is low-value content consuming `2rem` of precious screen real estate.

**Fix:**
- Remove the footer entirely on mobile. Move the GitHub link and copyright into a Settings/About section accessible from the section navigation dropdown.
- This reclaims the top bar for actual navigation or content.

### 6.2 Mobile Panel Transitions are Instant, Not Animated

**Problem:** Switching between channels/videos/content tabs via the bottom tab bar does an instant show/hide (`hidden`/`flex`). There's no transition, which makes the UI feel like a prototype rather than a polished app.

**Fix:**
- Add a horizontal slide transition (or a simple crossfade) when switching panels. Svelte's built-in `fly` transition would work well here.
- Keep the transition fast (150-200ms) so it doesn't feel sluggish.

### 6.3 Bottom Tab Bar Icons Need Active-State Polish

**Problem:** Active tab items only change color and opacity. The icons and labels are the same size in both states. This makes it hard to tell which tab is active at a glance on a small screen.

**Fix:**
- Add a filled variant of each icon for the active state (or add a small dot/bar under the active tab label).
- Slightly scale the active icon (`scale-110`) for visual emphasis.

### 6.4 No Swipe Gesture Support

**Problem:** The three-panel mobile layout (channels -> videos -> content) is a natural candidate for swipe navigation, but only bottom-tab taps are supported.

**Fix:**
- Add horizontal swipe gesture detection between panels. Libraries like `hammer.js` or a simple touch event handler would enable this.
- Show subtle edge indicators (thin accent line on the swipe direction) to hint at the gesture.

---

## 7. Cross-Cutting Visual Inconsistencies

### 7.1 Mixed Component API Styles (Svelte 4 vs Svelte 5)

**Problem:** Some components use `export let` props (Svelte 4 style: ChannelCard, VideoCard, ContentEditor, ContentActionButton, Toggle, ConfirmationModal) while others use `$props()` rune (Svelte 5 style: WorkspaceHeader, WorkspaceContentPanel, ThemePanel, AiStatusIndicator). This isn't a visual bug but contributes to the Frankenstein feel in maintenance.

**Fix:**
- Migrate all components to `$props()` rune syntax for consistency. This is a mechanical refactor.

### 7.2 Inconsistent Border Radius Values

**Problem:** The app uses at least 5 different radius strategies:
- `rounded-[var(--radius-sm)]` (8px) -- cards, inputs
- `rounded-[var(--radius-md)]` (12px) -- panels, banners
- `rounded-[var(--radius-lg)]` (20px) -- page sections, search popover
- `rounded-full` (9999px) -- pills, buttons, toggles
- `rounded-[6px]` -- video thumbnails (hardcoded, doesn't use the scale)

**Fix:**
- Standardize on three tiers: `--radius-sm` for inline elements, `--radius-md` for cards/panels, `--radius-lg` for page-level containers. Remove hardcoded pixel values.
- Video thumbnails should use `rounded-[var(--radius-sm)]`.

### 7.3 Opacity as a Styling Crutch

**Problem:** The codebase uses `opacity-` classes excessively: `opacity-40`, `opacity-50`, `opacity-60`, `opacity-70`, `opacity-80` on text, icons, and containers. This creates a washed-out, low-contrast feel where nothing stands out. Examples:
- Channel handle: `opacity-40`
- Video date: `opacity-50`
- Sidebar heading: full opacity (inconsistent with other labels)
- Summary meta: `opacity-40` on the container
- Footer links: `opacity-0.5` / `opacity-0.6`

**Fix:**
- Eliminate opacity classes on text entirely. Use `--soft-foreground` directly (it's already a muted color) or introduce `--muted-foreground` at 50% visual weight.
- Reserve opacity for interactive state changes (hover, disabled) only.
- This single change will dramatically improve readability and reduce the washed-out feel.

### 7.4 Inconsistent Typography Scale

**Problem:** Font sizes jump erratically: `10px`, `11px`, `12px`, `13px`, `14px`, `15px`, `20px`, `28px`, `32px`. Labels use `10px` and `11px` interchangeably. Body text uses `13px` in some places and `15px` in others.

**Fix:**
- Define a 5-step type scale and use it consistently:
  - `--text-xs`: 10px (labels, badges)
  - `--text-sm`: 12px (secondary text, metadata)
  - `--text-base`: 14px (body text, list items)
  - `--text-lg`: 18px (section headings)
  - `--text-xl`: 24px (page headings)
- Map every text element to one of these steps.

### 7.5 Excessive Use of `font-bold` and Uppercase Tracking

**Problem:** Almost every piece of text is either `font-bold` or `font-semibold` with `uppercase tracking-[0.1em]`. When everything is bold and uppercase, nothing has emphasis. Labels, badges, headings, and body text all compete at the same visual weight.

**Fix:**
- Reserve `uppercase tracking-wide` for category labels only (e.g., "CHANNELS", "VIDEOS", "TYPE", "STATUS").
- Use `font-medium` as the default weight for body text and list items.
- Use `font-semibold` for headings and active states only.
- This creates a proper visual hierarchy where the eye knows where to look.

---

## 8. Download Queue Page

### 8.1 Duplicates Workspace Layout Code

**Problem:** `download-queue/+page.svelte` (1,277 lines) rebuilds the header, channel sidebar, and drag-and-drop logic from scratch instead of sharing components with the workspace. The header is nearly identical but subtly different (no search bar, different guide steps).

**Fix:**
- Extract a shared `AppShell` layout component that includes the header (logo, AI status, theme, section nav) and footer.
- Each page provides its own content, but the chrome is shared. This eliminates visual drift between pages.

### 8.2 Queue Tab Styling Differs from Content Mode Tabs

**Problem:** The workspace uses `Toggle.svelte` (pill-shaped buttons with icons) for content mode tabs. The download queue page uses a different tab pattern (not shared). This means the two tab UIs look different despite serving the same purpose.

**Fix:**
- Reuse `Toggle.svelte` (or a generalized `TabBar` component) for queue tabs.

---

## 9. Highlights Page

### 9.1 Page is a Flat List, Not Grouped

**Problem:** While the data is grouped by channel > video, the visual hierarchy is flat: each group looks like a card inside a card with the same border treatment. It's hard to scan.

**Fix:**
- Use a proper accordion or collapsible section pattern: channel name as a sticky header, videos as expandable rows, highlights as the expanded content.
- This reduces visual noise and lets users focus on one channel at a time.

---

## 10. Performance & PWA

### 10.1 No Skeleton Loading for the Workspace Shell

**Problem:** On initial load, the three-column layout appears empty until the bootstrap API returns. There's no shell skeleton, so the page feels broken for 1-2 seconds.

**Fix:**
- Render the three-column grid shell immediately with placeholder shimmer in each column. The current skeleton code exists inside each sidebar but the outer grid isn't shown until data arrives.

### 10.2 Search Popover Should Trap Focus

**Problem:** The search results popover is a complex dialog with interactive elements but doesn't trap focus. Users can tab out of it into the content behind it.

**Fix:**
- Add a focus trap when the search popover is open.
- Close on `Escape` (already implemented) and on Tab past the last result.

---

## Priority Ranking

| Priority | Spec | Impact | Effort |
|----------|------|--------|--------|
| P0 | 7.3 Opacity crutch removal | Fixes washed-out look globally | Low |
| P0 | 7.5 Font weight hierarchy | Fixes "everything screams" problem | Low |
| P0 | 2.2 Duplicate font loading | Free perf win | Trivial |
| P1 | 1.2 Grid gutter/padding consistency | Fixes column misalignment | Medium |
| P1 | 5.1 Tabs vs actions row separation | Fixes wrapping/clutter on content panel | Medium |
| P1 | 6.1 Remove mobile footer | Reclaims screen real estate | Low |
| P1 | 7.4 Typography scale | Fixes erratic sizing | Medium |
| P2 | 3.1 Sidebar toolbar density | Reduces icon clutter | Medium |
| P2 | 4.1 Video card density options | Fixes mobile video list UX | Medium |
| P2 | 5.4 Summary quality visibility | Makes key feature visible | Low |
| P2 | 8.1 Shared AppShell | Eliminates layout drift between pages | High |
| P3 | 6.2 Panel transitions | Polish / perceived quality | Low |
| P3 | 6.4 Swipe gestures | Mobile navigation enhancement | Medium |
| P3 | 1.1 Page component decomposition | Maintenance / long-term health | High |
| P3 | 7.1 Svelte 5 migration | Consistency / maintenance | Medium |
