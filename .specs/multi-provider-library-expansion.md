# Multi-Provider Library Expansion

## Problem

dAstIll currently models the world as YouTube channels containing videos with transcripts and summaries. That shape breaks down as soon as the product needs to support podcasts, research publications, publication series, saved searches against science engines, and manually tracked websites. The current `channel -> video -> transcript/summary` assumptions make identity, storage, routing, search, sync, and library organization too provider-specific to extend cleanly. If new providers are added without a broader product model, the subscription surface will also become noisy and hard to scan because podcasts, publication searches, series, and manually tracked sites all group content differently.

## Goal

Expand dAstIll into a unified multi-provider content library that can ingest and organize podcasts, publications, saved-search publication feeds, authenticated publisher content such as New York Times subscriptions, websites, and existing YouTube content as first-class content types. The backend and frontend should both operate on a provider-neutral model, and the library should remain clutter-free by grouping subscriptions according to their natural source type. Implementation should begin with a New York Times MVP that proves the authenticated publisher flow end to end before the product targets a second new provider:

- podcast episodes under podcast series
- publications under publication series when a real series exists
- publications under saved-search sources when the subscription is query-backed
- manually tracked websites under a generic `Websites` area with user-created folders

## Implementation Status

Current branch status:

- canonical `source` / `item` / `part` / `media asset` contracts are implemented
- generic `source` / `item` deep links are implemented with legacy `channel` / `video` fallback
- YouTube continues to work through the compatibility layer
- OpenAlex saved-search subscriptions are implemented as a first query-backed non-auth provider
- podcast RSS subscriptions are implemented as a first feed-backed non-auth provider
- manually tracked website pages are implemented as direct tracked sources

Still open in this branch:

- New York Times authenticated publisher flow
- website folder creation, rename, reorder, and assignment UI
- a fully source-native library shell that removes the remaining compatibility dependence on `channel` / `video`

This means the branch has intentionally moved ahead on non-auth providers before the NYT lane is complete. The original sequencing note remains useful as product intent, but it is no longer the literal implementation order in this branch.

## Requirements

- Introduce a provider-neutral canonical model that separates:
  - a subscribed or tracked source
  - an individual content item
  - one or more content parts or media assets derived from that item
- Support at least these source archetypes:
  - podcast series backed by RSS or equivalent feed metadata
  - publication series backed by publisher feeds or provider pages
  - saved-search publication sources backed by a query against a science engine
  - authenticated publisher sources backed by a user login and entitlement-aware provider access
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
- Allow users to connect an authenticated publisher account when a provider requires login to access subscribed content.
- Support New York Times as a first-wave authenticated publisher provider:
  - user can log into their New York Times account from dAstIll
  - dAstIll can verify that the user has an active subscription or other entitlement needed to access content
  - user can subscribe to New York Times content sources and ingest entitled content into the unified library model
- Treat New York Times full-content ingestion as an authenticated browser-session integration, not as an unauthenticated crawler:
  - discovery and metadata lookup may use public NYT surfaces such as section pages, RSS feeds, or official APIs where available
  - full article ingestion must assume an authenticated session is required for at least some content
  - the system must detect and record whether an article was ingested as full text, partial text, or metadata-only because access was not available
- Do not assume the official NYT developer APIs are sufficient for subscriber-only full article ingestion:
  - the MVP may use official APIs or feeds for discovery and metadata enrichment
  - the MVP should plan for article-body extraction to come from the authenticated article-rendering path instead of assuming an API returns the full subscriber article body
- Make the NYT MVP technically and operationally explicit:
  - login should happen in a real browser context that accepts NYT cookies and completes any NYT-managed authentication flow
  - dAstIll should store only the minimum session material needed for continued access, encrypted at rest, with clear expiration and re-auth behavior
  - article ingestion should load the target NYT article with the authenticated session, determine whether the user is entitled to read it, and only then extract the readable article payload into canonical `ContentPart` records
  - if the session is valid but the article still cannot be read or extracted, the item should remain metadata-only and surface the failure reason
- Sequence delivery so the first implementation slice ships a usable New York Times MVP before adding the next non-YouTube provider.
- Preserve a distinction between public content availability and user-entitled content availability so the system can explain why some provider items are visible but not ingestible.
- Require an explicit compliance gate before shipping any NYT full-content ingestion path:
  - if NYT written permission, licensing, or another approved legal basis is required, MVP shipment must not bypass that requirement
  - a technically feasible but non-compliant scraping path is not sufficient for release
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
- Supporting every paid or authenticated publisher in the first pass beyond an initial New York Times implementation.
- Starting implementation work on a second new provider before the New York Times MVP is shipped and verified.
- Assuming that "user has an active NYT subscription" by itself makes automated scraping permissible to ship.
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

The system should explicitly support four distinct source behaviors because they sync and group content differently:

- feed-backed series
  - podcast RSS feeds
  - publication feeds or publisher update feeds
- query-backed series
  - saved searches in OpenAlex, arXiv, Semantic Scholar, or similar engines
  - source identity is the provider plus normalized query definition
