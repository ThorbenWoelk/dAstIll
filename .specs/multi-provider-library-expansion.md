# Multi-Provider Library Expansion

## Problem

dAstIll currently models the world as YouTube channels containing videos with transcripts and summaries. That shape breaks down as soon as the product needs to support podcasts, research publications, publication series, saved searches against science engines, and manually tracked websites. The current `channel -> video -> transcript/summary` assumptions make identity, storage, routing, search, sync, and library organization too provider-specific to extend cleanly. If new providers are added without a broader product model, the subscription surface will also become noisy and hard to scan because podcasts, publication searches, series, and manually tracked sites all group content differently.

## Goal

Expand dAstIll into a unified multi-provider content library that can ingest and organize podcasts, publications, saved-search publication feeds, websites, and existing YouTube content as first-class content types. The backend and frontend should both operate on a provider-neutral model, and the library should remain clutter-free by grouping subscriptions according to their natural source type:

- podcast episodes under podcast series
- publications under publication series when a real series exists
- publications under saved-search sources when the subscription is query-backed
- manually tracked websites under a generic `Websites` area with user-created folders

## Requirements

- Introduce a provider-neutral canonical model that separates:
  - a subscribed or tracked source
  - an individual content item
  - one or more content parts or media assets derived from that item
- Support at least these source archetypes:
  - podcast series backed by RSS or equivalent feed metadata
  - publication series backed by publisher feeds or provider pages
  - saved-search publication sources backed by a query against a science engine
  - manually tracked websites and pages
  - YouTube channels mapped into the same generic model
- Support at least these item kinds:
  - podcast episode
  - publication or paper
  - article or webpage
  - video
- Support at least these content part or asset kinds:
  - full text
  - abstract
  - transcript
  - show notes
  - chapters
  - generated summary
  - source audio
  - generated summary audio
- Assign stable internal IDs to sources, items, and parts so provider-specific IDs such as YouTube video IDs, RSS GUIDs, DOIs, arXiv IDs, or query hashes do not act as the only primary keys.
- Preserve provider-specific metadata without leaking provider-specific identity into the core schema.
- Allow publication subscriptions to be query-backed rather than feed-backed, so a user's current notion of a "channel" can become a saved search scoped to a specific science engine and query.
- Allow website tracking without forcing every tracked page to pretend to be part of a feed or publication series.
- Present all user subscriptions inside one unified library model rather than separate disconnected product areas.
- Preserve source grouping semantics by type:
  - podcast episodes grouped under podcast series
  - publications grouped under publication series when a concrete series exists
  - publications grouped under a saved-search source when the subscription is query-backed
  - websites grouped under a generic websites area with user-created folders
- Support at least these subscription container kinds:
  - series
  - saved search
  - folder
  - standalone tracked source when no higher-level grouping exists
- Provide a clutter-free top-level subscription surface that helps users browse sources without mixing every item from every provider into one unstructured sidebar.
- Make it obvious which subscriptions are:
  - feed-backed
  - query-backed
  - manually curated
- Allow users to create, rename, reorder, and remove manual website folders under the generic `Websites` area.
- Allow individual websites or pages to be assigned to a manual website folder without requiring each one to become a pseudo-series.
- Support deep links and workspace selection using generic source and item IDs rather than only `channel_id` and `video_id`.
- Support library browsing and filtering by:
  - source type
  - provider
  - content type
  - unread or acknowledged state, or equivalent mixed-library progress state
- Preserve the existing summary pipeline as a reusable text-based capability for non-video content.
- Preserve the existing search and chat product goals while generalizing result payloads from `channel/video/source_kind` to `source/item/part/provider`.
- Keep existing YouTube content functional during the migration so the product can support both legacy YouTube flows and new source types in the same library.
- Preserve room for future provider types without redesigning the entire library model again.

## Non-Goals

- Building every possible provider integration in the first pass.
- Solving paid or authenticated publisher access in this scope.
- Final pixel-level UI styling, animation, or visual polish decisions.
- Advanced recommendation or ranking logic for subscriptions.
- Collaborative folders or shared libraries.
- Automatic taxonomy generation for manually added websites in the first pass.
- Full deduplication across multiple providers that reference the same external content.
- Replacing the current summarization, evaluation, or search systems wholesale.
- Full migration off all legacy `channel` and `video` naming in a single release.

