# Changelog

## [Unreleased]

### Changed

- Move channel subscriptions and sign out to a new Profile page, which takes the Sections tab's place in the page bar.
- Replace the sideways "In this story" line with sticky section headings that fold a section away to skip it.

### Fixed

- Keep transcript jobs pending when a local extractor is missing, and discover installed extractor paths during shared-env setup.
- Fetch YouTube transcripts locally through durable GCS jobs while Cloud Run generates and evaluates articles; waiting for the local app no longer blocks ready work or consumes retries.
- Write summary articles as dense newsletter briefs, with substance-first openings, less filler, and an evaluator that checks information coverage rather than length.
- Keep the model's planning notes and abandoned drafts out of generated summaries.
- Update DOMPurify and three Rust crates to releases without known advisories.
- Keep categories on one line with horizontal scrolling at every screen width.
- Restart the local development stack after macOS wakes to recover stale browser connections.
- Update audited Rust dependencies to patched releases.
