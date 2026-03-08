# Tasks: Summary Processing During Local Fallback

## Current State
Root cause fixed. The evaluator default model name (`qwen3-coder:480b-cloud`) was not recognized as cloud because cloud detection only matched the `:cloud` suffix. That let the evaluator grab the local semaphore as if it were a local model, then hit 429 and attempt local fallback again, which could deadlock or starve summary generation. Cloud detection now accepts both `:cloud` and `-cloud`, and summary evaluation is deferred during cloud cooldown to preserve local capacity for summaries.

## Steps
- [x] Create spec and task tracking files.
- [x] Write failing tests for evaluator deferral during cloud cooldown.
- [x] Implement evaluator deferral so summaries keep local capacity.
- [x] Run verification gates for the regression fix.

## Decisions Made During Implementation
- Treat summary evaluation as lower priority than summary generation when only local fallback remains.
- Fix cloud-model detection centrally instead of special-casing the evaluator, because the `-cloud` naming pattern affected semaphore/cooldown behavior throughout the service layer.
- During cloud cooldown, cloud-primary summary evaluation is treated as unavailable so it does not consume the shared local inference slot needed for summaries.
