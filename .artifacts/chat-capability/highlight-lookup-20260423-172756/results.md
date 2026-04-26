# Chat Capability Sweep Results

- Generated: `2026-04-23T15:28:23.268803+00:00`
- Base URL: `http://localhost:3544`
- Dataset: `/Users/thorben.woelk/repos/dAstIll/backend/tests/data/chat_capability_prompts.json`
- Prompt count: `6`

## Summary

- Passed prompts: `6/6`
- Answerability pass: `6/6`
- Grounding pass: `6/6`
- Shape pass: `6/6`
- Average score: `3.00`

## Capability Classes

- `highlight_lookup`: passed `6/6`, avg score `3.00`, failures `-`

## Failures By Class


## Prompt Results

### q062 PASS

- Prompt: What highlights have I saved from this video?
- Class: `highlight_lookup`
- Status: `Completed`
- Score: `3`
- Sources: `0`
- Failure: `-`
- Tools: Saved highlights lookup (highlight_lookup), Saved highlights lookup (highlight_lookup)

#### Answer

Retrieved tool evidence for: +{Open source is dead now?} What highlights have I saved from this video?

The answer model is unavailable, so this fallback returns the grounded tool results directly.

1. Look up saved highlights for videos matching "Open source is dead now?":
No saved highlights matched videos matching "Open source is dead now?".

### q063 PASS

- Prompt: Show me all highlights related to search.
- Class: `highlight_lookup`
- Status: `Completed`
- Score: `3`
- Sources: `0`
- Failure: `-`
- Tools: Saved highlights lookup (highlight_lookup), Saved highlights lookup (highlight_lookup)

#### Answer

Retrieved tool evidence for: Show me all highlights related to search.

The answer model is unavailable, so this fallback returns the grounded tool results directly.

1. Look up saved highlights for query "search.":
No saved highlights matched query "search.".

### q064 PASS

- Prompt: Show me all highlights related to summaries.
- Class: `highlight_lookup`
- Status: `Completed`
- Score: `3`
- Sources: `0`
- Failure: `-`
- Tools: Saved highlights lookup (highlight_lookup), Saved highlights lookup (highlight_lookup)

#### Answer

Retrieved tool evidence for: Show me all highlights related to summaries.

The answer model is unavailable, so this fallback returns the grounded tool results directly.

1. Look up saved highlights for query "summaries.":
No saved highlights matched query "summaries.".

### q065 PASS

- Prompt: Show me all highlights related to evaluation.
- Class: `highlight_lookup`
- Status: `Completed`
- Score: `3`
- Sources: `0`
- Failure: `-`
- Tools: Saved highlights lookup (highlight_lookup), Saved highlights lookup (highlight_lookup)

#### Answer

Retrieved tool evidence for: Show me all highlights related to evaluation.

The answer model is unavailable, so this fallback returns the grounded tool results directly.

1. Look up saved highlights for query "evaluation.":
No saved highlights matched query "evaluation.".

### q070 PASS

- Prompt: Find highlights that support a specific claim.
- Class: `highlight_lookup`
- Status: `Completed`
- Score: `3`
- Sources: `0`
- Failure: `-`
- Tools: Saved highlights lookup (highlight_lookup), Saved highlights lookup (highlight_lookup)

#### Answer

Retrieved tool evidence for: Find highlights that support a specific claim.

The answer model is unavailable, so this fallback returns the grounded tool results directly.

1. Look up saved highlights for query "support":
No saved highlights matched query "support".

### q071 PASS

- Prompt: Find highlights that contradict a specific claim.
- Class: `highlight_lookup`
- Status: `Completed`
- Score: `3`
- Sources: `0`
- Failure: `-`
- Tools: Saved highlights lookup (highlight_lookup), Saved highlights lookup (highlight_lookup)

#### Answer

Retrieved tool evidence for: Find highlights that contradict a specific claim.

The answer model is unavailable, so this fallback returns the grounded tool results directly.

1. Look up saved highlights for query "contradict":
No saved highlights matched query "contradict".

