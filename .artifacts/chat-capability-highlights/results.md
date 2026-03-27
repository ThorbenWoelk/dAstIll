# Chat Capability Sweep Results

- Generated: `2026-03-27T14:18:24.608647+00:00`
- Base URL: `http://localhost:3544`
- Dataset: `/Users/thorben.woelk/repos/dAstIll/backend/tests/data/chat_capability_prompts.json`
- Prompt count: `2`

## Summary

- Passed prompts: `1/2`
- Answerability pass: `2/2`
- Grounding pass: `1/2`
- Shape pass: `2/2`
- Average score: `2.50`

## Capability Classes

- `highlight_lookup`: passed `1/2`, avg score `2.50`, failures `no_sources`

## Failures By Class

- `no_sources`: q062

## Prompt Results

### q062 FAIL

- Prompt: What highlights have I saved from this video?
- Class: `highlight_lookup`
- Status: `Completed`
- Score: `2`
- Sources: `0`
- Failure: `no_sources`
- Notes: highlight prompt did not use the saved highlights tool

#### Answer

I cannot see your saved highlights because I do not have access to the indexed library or transcript data for this turn. To view your highlights, please ask again to trigger a library search so I can retrieve the relevant information.

### q063 PASS

- Prompt: Show me all highlights related to search.
- Class: `highlight_lookup`
- Status: `Completed`
- Score: `3`
- Sources: `0`
- Failure: `-`
- Tools: Saved highlights lookup (highlight_lookup), Saved highlights lookup (highlight_lookup)

#### Answer

Based on the saved highlights, there is **1 highlight** matching the query "search":

---

**Source:** AI News & Strategy Daily | Nate B Jones / Tobi Lütke Made a 20-Year-Old Codebase 53% Faster Overnight. Here's How.

**Summary highlight:** "Agents" are too often misunderstood as simply LLMs plus tools in a loop; in practice, there are four distinct "species" of agents in production use, each suited to different problems. The four types are: Coding Harnesse...[Tool 1]

---

This appears to be the only saved highlight related to "search" in the system.

