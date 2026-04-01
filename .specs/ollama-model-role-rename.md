# Ollama Model Role Rename

## Problem

`OllamaRuntimeConfig` currently exposes `model` and `chat_model`, while the environment uses
`OLLAMA_MODEL` and `OLLAMA_CHAT_MODEL`. In practice, `model` is the summary-generation model and
`chat_model` is an optional default override for chat. The current names make the ownership of
those roles unclear.

## Goal

Rename the config fields, environment variables, examples, and docs so the summary-generation
model and the default chat model are explicit everywhere they appear.

## Requirements

- Backend config must expose explicit names for the summary-generation model and the default chat
  model.
- Backend startup wiring must read the renamed fields and environment variables consistently.
- Validation messages and tests must refer to the renamed variables.
- Local env templates, startup helpers, deployment workflow config, and checked-in examples must use
  the renamed variables.
- Documentation must describe the renamed model roles consistently.

## Non-Goals

- Changing the runtime behavior of summary generation, chat generation, embeddings, or evaluation.
- Renaming unrelated model roles such as the evaluator, embedding, reranker, or HyDE model.
- Preserving backward compatibility with the old environment variable names.

## Design Considerations

- Use `summary_model` and `default_chat_model` in Rust so the struct reads like the actual runtime
  role split.
- Use `OLLAMA_SUMMARY_MODEL` and `OLLAMA_DEFAULT_CHAT_MODEL` in environment-facing surfaces to keep
  the backend config and deployment docs aligned.

## Open Questions

- None.