## Design Considerations

### Canonical model

The expansion should center on a provider-neutral model such as:

- `ContentSource`
  - represents a subscription target or tracked origin
  - examples: podcast series, publisher series, saved search, website source, YouTube channel
- `ContentItem`
  - represents a specific episode, paper, article, webpage, or video
- `ContentPart`
  - represents the searchable or readable material attached to an item
  - examples: transcript, full text, abstract, show notes, chapters, summary
- `MediaAsset`
  - represents playable media attached to an item
  - examples: source audio, generated summary audio

This avoids conflating source grouping, item identity, and derived content.

### Source archetypes

The system should explicitly support three distinct source behaviors because they sync and group content differently:

- feed-backed series
  - podcast RSS feeds
  - publication feeds or publisher update feeds
- query-backed series
  - saved searches in OpenAlex, arXiv, Semantic Scholar, or similar engines
  - source identity is the provider plus normalized query definition
- manually curated website tracking
  - user adds sites or pages directly
  - does not require provider-managed grouping

### Provider adapters

Provider-specific logic should live behind adapters rather than inside the canonical model. Each provider may need some subset of:

- source resolution
- source sync and pagination
- item metadata extraction
- text extraction
- transcript import
- audio discovery

The important constraint is that provider differences stop at the adapter boundary.

### Clutter-free library organization

The product should shift from a "channels and videos" mental model to a broader "sources and items" mental model. The library should avoid a giant flat list of mixed provider entries. A cleaner structure is:

- top-level library surface
  - shows source groups or container types
- within each group
  - shows subscribed sources
- within a source
  - shows items such as episodes, papers, articles, or videos

This preserves information scent while keeping the top-level list stable even when the user has many subscriptions.

### Publication subscriptions

Publication subscriptions may not always map to a real-world publisher series. Some will be saved searches such as "recent multimodal AI papers in OpenAlex" or "recent Google publications matching a query." Those should behave like durable source containers with their own names, sync rules, and item lists.

### Podcast subscriptions

Podcast subscriptions should use series-level grouping as the default unit. Episodes belong under the series and should not clutter the top-level subscription surface individually.

### Website tracking

Websites are different because users may want to track a set of unrelated sites without forcing them into artificial provider-defined series. A dedicated `Websites` area with manual folders keeps this flexible without mixing manually curated pages into publication or podcast sections.

### Search and chat generalization

Search and chat should continue indexing and citing content parts, but those citations need to reference generic source and item identity. Retrieval should distinguish:

- item kind
- provider
- part kind

For example, a publication may have `abstract`, `full_text`, and `summary`, while a podcast episode may have `show_notes`, `transcript`, `source_audio`, and `summary`.

### Navigation and routing

The workspace and deep-link model should target:

- selected source group or container
- selected source
- selected item
- selected content part

This is more durable than a fixed `channel/video` selection model and makes it possible to reuse one workspace shell across all provider types.

### Migration strategy

The implementation should be incremental. YouTube should become the first adapter mapped into the new generic model instead of being rewritten out of existence. This reduces migration risk and keeps the app usable while podcasts, publications, and websites are added.

## Open Questions

- Which science engines should be treated as first-wave query-backed publication providers versus enrichment-only providers?
- Should the top-level library surface be organized primarily by source type, by user-defined folders, or by a hybrid of both?
- Should manually tracked websites support both folder assignment and ad hoc tags in the first pass, or should tags wait for a later iteration?
- Should the `Websites` area allow direct page tracking only, or also support tracking whole domains as sources with discovered pages beneath them?
- Should website tracking support feed discovery automatically when a site exposes RSS or Atom, or should that wait for a later pass?
- Should imported publication PDFs be stored as first-class assets in the first implementation slice, or should the first pass focus on metadata plus extracted text only?
- Which provider-specific metadata must be exposed in the frontend immediately versus stored only for later use?
