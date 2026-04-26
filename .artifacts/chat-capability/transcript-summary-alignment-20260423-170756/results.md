# Chat Capability Sweep Results

- Generated: `2026-04-23T15:08:21.423478+00:00`
- Base URL: `http://localhost:3544`
- Dataset: `/Users/thorben.woelk/repos/dAstIll/backend/tests/data/chat_capability_prompts.json`
- Prompt count: `5`

## Summary

- Passed prompts: `1/5`
- Answerability pass: `4/5`
- Grounding pass: `4/5`
- Shape pass: `1/5`
- Average score: `1.80`

## Capability Classes

- `transcript_summary_alignment`: passed `1/5`, avg score `1.80`, failures `generic_answer, shape_mismatch`

## Failures By Class

- `generic_answer`: q097
- `shape_mismatch`: q021, q095, q096

## Prompt Results

### q021 FAIL

- Prompt: What did I miss if I only read the summary?
- Class: `transcript_summary_alignment`
- Status: `Completed`
- Score: `2`
- Sources: `4`
- Failure: `shape_mismatch`
- Source videos: Apple is paying Google to fix Siri (yes really) | Hacking Claude Code to make it 15x cheaper? 👀👀👀 | My chaotic journey to find the right database | Never mind (OpenAI won again)
- Notes: alignment answer did not explicitly discuss both summary and transcript

#### Answer

Retrieved evidence for: What did I miss if I only read the summary?

The answer model is unavailable, so this fallback lists the highest-ranked saved excerpts instead of synthesizing beyond them.

1. Hacking Claude Code to make it 15x cheaper? 👀👀👀 - Theo - t3․gg: work and if I can watch it while it's working and yell at it when it's wrong, I will. Cool. It seems to have blasted through this much quicker overall. Here we got a type error here. Okay, it seems like it didn't get the new tool call syntax right and because of that things are different. Also, the max steps change wasn't applied correctly either. Um, there are still some type errors that indicate missing changes. Do... [1]
2. My chaotic journey to find the right database - Theo - t3․gg: ile keeping the Dexie local model unchanged. Major explored-but-rejected options include Zero (too early, concerns about open-source status and split-brain schema management), Jazz (painful co-state model, not ready for signed-out experiences), TinyBase (required WebSockets or DIY sync), and Legend State (recommended rolling custom sync). A critical insight for local-first sync is the necessity of soft deletes: you m... [2]
3. Never mind (OpenAI won again) - Theo - t3․gg: el where I can start a run, walk away for hours, and come back to fully working software. I have had runs stay on track for eight plus hours. Insane. A big upgrade is judgment under ambiguity. When prompts are missing details, it makes assumptions shockingly similar to what he would have personally decided on. Test and validation are a massive unlock with clear pass and fail targets. It will iterate for many hours wi... [3]
4. Apple is paying Google to fix Siri (yes really) - Theo - t3․gg: was forced by the courts to support RCS. Kind of. There's an interesting catch with RCS. RCS stands for rich communication services. It's an open standard that anyone can choose to support. There is something missing from the standard though. It does not include encryption. The RCS standard is an unencrypted standard which means any texts you send through RCS were unencrypted. Google doesn't want its users sending u.... [4]

### q095 FAIL

- Prompt: Which summary seems most reliable?
- Class: `transcript_summary_alignment`
- Status: `Completed`
- Score: `2`
- Sources: `11`
- Failure: `shape_mismatch`
- Source videos: Gemini 3.1 Pro is the smartest model ever made | Is gpt-5.1 the best code model ever? | Jira and Linear are legacy software | Microsoft and OpenAI are breaking up? | OpenAI: Trapped in 2nd place | TOON: “JSON for AI” (is it any good?) | Vercel Finally Caught Up | Where Should You Deploy In 2026? | Which browser should you use right now?
- Notes: alignment answer did not explicitly discuss both summary and transcript

#### Answer

Retrieved evidence for: Which summary seems most reliable?

The answer model is unavailable, so this fallback lists the highest-ranked saved excerpts instead of synthesizing beyond them.

