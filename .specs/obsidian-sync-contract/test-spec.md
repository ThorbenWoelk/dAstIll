# Obsidian Sync Contract Test Spec

## Acceptance Criteria

- A signed-in user can request Markdown export packets only for items they can access.
- Export packets contain stable frontmatter, managed section markers, summary content, highlight content, and source metadata.
- Export is idempotent for unchanged source content and updates per-user export state.
- A signed-in user can import Markdown notes into private user-scoped storage.
- Import is idempotent for the same vault note ID or stable path-derived ID.
- Imported notes do not mutate canonical transcripts, summaries, highlights, or global source metadata.
- Imported notes are either queued for a user-scoped search projection or explicitly stored with pending index status.
- Anonymous and cross-user requests cannot export, import, or read another user's Obsidian sync state.

## Proof For The Current Increment

1. Add backend unit tests for Markdown packet rendering and parsing.
2. Add backend route tests or handler-level tests for auth, access scoping, idempotency, and storage writes.
3. Add search-boundary tests that prove imported notes do not enter the global source projection.
4. Run backend checks.
5. Exercise the API with a small fixture packet that represents a future Obsidian plugin call.

The increment is proven when an authenticated API call can round-trip:

```text
dAstIll item -> Markdown export packet -> imported vault note record
```

and the stored records remain under the authenticated user's scope.

## Automated Checks

Backend:

- `cd backend && cargo check`
- `cd backend && cargo test`

Focused tests to add:

- Export renderer includes required frontmatter fields.
- Export renderer includes managed markers for summary, highlights, and source metadata.
- Export renderer remains stable when source content is unchanged.
- Export rejects inaccessible item IDs.
- Import parser accepts valid frontmatter and body.
- Import parser accepts notes without dAstIll source IDs.
- Import parser rejects missing stable note identity when no path or note ID is supplied.
- Import rejects oversized note bodies.
- Import stores records under `user-vault-notes/{user_id}/`.
- Re-import with the same content hash does not create duplicates.
- Re-import with changed body updates the existing record and index status.
- Anonymous calls return `403` or the repo-standard auth failure.
- Cross-user access does not return export or import records.

Optional frontend checks only if a UI is added:

- `cd frontend && bun run check`
- `cd frontend && bun run test`

## Manual Checks

Use a local signed-in session or an authenticated test token:

1. Select a known source item with a ready summary and at least one highlight.
2. Call the export endpoint for that item.
3. Confirm the response includes one Markdown packet with:
   - frontmatter
   - source URL or source metadata
   - summary managed block
   - highlights managed block
   - human-owned notes section
4. Call the import endpoint with a note fixture containing frontmatter and body.
5. Confirm the status endpoint reports imported note count and pending or ready index state.
6. Repeat the same import and confirm no duplicate record appears.
7. Change the note body, import again, and confirm the content hash changes.

## Edge Cases

- Export item exists globally but is outside the caller's library scope.
- Export item has no ready summary.
- Export item has no highlights.
- Export item is a YouTube video, OpenAlex publication, podcast episode, or website page.
- Highlight text contains Markdown-sensitive characters.
- Imported note has no dAstIll source reference.
- Imported note has duplicate frontmatter keys.
- Imported note has malformed frontmatter.
- Imported note body is empty.
- Imported note body exceeds the configured limit.
- Imported note path changes but frontmatter ID stays stable.
- Imported note frontmatter ID changes but path stays stable.
- Same vault note ID is imported by two different users.
- Import arrives while search indexing is unavailable.

## Observability Or Failure Signals

- Export logs should include user ID hash or scoped user identifier, item count, and packet count.
- Import logs should include user ID hash or scoped user identifier, note count, updated count, skipped count, and rejected count.
- Logs must not include full note bodies, highlight bodies, or private vault content.
- Failed imports should return structured errors that name the note identity and failure class.
- Index status should show pending, ready, failed, and last error without exposing private note body text.
- Search or chat results must clearly identify imported vault notes as user notes.

## Stop Line

Stop after Phase 1 when:

- Export/import authenticated APIs exist.
- User-scoped storage records exist.
- Markdown rendering and parsing are covered by tests.
- Idempotency and access control are covered by tests.
- Imported notes are isolated from global canonical content and global search projections.
- A fixture-based API round trip works.

Do not build the Obsidian plugin, local filesystem writer, Git sync flow, or full UI in this increment.

