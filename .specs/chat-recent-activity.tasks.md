# Tasks: Chat Recent Activity

## Current State
The backend now resolves unambiguous plain channel/video references, detects recent-activity prompts, and uses a dedicated `recent_library_activity` tool before the generic search path. Verification is complete through lint, tests, release build, and a live chat request against the local app.

## Steps
- [x] Inspect current chat planner, scope resolution, and tool loop behavior.
- [x] Create spec and task tracking files.
- [x] Add recent-activity intent and plain-text scope inference.
- [x] Implement the `recent_library_activity` chat tool and status output.
- [x] Update synthesis behavior for recent prompts and unsupported real-time prompts.
- [x] Add tests for scope, intent, tool parsing, and recent retrieval.
- [x] Run lint, tests, release build, and manual backend verification.

## Decisions Made During Implementation
- Initial implementation scope is channel-focused recent activity only.
- Recent prompts default to library recency, not external live knowledge.
- Plain entity auto-scope only happens for exact unambiguous matches.
- Recent prompts with scoped videos do not auto-upgrade into channel-recent behavior yet; only channel-scoped recent activity is implemented directly.