1. Vercel Finally Caught Up - Theo - t3․gg / Key Points: marginal [1]
2. Where Should You Deploy In 2026? - Theo - t3․gg: TL;DR For most applications (98%+), serverless deployment options are sufficient and recommended as a starting point; move to VPS only if you encounter specific needs. Top recommendations (S-tier): Vercel for serverless, Railway and Render for VPS — all offer excellent developer experience, reliability, and reasonable pricing. Cloudflare offers the lowest costs due to unique infrastructure (no Docker, uses V8 isolate... [2]
3. Gemini 3.1 Pro is the smartest model ever made - Theo - t3․gg: Bench to measure how well models can name skateboarding tricks based on descriptions, combining niche knowledge with spatial recognition. When first created, Gro 4 had the highest score in the high 70s (75 on most recent run), while GPT5 scored 98 during testing at OpenAI's office. Modern OpenAI models have regressed, with the highest current score around 87. Gemini 3.1 Pro Preview "consistently hits 100%" on this b.... [3]
4. Jira and Linear are legacy software - Theo - t3․gg: workspace, and Linear can intelligently refine, synthesize, or take action on context immediately Automations and Non-Developer Adoption The speaker initially overlooked automations in the Codex app and notes most developers have as well Example automations shown: summarize yesterday's Git activity for standup, synthesize weekly PRs/rollouts/incidents/reviews into updates, draft release notes for merged PRs The spea.... [4]
5. Microsoft and OpenAI are breaking up? - Theo - t3․gg / Key Points: lies. [5]
6. TOON: “JSON for AI” (is it any good?) - Theo - t3․gg: to write a plan, pasted two errors in and got a working implementation. I think I came around to tune in the end here. I'm curious what y'all think though. Is this overrated, not worthwhile, or actually a pretty useful little tool to use alongside your LLMs? Let me know what you guys think. Until next time, he starts. [6]
7. Which browser should you use right now? - Theo - t3․gg / Overview: This extensive video is a comprehensive review of the current browser landscape, covering major browsers (Chrome, Edge, Firefox, Safari), privacy-focused alternatives (Brave, Vivaldi, Orion), AI-focused browsers (Dia, Comet), and emerging projects (Zen, Helium, Ladybird). The speaker, a notorious browser-hopper who previously championed Arc, systematically evaluates each browser's strengths, weaknesses, UX decisions,... [7]
8. OpenAI: Trapped in 2nd place - Theo - t3․gg: TL;DR OpenAI consistently releases groundbreaking AI capabilities that briefly put them in first place, but competitors quickly catch up or surpass them, leaving OpenAI in "perpetual second place" across most technical categories. OpenAI's true competitive moat is ChatGPT itself—the default AI chat application for most users—which generates 70% of their revenue and keeps users from switching to technically superior a... [8]
9. Which browser should you use right now? - Theo - t3․gg: TL;DR Chrome/Chromium has had a massively positive impact on web standards and is technically the best implementation, but Google's monopolistic tendencies show in forced AI integrations like Gemini. Manifest V3 was the right call for security (preventing malware), not an anti-ad-blocker move, though ad-blocking is now slightly worse in Chrome. Brave is strongly criticized for buggy UX, breaking websites, aggressive ... [9]
10. Is gpt-5.1 the best code model ever? - Theo - t3․gg / Key Points: model remains the default for planning, but overall disappointment is clear. [10]
11. Which browser should you use right now? - Theo - t3․gg / Takeaways: Chrome's dominance has been net positive for web standards, but Google's monopolistic behavior shows in forced AI integrations; the browser itself remains solid for everyday use. Privacy claims in browsers often don't match reality—Firefox quietly removed its "never sell your data" promise, and Brave's aggressive privacy features actively break websites. Manifest V3 was necessary for security (preventing malware from... [11]

### q096 FAIL

- Prompt: Which summary seems least reliable?
- Class: `transcript_summary_alignment`
- Status: `Completed`
- Score: `2`
- Sources: `7`
- Failure: `shape_mismatch`
- Source videos: Gemini 3.1 Pro is the smartest model ever made | Is gpt-5.1 the best code model ever? | Jira and Linear are legacy software | This awesome CSS feature is blocked by drama (Google and Apple can't agree) | Vercel Finally Caught Up | What is Theo's Worst Take? | Where Should You Deploy In 2026?
- Notes: alignment answer did not explicitly discuss both summary and transcript

#### Answer

Retrieved evidence for: Which summary seems least reliable?

The answer model is unavailable, so this fallback lists the highest-ranked saved excerpts instead of synthesizing beyond them.

1. Where Should You Deploy In 2026? - Theo - t3․gg: d locations, and no serious company uses them. Fly.io has world-class DX and unique features (Elixir-native, Flame), but database reliability issues and financial instability make it risky. AWS (EC2/Lambda) is reliable but expensive, difficult to set up, and has poor DX — only choose if your employer already chose it for you. Digital Ocean has excellent documentation but feels lost strategically and may be "circling.... [1]
2. Vercel Finally Caught Up - Theo - t3․gg / Key Points: marginal [2]
3. What is Theo's Worst Take? - Theo - t3․gg / TL;DR: A speaker is asked to identify their "worst take" The speaker initially claims all of their takes are good, which is suggested might itself be their worst take A past "evil" statement about something called "go" is referenced but not detailed The speaker criticizes a storybook as "useless" Another speaker defends the storybook's value for filling up a "known module" [3]
4. Gemini 3.1 Pro is the smartest model ever made - Theo - t3․gg: I. The CLI has a "potential loop was detected" hook because models loop and fail so frequently. The presenter describes the CLI as "legitimately unusable." File Handling and Basic Operations Problems The model seems "hardcoded" to only read 100 lines at a time, requiring multiple read operations for longer files (reading lines 1-100, then 101-200, etc.). It frequently fails to edit files it just read, passing "bad sy... [4]
5. This awesome CSS feature is blocked by drama (Google and Apple can't agree) - Theo - t3․gg / Key Points: item numbering appeared scattered). Keyboard navigation could be problematic. Both proposals address this with a proposed `reading-flow` property to ensure accessible navigation. Poll Results A community poll asking preference between approaches showed approximately 80% favoring Google's `display: masonry` approach. The presenter also concluded preferring Google's approach, particularly after realizing the named area... [5]
6. Jira and Linear are legacy software - Theo - t3․gg: rst pass BE the plan Similar pattern happened with MCP: it was supposed to be the best way for models to access data, but "it sucked"—until models could use code to use MCP, then it became much better and more reliable The speaker predicts the same pattern: "we're going to reinvent plans a million fucking times over the next year. And then we're going to just go back to code" Code as Planning The speaker advocates fo... [6]
7. Is gpt-5.1 the best code model ever? - Theo - t3․gg / Key Points: model remains the default for planning, but overall disappointment is clear. [7]

### q097 FAIL

- Prompt: What evidence in the transcript supports the summary?
- Class: `transcript_summary_alignment`
- Status: `Completed`
- Score: `0`
- Sources: `0`
- Failure: `generic_answer`
- Notes: assistant answer was too short for the expected prompt type (75 chars) | no grounding sources were attached | alignment answer did not explicitly discuss both summary and transcript

#### Answer

I can’t answer that from the currently indexed transcripts and summaries.

### q098 PASS

- Prompt: Does the summary miss anything important from the transcript?
- Class: `transcript_summary_alignment`
- Status: `Completed`
- Score: `3`
- Sources: `4`
- Failure: `-`
- Source videos: Delete your CLAUDE.md (and your AGENT.md too) | Trump just announced his AI plan. It’s weird. | What’s a Hard Fork?

#### Answer

Retrieved evidence for: Does the summary miss anything important from the transcript?

The answer model is unavailable, so this fallback lists the highest-ranked saved excerpts instead of synthesizing beyond them.

1. Delete your CLAUDE.md (and your AGENT.md too) - Theo - t3․gg: ols (bash calls, grep, file reading); they're good at finding information independently. If information is in the codebase (package.json for commands, file structure for architecture), the model can find it—it doesn't need to be duplicated in a context file. The speaker's experiment: running `/init` on a project called "Lawn" generated a claude.md with architecture, commands, key patterns, etc. The agent read the pac... [1]
2. What’s a Hard Fork? - Hard Fork / At a glance: The video title asks "What’s a Hard Fork?", but the transcript contains no substantive content on this topic. The provided transcript is solely an automated speech recognition (ASR) system metadata note. The text indicates it originated from a local OpenAI-compatible ASR endpoint, not from official RSS show notes. [2]
3. What’s a Hard Fork? - Hard Fork / Takeaways: The intended educational content regarding what a hard fork is could not be summarized, as the transcript contains only ASR metadata and no actual discussion. [3]
4. Trump just announced his AI plan. It’s weird. - Theo - t3․gg: worked hard on a lot of different sides to solve these problems. But it does show that this works. What a ride that one was. From woke AI to Chinese infrastructure to so much more. There's a lot of good in this but also some concerning pieces. I am hopeful that this does set the US in the right direction. And I'm thankful that this at least seems like it was written by people who understand what AI is and what risks ... [4]

