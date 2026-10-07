# dAstIll Reader

The Svelte 5 frontend for dAstIll: your subscribed channels, printed as a calm morning paper.

- The front page shows unread video summaries from every channel, newest first.
- Mark a story as read and the next one takes its place. Undo brings it back.
- Pick a section to read one channel at a time.
- Select text to highlight it. `/highlights` lists and searches every highlight.
- `/finished` lists stories you marked as read. `/stories/{id}` opens any story.
- `/sections` follows or unfollows channels, and signs you out.

## Develop

```bash
bun install
bun run dev        # http://localhost:3000, proxies /api to VITE_API_BASE
```

Env keys are listed in `.env.example`. See [../docs/operations/local-development.md](../docs/operations/local-development.md) for the shared env setup.

## Check

```bash
bun run format:check && bun run lint && bun run check
bun run test
bun run test:e2e
bun run build
```

Design rules: [../design.md](../design.md).
