# Chat Capability Sweep Results

- Generated: `2026-04-23T15:30:08.056946+00:00`
- Base URL: `http://localhost:3544`
- Dataset: `/Users/thorben.woelk/repos/dAstIll/backend/tests/data/chat_capability_prompts.json`
- Prompt count: `4`

## Summary

- Passed prompts: `4/4`
- Answerability pass: `4/4`
- Grounding pass: `4/4`
- Shape pass: `4/4`
- Average score: `3.00`

## Capability Classes

- `highlight_clustering`: passed `4/4`, avg score `3.00`, failures `-`

## Failures By Class


## Prompt Results

### q066 PASS

- Prompt: Which highlight best captures the video's main point?
- Class: `highlight_clustering`
- Status: `Completed`
- Score: `3`
- Sources: `0`
- Failure: `-`
- Tools: Saved highlights lookup (highlight_lookup), Saved highlights lookup (highlight_lookup)

#### Answer

Retrieved tool evidence for: +{Open source is dead now?} Which highlight best captures the video's main point?

The answer model is unavailable, so this fallback returns the grounded tool results directly.

1. Look up saved highlights for query "Which  best captures the video's main point?":
No saved highlights matched query "Which  best captures the video's main point?".

### q067 PASS

- Prompt: Which highlights are most useful as a quick reference?
- Class: `highlight_clustering`
- Status: `Completed`
- Score: `3`
- Sources: `0`
- Failure: `-`
- Tools: Saved highlights lookup (highlight_lookup), Saved highlights lookup (highlight_lookup)

#### Answer

Retrieved tool evidence for: Which highlights are most useful as a quick reference?

The answer model is unavailable, so this fallback returns the grounded tool results directly.

1. Look up saved highlights for query "Which  are most useful as a quick reference?":
No saved highlights matched query "Which  are most useful as a quick reference?".

### q068 PASS

- Prompt: What are the most interesting snippets I've highlighted across the library?
- Class: `highlight_clustering`
- Status: `Completed`
- Score: `3`
- Sources: `0`
- Failure: `-`
- Tools: Saved highlights lookup (highlight_lookup), Saved highlights lookup (highlight_lookup)

#### Answer

Retrieved tool evidence for: What are the most interesting snippets I've highlighted across the library?

The answer model is unavailable, so this fallback returns the grounded tool results directly.

1. Look up saved highlights for query "What are the most interesting snippets I've ed across the library?":
No saved highlights matched query "What are the most interesting snippets I've ed across the library?".

### q069 PASS

- Prompt: Group my highlights by theme.
- Class: `highlight_clustering`
- Status: `Completed`
- Score: `3`
- Sources: `0`
- Failure: `-`
- Tools: Saved highlights lookup (highlight_lookup), Saved highlights lookup (highlight_lookup)

#### Answer

Retrieved tool evidence for: Group my highlights by theme.

The answer model is unavailable, so this fallback returns the grounded tool results directly.

1. Look up saved highlights for query "Group my  by theme.":
No saved highlights matched query "Group my  by theme.".

