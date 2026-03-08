# Evaluator Cloud Policy

## Problem
Summary quality evaluation should only run on sufficiently large cloud models, but the current configuration path can still accept local or undersized evaluator models.

## Goal
Enforce that summary evaluation is always configured to use cloud models with more than 40B parameters.

## Requirements
- Startup rejects invalid `SUMMARY_EVALUATOR_MODEL` values.
- Accepted evaluator models must be cloud models and advertise a parameter size greater than 40B.
- Summary evaluation does not use any local fallback model.
- Regression tests cover the evaluator model policy.

## Non-Goals
- Changing summary generation model policy.
- Auto-selecting a replacement evaluator model.
- Estimating real parameter counts from remote metadata.

## Design Notes
Validate the evaluator model name at startup with a deterministic parser that recognizes size markers like `72b` or `480b`. Treat malformed or size-less names as invalid.
