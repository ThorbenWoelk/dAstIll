# PRD: Obsidian Sync Contract

## Problem

dAstIll and the user's Obsidian vault both hold useful knowledge, but they serve different jobs.

dAstIll is a multi-user web app. It owns ingestion, canonical source records, transcripts, summaries, highlights, user-scoped library state, and search projections across authenticated users.

The Obsidian vault is a personal Markdown workspace. It owns the user's writing, links, categories, and long-lived notes. It is synced through GitHub outside the app.

The two systems need a bridge that keeps these boundaries clear. Replacing dAstIll storage with Obsidian would break the multi-user product model. Treating Obsidian as a dump target only would miss the value of the user's written notes.

## Goal

Create the first implementation-ready contract for a per-user Obsidian bridge:

- Export selected dAstIll source material into stable Markdown packets.
- Import selected Obsidian notes into dAstIll as private, user-scoped note records.
- Preserve dAstIll canonical storage and search architecture.
- Keep the first increment usable by a CLI or future Obsidian plugin.

## Current Increment

**Phase 1: Per-user Obsidian export/import API contract**

Implement the smallest useful backend slice:

- Define user-scoped storage records for exported source-note sync state and imported vault notes.
- Add authenticated API endpoints for source-note export and vault-note import.
- Render dAstIll source packets as deterministic Markdown with frontmatter and managed block markers.
- Parse imported Markdown frontmatter and body into private user-scoped records.
- Mark imported notes for indexing through a separate user-scoped search path or a clearly isolated pending queue.

## Clear Deliverable

A signed-in user can call backend APIs to:

1. Export one or more accessible dAstIll items as Markdown packets with stable IDs, source metadata, summary content, and highlights.
2. Import one or more Markdown notes from an Obsidian vault into private user-scoped app storage.
3. Re-run export or import without duplicating records when stable IDs and content hashes match.

The deliverable is API-first. A local CLI or test fixture may exercise the contract. A production Obsidian plugin is not required in this increment.

## Non-Goals

- Do not replace Turso, S3 data storage, or S3 Vectors with Obsidian.
- Do not write directly to a local filesystem from the web backend.
- Do not give the backend direct GitHub write access to the user's vault.
- Do not build the Obsidian plugin in this increment.
- Do not import vault notes into global canonical content.
- Do not expose private imported notes to other users.
- Do not redesign the workspace UI.
- Do not change the existing transcript, summary, highlight, chat, or source ingestion flows except where needed to read export data.

## Users Or Actors

- Signed-in dAstIll user who wants their source reading and highlights in Obsidian.
- Same user writing personal notes in Obsidian and wanting those notes available to dAstIll search and chat.
- Future Obsidian plugin or CLI acting on behalf of the signed-in user.
- Backend worker maintaining private imported-note indexing.
- Operator debugging sync state and user-scoped storage.

## Requirements

### Storage Boundaries

- Canonical source content stays in existing dAstIll stores.
- Existing summaries, transcripts, source metadata, and highlights remain the source for export packets.
- Imported Obsidian notes must be stored under a user-specific prefix, for example `user-vault-notes/{user_id}/`.
- Export sync metadata must be stored under a user-specific prefix, for example `user-obsidian-exports/{user_id}/`.
- Imported notes must never be indexed into the shared global `search-chunks/` projection without a user-scope filter that is enforced at retrieval time.

### Export Contract

- Export accepts only source items the authenticated user can access.
- Export returns Markdown packets, not raw filesystem writes.
- Each packet includes frontmatter with:
  - stable dAstIll item ID
  - provider
  - source ID
  - item kind
  - title
  - canonical URL when available
  - content hashes for managed sections
  - export timestamp
- Markdown body includes managed block markers for app-owned sections:
  - summary
  - highlights
  - source metadata
- Markdown body includes a human-owned note section that the app must not overwrite.
- Export should be deterministic for unchanged source data except for explicit export timestamp fields.
- Export state should record the last exported hash per user and item.

### Import Contract

- Import accepts Markdown packets from a trusted client acting as the signed-in user.
- Import parses frontmatter and body using a structured parser or a constrained parser with tests.
- Imported records store:
  - vault note ID or path
  - title
  - body text
  - frontmatter metadata
  - content hash
  - created/updated timestamps from payload when available
  - import timestamp
- Import is idempotent by `(user_id, vault_note_id)` or a stable path-derived ID.
- Import must support notes that have no dAstIll source ID.
- Import must reject oversized notes with a clear error.
- Import must treat note body and frontmatter as untrusted user content.

### Search And Chat Boundary

- Imported notes should become searchable only inside the owning user's scope.
- Retrieval must keep imported vault notes separate from global source chunks in attribution.
- Chat prompts must label imported vault notes as user notes, not source transcripts or summaries.
- If the first increment does not fully index imported notes, it must persist a pending state and expose enough status to prove isolation.

### API Shape

Candidate endpoints:

- `POST /api/me/obsidian/export`
- `POST /api/me/obsidian/import`
- `GET /api/me/obsidian/status`

The final names may follow existing route conventions, but the API must stay explicitly authenticated and user-scoped.

### Conflict Rules

- App-owned managed blocks may be refreshed by export.
- Human-owned sections are outside backend overwrite scope because the backend returns packets instead of writing files.
- Import does not mutate canonical summaries, transcripts, or highlights.
- If an imported note references a dAstIll item, that relationship is stored as metadata only.

### Roadmap After This Increment

- Obsidian plugin that writes export packets into the local vault and posts changed notes back to dAstIll.
- User UI for choosing which items to export.
- User UI for viewing import/index status.
- Private note search and chat citations with vault backlinks.
- Optional one-way vault export CLI for local testing and power users.

## Risks And Open Questions

- User-scoped semantic retrieval needs a clear implementation path. The current global search projection is built around source chunks and S3 Vectors keyed by chunk ID.
- S3 Vectors metadata filtering by user scope must be verified before private vault notes use semantic search in production.
- Markdown frontmatter parsing can become fragile if implemented with ad hoc string rules.
- Imported notes may contain secrets or personal data. Logs must avoid note body content.
- Obsidian file paths can change. Stable note IDs need either frontmatter IDs or a path migration strategy.
- Exporting summaries and highlights can create Git churn in the vault if timestamps or formatting are unstable.
- The Obsidian plugin auth model remains open. It should avoid storing long-lived bearer tokens in plaintext.

