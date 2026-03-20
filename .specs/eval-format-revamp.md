# Spec: Eval Format Revamp

## Goal
Change the summary evaluation result format from a generic string to a concise, poignant, markdown-formatted note with bullet points, categorized by failure modes (e.g., hallucinations, omissions, factual errors).

## Context
The current evaluation system uses an LLM-as-a-judge to score summaries against transcripts. The evaluator returns a score (0-10) and an `incoherence_note`. This note is currently a brief, often unstructured string. We want to improve the utility of this note by making it more structured and readable for the user.

## Requirements

### 1. Markdown Formatting
The `quality_note` (derived from `incoherence_note` in the evaluator response) must be formatted using Markdown.

### 2. Categorization
Issues found in the summary should be categorized. Suitable categories include:
- **Hallucinations**: Claims not supported by the transcript.
- **Factually Incorrect**: Specific errors in names, numbers, or facts.
- **Omissions**: Significant topics or arguments from the transcript missing from the summary.
- **Generic/Vague**: Statements that are too broad or lack substance.

### 3. Bullet Points
Each point within a category should be a bullet point.

### 4. Conciseness
The notes should be "poignant" - direct and to the point.

## Example Output

**Good Example:**
```markdown
Factually incorrect: 
- website is 'bugsapplesloves.com', not 'bugs.apple.com'

Omissions:
- The detailed explanation of the memory leak in the secondary process.
```

**Bad Example:**
```text
It also misattributes 'bugs.apple.com' (the website is 'bugsapplesloves.com'). It also missed the memory leak part.
```

## Technical Changes

### Backend
- Update the prompt in `backend/src/services/summary_evaluator.rs` to request the new format.
- Ensure the JSON schema for the evaluator response can accommodate the potentially longer, multi-line markdown string.
- (Optional) Update the JSON field name if `incoherence_note` no longer feels appropriate, though keeping it simplifies DB migrations if it's already persisted.

### Frontend
- Update `WorkspaceSummaryMeta.svelte` to use `renderMarkdown` for the `quality_note`.
- Ensure the styling supports markdown elements like lists.
- If the frontend currently expects a single line or trims whitespace excessively, it may need updates to preserve markdown formatting.

## Verification Plan
- Unit tests for the evaluator prompt.
- Manual verification of evaluation results for known "bad" summaries.
- UI check to ensure markdown rendering is correct.
