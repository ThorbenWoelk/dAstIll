# Spec: Summarizer Prompt Reliability Hardening

## Problem
Summarizer and transcript-clean prompts show low compliance under long Ollama runs, leading to repeated retries and weak summary quality.

## Goals
- Improve transcript-clean compliance with stronger token-preservation instructions.
- Improve summary faithfulness and structure consistency with stricter anti-hallucination instructions.
- Add deterministic tests that guard prompt contracts without requiring live Ollama inference.

## Non-Goals
- Model/provider changes.
- End-to-end flaky tests that call live Ollama.

## Requirements
- Refactor prompt text into dedicated builders so tests can validate them.
- Add tests covering key prompt rules for both summary and transcript-clean flows.
- Keep runtime behavior/API unchanged except for better prompts and reliability.
