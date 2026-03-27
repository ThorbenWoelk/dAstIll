# Agent Guide

`AGENTS.md` is the agent entry point for this repository.
Keep this file short, operational, and focused on how to work in the repo.
Deeper domain-specific guidance belongs in dedicated docs and should be linked from here.

## Source Of Truth

- Frontend design system, Svelte frontend cleanliness rules, UI architecture guidance, file-size thresholds, and frontend testing expectations live in [DESIGN.md](/Users/thorben.woelk/repos/dAstIll/DESIGN.md).

## How To Work Here

- Read this file first, then open the linked domain doc you need.
- Do not duplicate large guidance blocks across multiple markdown files.
- When frontend rules change, update `DESIGN.md` and keep only the pointer here.
- Keep repo guidance legible for agents: short entry points here, detailed source-of-truth docs elsewhere.

## Documentation Split

- `AGENTS.md`: agent workflow entry point, document map, repo-level instructions.
- `DESIGN.md`: design system and frontend engineering standards.

## Verification

- After code changes, run the relevant local verification steps before treating work as verified.
- For frontend work, follow the verification checklist in [DESIGN.md](/Users/thorben.woelk/repos/dAstIll/DESIGN.md).
