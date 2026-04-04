# Tasks: Multi-Provider Library Expansion

## Current State
Canonical multi-provider contracts are implemented. YouTube, OpenAlex saved searches, podcast RSS feeds, and manually tracked website pages are subscribable through the current app shell via compatibility projection into the existing channel/video UI. Authenticated publisher support, website folders, and a fully source-native library UI remain open.

## Steps
- [x] Inventory all backend and frontend assumptions that hard-code `channel`, `video`, `video_id`, and `channel_id` as canonical identity.
- [x] Define the canonical `ContentSource`, `ContentItem`, `ContentPart`, and `MediaAsset` schemas and their TypeScript bindings.
- [x] Define provider adapter contracts for feed-backed, query-backed, and manual website sources.
- [x] Define the subscription container model for series, saved searches, folders, and standalone tracked sources.
- [ ] Define the top-level library information architecture for mixed-provider subscriptions with low visual clutter.
- [ ] Define the website-folder workflow for creating, renaming, reordering, and assigning tracked websites.
- [x] Define selection, routing, and deep-link behavior using generic source and item identity.
- [x] Define search, chat, highlights, and audio contracts against generic source, item, and part identity.
- [ ] Define mixed-library filters and progress states that work across podcasts, publications, websites, and YouTube.
- [x] Define the first-wave podcast ingestion flow, including source audio, show notes, transcript import, and ASR fallback boundaries.
- [x] Define the first-wave publication ingestion flow, including publication-series sources, saved-search sources, metadata enrichment, and text extraction boundaries.
- [x] Define migration behavior from the current channel-centric model and UI to the new source-centric library model.
- [ ] Define verification criteria for mixed-library behavior and large subscription sets so clutter and discoverability can be evaluated explicitly.

## Decisions Made During Implementation
- The canonical model will distinguish sources, items, parts, and media assets instead of stretching `channel` and `video` to cover every provider.
- Query-backed publication subscriptions are first-class sources and first-class subscription containers, not a workaround or an implementation detail.
- Podcast episodes remain grouped under their series and do not appear as top-level subscriptions.
- Manually tracked websites live under a dedicated `Websites` area with user-managed folders instead of being flattened into the main series lists.
- The migration is additive: generic `source` and `item` contracts now sit alongside legacy `channel` and `video` fields so existing YouTube flows keep working while new providers move onto the canonical model.
- Feed-backed, query-backed, and manual website provider behavior now has an explicit backend adapter surface, with YouTube wired through the feed-backed contract for source resolution and canonical sync batching.
- OpenAlex saved-search sources, podcast RSS feeds, and manually tracked website pages now sync into the current workspace through compatibility projection rather than waiting for a full shell rewrite.
- Website folder management is still intentionally open: website pages are subscribable now, but the dedicated `Websites` folder UI from the target design is not implemented yet.
