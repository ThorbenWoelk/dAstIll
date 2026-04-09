# Spec 10: Intermediate Responsive Breakpoints

Priority: **P2**
Scope: Layout system, global CSS, key components

## Problem

The app uses a single structural breakpoint: `lg: 1024px`. Below that is "mobile" and above is "desktop." There is no intermediate layout for tablets (768-1024px) or large phones in landscape (640-768px).

This means an iPad in portrait (810px) gets the exact same layout as an iPhone SE (375px). Tablets have enough space for a two-column layout but are forced into the phone UI with overlays and drawers.

## Current Breakpoint Usage

| Breakpoint | Size | Current Usage |
|------------|------|---------------|
| `sm` (640px) | Phone landscape / large phone | Minimal: some padding (`sm:px-6`), modal button layout (`sm:flex-row-reverse`) |
| `md` (768px) | Tablet portrait | Not used anywhere |
| `lg` (1024px) | Desktop | The only structural breakpoint — everything switches here |
| `xl` (1280px) | Wide desktop | Minimal: grid columns on vocabulary/highlights pages |

## Proposed Additions

### `md` breakpoint (768px) — Tablet layout

For screens 768-1023px, show a hybrid layout:
- **Two-column layout** where appropriate (sidebar visible + content, not full-screen overlay)
- **Bottom nav** still present (from spec 02) but could show larger labels
- **WorkspaceSidebar**: show as a permanent narrow panel (200-240px) instead of an overlay
- **Chat**: show sidebar + messages side-by-side instead of overlay
- **Grid layouts**: vocabulary `md:grid-cols-2` (currently waits for `lg`), highlights `md:grid-cols-1` with larger cards

### `sm` breakpoint (640px) — Large phone

For screens 375-639px, current mobile layout mostly works but:
- **MobileChannelGallery**: card width could use `sm:w-[50vw]` instead of `64vw` on larger phones
- **Content padding**: `sm:px-5` for slightly more breathing room
- **Chat input**: compact mode as in spec 08

## Implementation Approach

This is the lowest-priority spec because it requires touching many layout components. Implement after the P0 and P1 specs are done.

1. Start with `WorkspaceSidebar` — add `md:flex` alongside `lg:flex` to show sidebar on tablets
2. Update `WorkspaceShell` to support a three-tier layout (phone / tablet / desktop)
3. Update `chat/+page.svelte` for tablet two-column
4. Update grid layouts with `md:` column counts
5. Test on iPad simulator / 768px Chrome DevTools

## Files to Modify

| File | Change |
|------|--------|
| `WorkspaceShell.svelte` | Three-tier responsive layout |
| `WorkspaceSidebar.svelte` | `md:flex` permanent narrow panel |
| `chat/+page.svelte` | `md:` two-column layout |
| `vocabulary/+page.svelte` | `md:grid-cols-2` |
| `MobileChannelGallery.svelte` | `sm:` card width adjustment |

## Verification

- iPad portrait (810px) shows a usable two-column layout
- iPhone (375px) layout unchanged from current mobile
- No layout shifts or content overflow at any width between 320px and 1440px
- `bun run check`, `bun run test` pass
