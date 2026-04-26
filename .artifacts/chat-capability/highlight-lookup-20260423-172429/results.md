# Chat Capability Sweep Results

- Generated: `2026-04-23T15:25:33.760196+00:00`
- Base URL: `http://localhost:3544`
- Dataset: `/Users/thorben.woelk/repos/dAstIll/backend/tests/data/chat_capability_prompts.json`
- Prompt count: `6`

## Summary

- Passed prompts: `0/6`
- Answerability pass: `0/6`
- Grounding pass: `6/6`
- Shape pass: `0/6`
- Average score: `0.00`

## Capability Classes

- `highlight_lookup`: passed `0/6`, avg score `0.00`, failures `stream_error`

## Failures By Class

- `stream_error`: q062, q063, q064, q065, q070, q071

## Prompt Results

### q062 FAIL

- Prompt: What highlights have I saved from this video?
- Class: `highlight_lookup`
- Status: `StreamError`
- Score: `0`
- Sources: `0`
- Failure: `stream_error`
- Tools: Saved highlights lookup (highlight_lookup), Saved highlights lookup (highlight_lookup)
- Notes: stream ended with an explicit error event | assistant content was empty | highlight answer did not explicitly reference highlights or snippets

#### Answer

_No assistant content._

### q063 FAIL

- Prompt: Show me all highlights related to search.
- Class: `highlight_lookup`
- Status: `StreamError`
- Score: `0`
- Sources: `0`
- Failure: `stream_error`
- Tools: Saved highlights lookup (highlight_lookup), Saved highlights lookup (highlight_lookup)
- Notes: stream ended with an explicit error event | assistant content was empty | highlight answer did not explicitly reference highlights or snippets

#### Answer

_No assistant content._

### q064 FAIL

- Prompt: Show me all highlights related to summaries.
- Class: `highlight_lookup`
- Status: `StreamError`
- Score: `0`
- Sources: `0`
- Failure: `stream_error`
- Tools: Saved highlights lookup (highlight_lookup), Saved highlights lookup (highlight_lookup)
- Notes: stream ended with an explicit error event | assistant content was empty | highlight answer did not explicitly reference highlights or snippets

#### Answer

_No assistant content._

### q065 FAIL

- Prompt: Show me all highlights related to evaluation.
- Class: `highlight_lookup`
- Status: `StreamError`
- Score: `0`
- Sources: `0`
- Failure: `stream_error`
- Tools: Saved highlights lookup (highlight_lookup), Saved highlights lookup (highlight_lookup)
- Notes: stream ended with an explicit error event | assistant content was empty | highlight answer did not explicitly reference highlights or snippets

#### Answer

_No assistant content._

### q070 FAIL

- Prompt: Find highlights that support a specific claim.
- Class: `highlight_lookup`
- Status: `StreamError`
- Score: `0`
- Sources: `0`
- Failure: `stream_error`
- Tools: Saved highlights lookup (highlight_lookup), Saved highlights lookup (highlight_lookup)
- Notes: stream ended with an explicit error event | assistant content was empty | highlight answer did not explicitly reference highlights or snippets

#### Answer

_No assistant content._

### q071 FAIL

- Prompt: Find highlights that contradict a specific claim.
- Class: `highlight_lookup`
- Status: `StreamError`
- Score: `0`
- Sources: `0`
- Failure: `stream_error`
- Tools: Saved highlights lookup (highlight_lookup), Saved highlights lookup (highlight_lookup)
- Notes: stream ended with an explicit error event | assistant content was empty | highlight answer did not explicitly reference highlights or snippets

#### Answer

_No assistant content._

