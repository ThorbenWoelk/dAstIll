# DASTILL Frontend v1.0.0

DASTILL v1.0.0 is the Svelte 5 frontend for channel tracking and video distillation workflows.

## Product Scope (v1.0.0)

- Channel workspace with customizable channel order.
- Video list with short/long and acknowledged/unacknowledged filtering.
- Transcript, summary, and info tabs for each video.
- Summary quality metadata display (score and incoherence note).
- Download queue observatory for transcript/summary processing states.

## Local Development

Install dependencies:

```sh
bun install
```

Run development server:

```sh
bun run dev
```

Typecheck:

```sh
bun run check
```

Production build:

```sh
bun run build
```

Preview production build:

```sh
bun run preview
```
