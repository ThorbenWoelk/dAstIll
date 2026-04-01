# Tasks: Docs Architecture Diagrams

## Current State
Diagram rendering support is live in the VitePress theme, the targeted architecture and
flow pages now include explicit diagrams, and the docs were verified in a fresh preview
instance where every targeted page rendered Mermaid output as SVG with no render-error
blocks.

## Steps
- [x] Create spec and task files for docs architecture diagrams.
- [x] Add diagram rendering support that works in the VitePress docs UI.
- [x] Verify architecture and data-flow details against the current codebase.
- [x] Add diagrams to the existing architecture and flow docs pages.
- [x] Build and format the docs to verify the diagrams render correctly.

## Decisions Made During Implementation
- Use a single diagram mechanism across docs pages so architecture visuals render
  consistently in VitePress.
- Render Mermaid on the client through a shared VitePress theme component instead of
  relying on unsupported fenced code blocks.
- Reflect the current hybrid storage model explicitly: videos live in Firestore, while
  channels, transcript/summary blobs, user-scoped records, and search projection data
  live in S3.
