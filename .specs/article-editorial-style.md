# Article editorial style

## User story

As a reader, I want a dense newsletter-style article that gives me the source's
ideas, evidence, and limitations without spending space describing the recording.

## Acceptance

- Lead with the strongest supported finding or argument, including in Overview.
- Preserve distinct claims, mechanisms, useful examples, numbers, and uncertainty.
- Omit recording logistics, greetings, repeated explanations, and clear ads.
- Use plain prose, concrete headings, and no hype or generic closing advice.
- Keep the four existing sections and vocabulary correction behavior.
- Judge completeness by substantive coverage, not word count or every aside.
- Do not weaken faithfulness or invent practical advice to fill a section.

## Verification

- Check the actual generation request carries the editorial system instruction.
- Check prompt guidance at all transcript length tiers and evaluator alignment.
- Run backend checks and tests; reinstall the macOS launcher.
- Smoke-test the configured model with a workshop fixture containing logistics,
  measured results, a limitation, and a sponsor segment when it is reachable.

## Results

- `cargo check` passed; 655 backend tests passed (8 opt-in tests excluded).
- The live workshop regression passed with `glm-5.3:cloud`. It preserved the
  measurements and limitations and omitted recording logistics and the sponsor.
- The configured `gemma4:31b-cloud` evaluator returned 10/10 on both axes for
  that fixture after its prompt explicitly required raw JSON and exact score keys.
- Rust formatting and diff whitespace checks passed. The macOS launcher was reinstalled.
- Before shipping, frontend formatting, lint, type checks, 85 unit tests, 29 E2E tests,
  production build, and production dependency audit passed. Backend audit passed
  with its two existing allowed warnings. macOS launcher tests and compilation passed.
- Two local activation attempts with `start_app.sh --detach` exceeded the
  30-second backend readiness deadline. This change does not edit startup settings.

This fixture checks the reported failure; it does not establish quality on every topic or length.
