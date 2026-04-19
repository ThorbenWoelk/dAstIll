# ADR: Podcast ASR Uses Source Audio Assets And Operator-Owned STT

## Status

Accepted

## Context

Podcast RSS feeds do not reliably publish transcripts. Some feeds expose
`podcast:transcript` links, but common hosted feeds can expose only show notes
and audio enclosures. Treating show notes as transcripts makes summaries and
search results look complete while grounding them in descriptions rather than
episode content.

Podcast episodes also differ from YouTube videos in one important UI detail:
they can be useful and actionable before a transcript is ready. If the normal
channel list hides every row without a ready transcript, a newly subscribed
podcast appears empty even though sync succeeded and transcription is pending.

## Decision

Podcast feed sync stores source facts first:

- episode metadata
- show notes as description/display content only
- publisher transcript references when present
- RSS audio enclosures as `MediaAssetKind::SourceAudio`

Show notes are never stored as transcripts.

Publisher transcript references are preferred when present. When they are
missing, dAstIll can transcribe the audio enclosure through a local/free
OpenAI-compatible ASR endpoint:

```text
POST {LOCAL_ASR_BASE_URL}/audio/transcriptions
```

The Rust backend is only the ASR client. The STT model runs in a separate
operator-owned ASR service. The recommended free local runtime is the maintained
`whisper.cpp` server with the `base.en` GGML model, but the backend depends only on
the OpenAI-compatible endpoint contract.

Pending podcast episodes are visible in normal channel lists. Non-podcast rows
still require a ready transcript unless the caller explicitly asks for the
transcript queue.

## Alternatives Considered

- **RSS-only transcripts:** Rejected because many feeds do not publish
  `podcast:transcript` links.
- **Show notes as transcript fallback:** Rejected because it poisons summaries,
  search, and chat grounding.
- **Bundling STT into the Rust backend container:** Rejected because model files,
  CPU/GPU pressure, and ASR failures should not share the main API process.
- **Depending on a low-adoption wrapper repo:** Rejected for production. Wrapper
  repos can be local experiments, but production should run a trusted
  implementation behind the endpoint contract.
- **Hide pending podcasts like pending YouTube videos:** Rejected because it
  makes successful podcast subscriptions look empty before ASR runs.

## Consequences

Positive:

- podcast subscriptions show episodes immediately after sync
- summaries and search are grounded in real transcripts, not show notes
- local and production ASR can use the same backend contract
- STT implementation can be replaced without changing dAstIll storage or routes

Tradeoffs:

- production needs an ASR service and runtime configuration before podcast ASR
  is functional
- long podcast episodes can make synchronous transcription slow until durable ASR
  job state is added
- ASR media fetching must stay strict about SSRF, redirects, and byte caps

## Directives

- Keep `VideoInfo` as watch/page metadata. Do not add source audio fields there.
- Store podcast audio as a media asset keyed by episode/video id.
- Keep `LOCAL_ASR_ENABLED=false` as the backend code default. The release workflow may enable the
  repo-owned Cloud Run ASR service explicitly for production.
- Use Cloud Run IAM for the repo-owned production ASR service. Use `LOCAL_ASR_API_KEY` only for local or external bearer-token ASR services.
- Keep podcast pending rows visible in regular channel views so users can see
  that subscription and sync worked.
- Do not introduce low-maintenance ASR wrapper repositories as production
  dependencies.
