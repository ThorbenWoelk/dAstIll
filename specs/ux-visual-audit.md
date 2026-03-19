# dAstIll Visual UX Audit (Screenshot-Based)

Based on screenshots taken at 1440px (desktop), 768px (tablet), and 390px (mobile) across all three pages: Workspace, Queue, and Highlights. Both light and dark mode reviewed.

Reference screenshots in `./specs/`:
- `desktop-1440-workspace.png`, `desktop-1440-queue.png`, `desktop-1440-highlights.png`
- `desktop-1440-dark-workspace.png`
- `tablet-768-workspace.png`, `tablet-768-queue.png`
- `mobile-390-workspace.png`, `mobile-390-queue.png`, `mobile-390-highlights.png`

---

## 1. Cross-Page Layout Inconsistency (The Biggest Frankenstein Signal)

### 1.1 Three Pages, Three Different Layouts

**Visible in:** All desktop screenshots side-by-side.

| Page | Layout | Sidebar | Bottom Nav (mobile) |
|------|--------|---------|---------------------|
| Workspace | 3-column grid (channels + videos + content) | Channel sidebar | CHANNELS / VIDEOS / CONTENT |
| Queue | 2-column (channels + queue details) | Channel sidebar (rebuilt) | CHANNELS / DETAILS |
| Highlights | Single centered column, no sidebar | None | No tab bar -- floating "HIGHLIGHTS" dropdown at bottom center |

The highlights page looks like it belongs to a completely different app. It has a serif heading, centered card layout, no sidebar, and a different mobile navigation pattern. The queue page is closer to the workspace but still uses a different column structure and rebuilds the channel sidebar from scratch.

**Fix:**
- Create a shared `AppShell` component with: header (logo, AI status, nav, search/theme), optional sidebar slot, main content slot, and footer.
- All three pages use this shell. The workspace fills all three slots; queue fills sidebar + content; highlights fills content only with a back-link to the sidebar view.
- On mobile, the bottom tab bar should always be present with 3-4 consistent items (e.g., Workspace, Queue, Highlights, Docs) as primary navigation, replacing the per-page ad-hoc tab bars.

### 1.2 Header Composition Varies Per Page

**Visible in:** Comparing desktop screenshots.

- **Workspace header:** DASTILL + ? + (nav pills: WORKSPACE/QUEUE/HIGHLIGHTS/DOCS) + sun icon + search bar
- **Queue header:** DASTILL + ? + sun icon + (nav pills shifted to far right, no search bar)
- **Highlights header:** DASTILL + sun icon only (no ? icon, no guide trigger, no search bar)

The nav pills are center-aligned on workspace but right-aligned on queue/highlights because the search bar pushes them. Without the search bar, the pills drift to the far right, leaving a big empty gap.

**Fix:**
- Lock the header layout: logo-area (left) | nav-pills (center) | actions (right: search toggle + theme + guide).
- The search bar should be a toggle that expands inline or drops down, not a permanent element that shifts everything else.
- All pages should have the same header elements (logo, nav, theme, guide).

---

## 2. Navigation & Tab Style Clash

### 2.1 Three Different Tab Paradigms

**Visible in:** Desktop workspace (pill tabs for content mode), desktop queue (underline tabs for queue sections), desktop workspace header (pill nav for page navigation).

| Location | Style | Example |
|----------|-------|---------|
| Page navigation (header) | Rounded pills, uppercase, tracking-wide | WORKSPACE / QUEUE / HIGHLIGHTS / DOCS |
| Content mode (workspace) | Rounded pills with icons, accent-soft bg | TRANSCRIPT / SUMMARY / HIGHLIGHTS / INFO |
| Queue tabs | Underline + bold text, no pills | TRANSCRIPTS / SUMMARIES / EVALUATIONS |

The queue tabs look like they came from a different design system. The underline style has no relationship to the pill style used everywhere else.

**Fix:**
- Unify all tab-like controls to use the same `Toggle.svelte` component (or a generalized `TabBar`).
- Queue tabs should be pills with the same styling as the workspace content mode tabs.

### 2.2 Mobile Section Navigation is Inconsistent

**Visible in:** `mobile-390-workspace.png` vs `mobile-390-highlights.png`.

- Workspace mobile: bottom tab bar (CHANNELS/VIDEOS/CONTENT) + "WORKSPACE" dropdown in header.
- Queue mobile: bottom tab bar (CHANNELS/DETAILS) + "QUEUE" dropdown in header.
- Highlights mobile: NO bottom tab bar. Instead a floating "HIGHLIGHTS" dropdown button at the very bottom center of the screen.

Users have no consistent way to navigate between pages on mobile. Each page invents its own navigation.

