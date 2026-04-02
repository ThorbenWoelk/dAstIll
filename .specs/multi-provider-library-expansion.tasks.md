# Tasks: Multi-Provider Library Expansion

## Current State
Spec created, not yet started.

## Steps
- [ ] Inventory all backend and frontend assumptions that hard-code `channel`, `video`, `video_id`, and `channel_id` as canonical identity.
- [ ] Define the canonical `ContentSource`, `ContentItem`, `ContentPart`, and `MediaAsset` schemas and their TypeScript bindings.
- [ ] Define provider adapter contracts for feed-backed, query-backed, and manual website sources.
- [ ] Define the subscription container model for series, saved searches, folders, and standalone tracked sources.
- [ ] Define the top-level library information architecture for mixed-provider subscriptions with low visual clutter.
- [ ] Define the website-folder workflow for creating, renaming, reordering, and assigning tracked websites.
- [ ] Define selection, routing, and deep-link behavior using generic source and item identity.
- [ ] Define search, chat, highlights, and audio contracts against generic source, item, and part identity.
- [ ] Define mixed-library filters and progress states that work across podcasts, publications, websites, and YouTube.
- [ ] Define the first-wave podcast ingestion flow, including source audio, show notes, transcript import, and ASR fallback boundaries.
- [ ] Define the first-wave publication ingestion flow, including publication-series sources, saved-search sources, metadata enrichment, and text extraction boundaries.
- [ ] Define migration behavior from the current channel-centric model and UI to the new source-centric library model.
- [ ] Define verification criteria for mixed-library behavior and large subscription sets so clutter and discoverability can be evaluated explicitly.

## Decisions Made During Implementation
- The canonical model will distinguish sources, items, parts, and media assets instead of stretching `channel` and `video` to cover every provider.
- Query-backed publication subscriptions are first-class sources and first-class subscription containers, not a workaround or an implementation detail.
- Podcast episodes remain grouped under their series and do not appear as top-level subscriptions.
- Manually tracked websites live under a dedicated `Websites` area with user-managed folders instead of being flattened into the main series lists.