- authenticated publisher sources
  - providers such as New York Times where content access depends on the user's account entitlement
  - source identity may depend on both provider metadata and the connected account context
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
- account authentication and session refresh
- entitlement checks for protected content

The important constraint is that provider differences stop at the adapter boundary.

### Authenticated publisher access

Authenticated publishers such as New York Times should plug into the same canonical model rather than forcing a parallel "connected publishers" product area. The provider adapter should own login flow integration, secure session handling, and entitlement-aware sync behavior. Users should be able to connect their New York Times account, browse subscribable New York Times sources within the same library surface, and ingest articles they are entitled to access.

The product should also make entitlement state legible. If a New York Times source exists but the connected account no longer has access, the UI should show that the source is authentication-gated or subscription-gated rather than silently failing.

### New York Times technical feasibility

New York Times support should be specified as a two-layer integration rather than a generic "scrape the site" adapter:

- discovery layer
  - identifies subscribable NYT sources such as sections, newsletters, topics, or author pages
  - can use public NYT surfaces such as section pages, RSS, and official APIs where those surfaces are enough for discovery and metadata
- authenticated ingestion layer
  - uses a user-authorized NYT browser session to open specific article URLs
  - determines whether the connected account is currently entitled to the article
  - extracts normalized article content only after successful access is confirmed

For the MVP, the default assumption should be that official NYT APIs are helpful for discovery and metadata but are not the primary mechanism for obtaining subscriber-only article bodies. The article body path should therefore be modeled as "render the article in an authenticated browser context, then extract the readable content representation."

That extraction path should be concrete:

- user initiates NYT login in a browser context controlled by dAstIll or a tightly coupled companion surface
- NYT completes its own login flow, including cookies and any interactive checks it requires
- dAstIll retains the minimum viable session state needed to revisit article URLs on the user's behalf
- sync opens each target article in the authenticated context
- sync classifies the result as one of:
  - full article accessible and extractable
  - article accessible but extraction failed
  - article not accessible because the user lacks entitlement
  - article blocked by anti-bot, session expiry, or another technical constraint
- only the first case creates a full-text `ContentPart`; the other cases create metadata-only items plus machine-readable failure state

The MVP should not assume a plain backend HTTP client can reliably fetch NYT article HTML. NYT uses anti-bot protections and login-dependent rendering behavior, so a naive server-side scraper is likely to be brittle even before legal review.

### New York Times release gate

There is an important distinction between technical possibility and release viability. Even if dAstIll can technically obtain article content through a user-authenticated session, NYT's published restrictions on automated scraping and data mining mean the team must treat release as gated by compliance review, written permission, licensing, or another clearly approved basis. The spec should therefore treat the NYT MVP as having both:

- a technical proof path
- a compliance approval path

MVP is not complete unless both paths are satisfied.

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

Publisher-backed subscriptions can also be account-backed rather than feed-backed. For example, a New York Times subscription may expose sections, topics, newsletters, author pages, or saved content areas that the user can subscribe to only after logging in.

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

The implementation should be incremental. Existing YouTube functionality should keep working during the migration, but the first new provider implementation should be New York Times. The team should ship a narrow, end-to-end New York Times MVP first, verify that the canonical model and authenticated publisher flow work in production, and only then start the next provider integration. This reduces migration risk, validates the provider-neutral model against the hardest initial case, and keeps the app usable while broader provider coverage is phased in.

The New York Times MVP should be treated as the proving ground for:

- authenticated provider account connection
- entitlement-aware sync
- subscribable provider-backed sources
- ingestion of entitled content into the generic source and item model
- library presentation that makes auth-gated content understandable
- a release checklist that separately verifies technical feasibility and compliance approval

## Open Questions

- Which science engines should be treated as first-wave query-backed publication providers versus enrichment-only providers?
- Which New York Times content scopes should be first-wave subscribable sources: sections, topics, newsletters, saved articles, author pages, or some smaller subset?
- What authentication mechanism should the New York Times integration use in the first implementation slice, and what secure credential/session storage is required on the backend?
- Should the NYT MVP use an embedded browser flow, a browser extension, a local companion process, or another user-session-preserving architecture for authenticated article access?
- What exact extraction target should the NYT MVP trust first on article pages: rendered DOM, structured data, embedded JSON state, or a provider-specific internal payload exposed to the authenticated page?
- What compliance or licensing approval is required before any authenticated NYT article extraction can ship to users?
- What exact exit criteria define the New York Times MVP as shipped, so work on the second provider does not begin prematurely?
- Should the top-level library surface be organized primarily by source type, by user-defined folders, or by a hybrid of both?
- Should manually tracked websites support both folder assignment and ad hoc tags in the first pass, or should tags wait for a later iteration?
- Should the `Websites` area allow direct page tracking only, or also support tracking whole domains as sources with discovered pages beneath them?
- Should website tracking support feed discovery automatically when a site exposes RSS or Atom, or should that wait for a later pass?
- Should imported publication PDFs be stored as first-class assets in the first implementation slice, or should the first pass focus on metadata plus extracted text only?
- Which provider-specific metadata must be exposed in the frontend immediately versus stored only for later use?