**Fix:**
- Replace all per-page bottom tab bars with a single app-level tab bar: **Workspace | Queue | Highlights** (3 items, always present).
- Within the workspace, panel switching (channels/videos/content) should use a secondary mechanism -- either swipe gestures, inline tabs at the top of the content area, or a segmented control below the header.

---

## 3. Empty States Look Broken

### 3.1 Skeleton Loaders Occupy Only Part of the Column

**Visible in:** `desktop-1440-workspace.png` -- the channel sidebar shows 4 skeleton cards, then a vast empty space below.

The skeleton suggests there are exactly 4 items loading, followed by nothing. In reality, the backend is loading. The empty space below the skeletons makes the column look incomplete.

**Fix:**
- Fill the visible column height with skeleton rows (6-8 based on available height, not a fixed 4).
- Add a subtle shimmer effect that extends to the bottom of the scroll area.

### 3.2 "Waiting for the library to fill" is Nearly Invisible

**Visible in:** `desktop-1440-workspace.png` -- the video column shows this message in light italic at very low contrast.

The text blends into the background. A first-time user would think the column is broken.

**Fix:**
- Replace with a structured empty state: an icon (e.g., a play button outline), a heading ("No videos yet"), and a subline ("Select a channel to browse its videos").
- Use normal text weight at `--soft-foreground` without opacity reduction.

### 3.3 "None" in Queue Header Looks Like a Bug

**Visible in:** `desktop-1440-queue.png` -- shows a default avatar with "None" text and "0 items".

When no channel is selected, the queue header displays "None" as if it's a channel name. This looks like a data error.

**Fix:**
- Replace with an explicit empty state: "Select a channel from the sidebar" in a centered block.
- Hide the queue tabs and "0 items" counter until a channel is actually selected.

### 3.4 "Select a video to view its content" is Lost

**Visible in:** `desktop-1440-workspace.png` -- the content panel (rightmost column) has this message dead center, but it's tiny text at very low opacity.

The largest column on screen has a single, barely-visible line of text. This is wasted real estate and feels unfinished.

**Fix:**
- Replace with a richer empty state: the app logo or a document icon, "Select a video" heading, and a brief description of what the content panel shows.
- Or, use this space to show the feature guide / onboarding card for first-time users.

---

## 4. Desktop-Specific Issues

### 4.1 Footer Steals Vertical Space for Low-Value Content

**Visible in:** All desktop screenshots -- a fixed footer at the bottom reading "2026 Thorben Woelk" + "GitHub" link.

The three-column workspace uses `h-[calc(100vh-4rem)]` for sticky columns, but the footer occupies ~36px of fixed space at the bottom. The copyright and GitHub link are not worth the permanent real estate cost.

**Fix:**
- Remove the fixed footer on desktop entirely. Move copyright into a collapsible footer inside the channel sidebar (at the bottom of the scroll area) or into an "About" section in the settings/theme panel.
- The GitHub link can go next to the Docs link in the header nav.

### 4.2 Search Bar Dominates the Header

**Visible in:** `desktop-1440-workspace.png` -- the search bar is ~380px wide and is the most visually prominent header element.

The pill-shaped search bar with its border, shadow, search icon, and "ask" hint is heavier than the logo, nav, and all other header elements combined. Yet search is a secondary discovery action, not the primary workflow.

**Fix:**
- Reduce to a compact search trigger (icon + short placeholder) that expands to full width on focus.
- Or move the search bar into its own row below the header, so it doesn't compete with navigation.

### 4.3 Videos Column Filter Icon is Nearly Invisible

**Visible in:** `desktop-1440-workspace.png` -- the filter icon to the right of "Videos" heading is at ~40% opacity.

Filters are an important feature (type: all/long/short, status: all/read/unread) but the trigger is barely visible.

**Fix:**
- Increase base opacity to 60%.
- When any filter is active, show a small count badge or the active filter name next to the icon.

---

## 5. Mobile-Specific Issues

### 5.1 "CUSTOM" Sort Label Clutters the Channel Toolbar

**Visible in:** `mobile-390-workspace.png` and `tablet-768-workspace.png` -- the channel sort button shows "CUSTOM" text next to its icon on mobile, but on desktop it only shows the icon.

On mobile where horizontal space is scarce, this text label makes the toolbar look busy. There are now 4 elements in the toolbar row: trash icon, search icon, sort icon + "CUSTOM" label.

**Fix:**
- Remove the text label on all breakpoints. The tooltip (on desktop hover) and the icon variant are sufficient to communicate sort mode.
- Or show the label only when the sort mode is non-default (alpha or newest) as a badge.

### 5.2 Search Bar on Mobile Takes a Full Row

**Visible in:** `mobile-390-workspace.png` and `tablet-768-workspace.png` -- the search bar wraps below the header as a full-width row.

