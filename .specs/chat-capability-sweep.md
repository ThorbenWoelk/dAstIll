# Chat Capability Sweep

## Problem

The chat backend has broad retrieval, synthesis, and tool-loop behavior, but it lacks a durable way to measure how well it handles the full range of realistic user questions we expect in dAstIll. Ad hoc spot checks are not enough to identify recurring failure classes or verify that retrieval and answer generation improve over time.

## Goal

Build a backend-level capability sweep that runs 100 canonical prompts through the real chat API, captures full streamed results, grades each answer against a prompt-specific rubric, clusters failures by capability class, and supports targeted reruns after backend changes.

## Requirements

- Store the canonical 100-prompt set in repo-tracked test data.
- Include, per prompt:
  - `id`
  - `prompt`
  - `search_strategy_expected`
  - `answerability_expected`
  - `good_answer_shape`
  - `capability_class`
  - `requires_timestamp`
  - `requires_highlights`
  - `requires_quality_score`
  - `requires_cross_video_synthesis`
  - `requires_opinion_inference`
- Add a backend executable that:
  - creates a fresh conversation per prompt by default
  - sends prompts through `/api/chat/conversations/{id}/messages`
  - parses SSE `status`, `sources`, `token`, `done`, and `error` events
  - stores one structured result row per prompt
  - emits aggregate JSON and Markdown reports
- Grade answers on:
  - answerability
  - grounding
  - shape
  - capability quality score
- Support rerunning a filtered subset by capability class or prompt id.
- Surface enough debug detail to explain failures:
  - tool usage
  - status trace
  - source count
  - source diversity
  - latency breakdown

## Non-Goals

- Browser-driven execution for all 100 prompts.
- Building a generic benchmark framework for arbitrary models or external corpora.
- Replacing the chat tool loop or retrieval stack wholesale.

## Design

### Canonical dataset

The prompt dataset lives in `backend/tests/data/chat_capability_prompts.json` and acts as the source of truth for the sweep.

### Runner

The runner lives in `backend/src/bin/chat_capability_eval.rs` and uses the public HTTP API so it exercises the same backend path as the product frontend.

### Outputs

Generated artifacts live under `.artifacts/chat-capability/`:

- `results.json`
- `results.md`
- `failures-by-class.json`

### Scoring

The runner uses a conservative automated rubric:

- answerability pass
- grounding pass
- shape pass
- capability score `0..3`

The final report summarizes pass rates by prompt and by capability class, plus common failure modes such as missing sources, single-video overconcentration, or generic unsupported answers.

## Remediation Loop

1. Run the full sweep.
2. Cluster failures by capability class.
3. Add tests for the highest-impact failure class.
4. Implement the minimum backend fix.
5. Rerun the affected class.
6. Repeat until the failure class stabilizes.
7. Rerun the full sweep.

## Verification

- `cargo test`
- `cargo build --release`
- `cargo run --bin chat_capability_eval -- --help`
- real sweep execution against the local backend with current library data