On a 390px screen, the search bar consumes ~56px of vertical space permanently (even when not in use). Combined with the header row (~48px) and the bottom tab bar (~56px), the user loses ~160px to chrome before seeing any content.

**Fix:**
- Hide the search bar by default on mobile. Show a search icon in the header that opens a full-screen search overlay when tapped.
- This reclaims the search row for content.

### 5.3 Highlights Page Has No Mobile Navigation

**Visible in:** `mobile-390-highlights.png` -- no bottom tab bar, only a floating "HIGHLIGHTS" dropdown.

Users on mobile who navigate to highlights have no way to get back to the workspace or queue without using the floating dropdown (which is a section switcher, not a tab bar). The interaction pattern is completely different from the other pages.

**Fix:**
- Use the same app-level bottom tab bar proposed in 2.2. The highlights page should have identical chrome to all other pages.

---

## 6. Visual Weight & Contrast

### 6.1 Active Accent Tab (TRANSCRIPT) is Very Heavy

**Visible in:** `desktop-1440-workspace.png` -- the active TRANSCRIPT pill has a strong red/accent background that visually dominates.

The accent-soft background + accent-strong text + border creates a button that looks more important than the page heading or logo. The inactive tabs are barely visible in comparison (transparent bg, 65% opacity).

**Fix:**
- Soften the active state: use `accent-soft` background at 50% opacity, keep accent text, drop the border.
- Increase inactive tab opacity from 65% to 80% so the contrast between active and inactive is less dramatic.

### 6.2 Skeleton Bars in Dark Mode Have Good Contrast, Light Mode is Washed Out

**Visible in:** Comparing `desktop-1440-workspace.png` (light) vs `desktop-1440-dark-workspace.png` (dark).

In light mode, the skeleton loading bars are barely distinguishable from the background (`--muted` on `--background` is very low contrast). In dark mode, the contrast is noticeably better.

**Fix:**
- In light mode, increase skeleton bar contrast: use `bg-[var(--border)]` instead of `bg-[var(--muted)]`, or add `opacity-80` to the current `opacity-60` bars.

---

## 7. Structural Proposals (Out-of-the-Box)

### 7.1 Unified Bottom Navigation (Mobile)

Replace all per-page bottom tab bars with a single app-level navigation:

```
[ Workspace ]  [ Queue ]  [ Highlights ]  [ Settings ]
```

- "Workspace" becomes the primary view (channels -> videos -> content flow handled via swipe or inline sub-tabs).
- "Queue" shows the download/processing pipeline.
- "Highlights" shows saved passages.
- "Settings" replaces the floating theme toggle, guide, and docs links.

This eliminates the three different mobile navigation patterns and gives users a consistent mental model.

### 7.2 Workspace Panel Switcher (Mobile)

Within the workspace, replace the current bottom tab bar with a swipeable panel approach:

- Swipe right-to-left: Channels -> Videos -> Content
- Show a page indicator (three dots) at the top to communicate position.
- The breadcrumb in the content panel (Channel > Video Title) becomes tappable to navigate back.

This is the standard mobile pattern (think iOS Mail: mailboxes -> messages -> message content).

### 7.3 Collapsible Sidebars (Desktop)

On desktop, allow the channel and video sidebars to collapse to icon-only width (~48px) with a toggle:

- Collapsed channel sidebar: just channel avatars in a vertical strip.
- Collapsed video sidebar: hidden entirely, content panel fills the space.
- This lets users who have selected their video maximize reading area.

---

## Priority Ranking

| Priority | Spec | Why | Effort |
|----------|------|-----|--------|
| P0 | 1.1 Three pages three layouts | Root cause of Frankenstein feel | High |
| P0 | 2.2 Mobile nav inconsistency | Users can't navigate between pages | Medium |
| P0 | 1.2 Header composition varies | Pages look unrelated | Medium |
| P1 | 2.1 Tab style clash | Queue tabs look foreign | Low |
| P1 | 3.2 Invisible empty states | First-time users think app is broken | Low |
| P1 | 3.3 "None" in queue | Looks like a bug | Low |
| P1 | 4.1 Footer steals space | Free vertical space | Low |
| P2 | 3.1 Skeleton loader coverage | Loading state looks incomplete | Low |
| P2 | 4.2 Search bar dominance | Header visual balance | Medium |
| P2 | 5.2 Mobile search full row | Mobile vertical space wasted | Medium |
| P2 | 6.1 Active tab too heavy | Visual weight balance | Trivial |
| P3 | 7.1 Unified bottom nav | OOTB: consistent mobile nav model | High |
| P3 | 7.2 Swipeable panels | OOTB: native-feeling mobile UX | High |
| P3 | 7.3 Collapsible sidebars | OOTB: desktop reading mode | Medium |
