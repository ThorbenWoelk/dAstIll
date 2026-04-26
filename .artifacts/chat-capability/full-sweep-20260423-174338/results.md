# Chat Capability Sweep Results

- Generated: `2026-04-23T15:52:08.778826+00:00`
- Base URL: `http://localhost:3544`
- Dataset: `/Users/thorben.woelk/repos/dAstIll/backend/tests/data/chat_capability_prompts.json`
- Prompt count: `100`

## Summary

- Passed prompts: `100/100`
- Answerability pass: `100/100`
- Grounding pass: `100/100`
- Shape pass: `100/100`
- Average score: `3.00`

## Capability Classes

- `direct_lookup`: passed `20/20`, avg score `3.00`, failures `-`
- `topic_aggregation`: passed `5/5`, avg score `3.00`, failures `-`
- `cross_video_synthesis`: passed `9/9`, avg score `3.00`, failures `-`
- `comparison`: passed `11/11`, avg score `3.00`, failures `-`
- `recommendation`: passed `10/10`, avg score `3.00`, failures `-`
- `creator_stance`: passed `7/7`, avg score `3.00`, failures `-`
- `highlight_lookup`: passed `6/6`, avg score `3.00`, failures `-`
- `highlight_clustering`: passed `4/4`, avg score `3.00`, failures `-`
- `transcript_summary_alignment`: passed `5/5`, avg score `3.00`, failures `-`
- `timestamp_navigation`: passed `8/8`, avg score `3.00`, failures `-`
- `tone_or_style_inference`: passed `10/10`, avg score `3.00`, failures `-`
- `meta_learning_or_next_step`: passed `5/5`, avg score `3.00`, failures `-`

## Failures By Class


## Prompt Results

### q001 PASS

- Prompt: What topics come up most across my library?
- Class: `topic_aggregation`
- Status: `Completed`
- Score: `3`
- Sources: `12`
- Failure: `-`
- Source videos: AI has rewired my brain | AI is ruining the job market | Cursor 1.0: Moving Past The Fork | How JS ruined the web | I can't believe he was right. | I don’t really use libraries anymore | OpenAI is in "CODE RED" (Did Gemini win that hard??) | The Future of TypeScript

#### Answer

Retrieved evidence for: What topics come up most across my library?

The answer model is unavailable, so this fallback lists the highest-ranked saved excerpts instead of synthesizing beyond them.

1. AI has rewired my brain - Theo - t3․gg: ithout AI. Absolutely. The difference is they will sound much more intelligent and they'll use a bunch of the right words now when they wouldn't have before because the AI will help them use the right wording. Most opinions are justifiable with the wrong information. And AI makes it so easy to find or even make up that information. And that's a genuine disaster for information economy and like the state of truthiness... [1]
2. I don’t really use libraries anymore - Theo - t3․gg / TL;DR: AI tools are fundamentally changing the calculus of when to use external libraries versus implementing solutions yourself, making it easier to "vibe code" alternatives. The speaker is actively removing libraries from projects when they cause problems, finding it often easier to rewrite functionality than fight with problematic dependencies. Libraries fall into categories: those beyond your knowledge (beginner-level p... [2]
3. I don’t really use libraries anymore - Theo - t3․gg / Key Points: level, increasing after a Christmas slump `leftpad` has weird spikes (people download it as a meme), but overall downloads are going up over time This is counterintuitive—while the need to install these has decreased (you can vibe code alternatives), downloads are increasing because more people are building things with AI assistance and may not know better. The speaker notes `leftpad` functionality is now built into ... [3]
4. AI is ruining the job market - Theo - t3․gg: TL;DR A Harvard-commissioned study across 250,000 firms confirms junior roles are decreasing while senior roles are increasing, with early career (ages 22-25) employment dropping notably as AI proliferation accelerated. Companies aren't hiring juniors to save money—juniors actually cost more when factoring in management overhead, slower output, and the need for senior supervision; the real value of juniors was always... [4]
5. How JS ruined the web - Theo - t3․gg: nowledges some valid criticisms (over-engineering in enterprise, unnecessary React usage for simple blogs) but attributes these to poor engineering choices and cultural problems, not the tools themselves. The "most websites" argument is flawed because most websites by URL count are abandoned, low-traffic pages; the real measure should be developer hours and user time spent, where modern frameworks dominate. Overview.... [5]
6. I don’t really use libraries anymore - Theo - t3․gg: into the field and are adopting these things. I would guess, I'll go check, but I would honestly guess these libraries are probably being installed more than ever, not less than ever, simply because of the popularity of coding going up as a result of these AI tools. Let's see if my theory here is right. Is odd has maintained roughly where it was, but it is going back up now after the Christmas slump. Yeah, downloads ... [6]
7. I can't believe he was right. - Theo - t3․gg: tions over 30 days (259 PRs) using Claude Code and Opus, without opening a traditional editor. Google reports 25%+ of code is AI-written; Microsoft reports ~30%; 32% of senior devs say at least half their code comes from AI—senior devs are adopting these tools fastest. The role of developers is shifting from writing code to reviewing and orchestrating AI-generated code, similar to how engineers transition to manageme... [7]
8. I don’t really use libraries anymore - Theo - t3․gg / Key Points: for understanding different library types: **Libraries beyond your knowledge**: These are used by people who don't know how to solve the problem themselves. Examples include `is-odd` (literally one line of code) and `leftpad`. The argument against these is that users are outsourcing competency and taking on supply chain risks without understanding them. **Libraries for tedious reimplementation**: Even capable develop... [8]
9. OpenAI is in "CODE RED" (Did Gemini win that hard??) - Theo - t3․gg: nch and open-weight models like Deepseek have intensified competitive pressure, with ChatGPT traffic reportedly declining 6-7% recently according to Similar Web analytics. OpenAI faces structural disadvantages across four competitive verticals: they don't make their own chips (unlike Google), can't compete in advertising revenue (unlike Google and Meta), and no longer holds a clear "best model" position in any catego... [9]
10. The Future of TypeScript - Theo - t3․gg: the future default. Overview The video provides an extensive deep dive into the future of TypeScript, focusing on the recently announced TypeScript 6 beta and the ongoing port of TypeScript to Go (which will become TypeScript 7). The speaker explains the historical context of TypeScript's creation at Microsoft scale, traces how TypeScript expanded JavaScript's viability across more application domains, and details wh... [10]
11. Cursor 1.0: Moving Past The Fork - Theo - t3․gg: Bugbot provides automatic code reviews that leave comments on GitHub. A notable feature is a "fix in Cursor" button that returns users to the editor with an auto-populated prompt to fix flagged issues. Bugbot comes with a 7-day free trial and is a separate product from the core Cursor subscription. AI Code Review Landscape** The creator strongly endorses AI code review, citing positive experiences with Code Rabbit a.... [11]
12. AI has rewired my brain - Theo - t3․gg: on first, hitting its limitations quickly, and then replacing it with the right complex solution—AI makes both the initial implementation and the sledgehammer replacement dramatically cheaper. Monorepos have become problematic for AI tools; instead, he's finding success with well-isolated microservices and keeping frontend/backend in the same repository for type-safety feedback loops. Code has become "cheap"—rewritin... [12]

### q002 PASS

- Prompt: Which recent uploads are most worth watching first?
- Class: `recommendation`
- Status: `Completed`
- Score: `3`
- Sources: `12`
- Failure: `-`
- Source videos: Breaking up with Vercel | Defending a disaster (modern frontend development rant) | My favorite browser is (kind of) dead | Serverless: A Comprehensive Breakdown | So I've had gpt-5 for a bit now... | The real reason Tea got hacked (it's NOT vibe coding) | We stopped using serverless. The results are insane. | What happened to me? | You suck at picking projects

#### Answer

Retrieved evidence for: Which recent uploads are most worth watching first?

The answer model is unavailable, so this fallback lists the highest-ranked saved excerpts instead of synthesizing beyond them.

1. What happened to me? - Theo - t3․gg: this can change the same way it changed here. If I start doing more of these types of videos and they perform better than expected, I'll lean more into this. For example, the logging video, we screwed up the export initially and the first version that went up was too short and had most of the content missing. So, we had to re-upload it, which destroys the video performance because a lot of people already saw it, so t... [1]
2. We stopped using serverless. The results are insane. - Theo - t3․gg / Key Points: nd-trip API call to determine where to upload. If a user changes regions for their app, they must also change the UploadThing token. Legacy infrastructure from the initial launch is being sunset; this caused a recent outage mentioned in the video. The new architecture decouples from S3-specific functionality, making storage backend more flexible. Bundle Size Reductions Client-side JS bundle was reduced by over 30% in... [2]
3. What happened to me? - Theo - t3․gg: of my community, the people hanging out in Twitch chat right now who have been there since day one, watched this whole thing happen, many of which joined me on the same journey. So, knowing all of this, I want to talk a bit about how I choose a topic for a video because another one of the questions I get all of the time is, "What is your dream video that you would do if the algorithm wouldn't kill it?" A question I g... [3]
4. So I've had gpt-5 for a bit now... - Theo - t3․gg / Key Points: steering. The knowledge cutoff appears to be recent, and it picks up on patterns very well. It does exactly what system prompts instruct—better than anything else the creator has used. Coding Capabilities The creator built most of Skatebench with GPT-5, and it built all demo components shown in the video "first try with no issues." It demonstrated excellent tool-calling behaviors throughout. When given a complex feat... [4]
5. So I've had gpt-5 for a bit now... - Theo - t3․gg: things too. It's knowledge cutoff seems to be pretty recent and it seems to pick up on patterns really well. It does exactly what the [ __ ] you tell it to through the system prompt. better than like anything else I've ever used. Okay, looks like it's done. Looks like it wasn't a big change. Cool. Let's uh rerun this and see how it does. Also, by the way, looks like Cloud 4 Opus still recommends upload thing the most... [5]
6. We stopped using serverless. The results are insane. - Theo - t3․gg: TL;DR UploadThing V7 delivers up to 5x faster uploads, with benchmarks showing improvements from ~5 seconds to ~1.5 seconds for multiple files and ~4 seconds to ~0.5 seconds for single small files. The architecture shifted away from direct S3 uploads to using a custom ingest server, reducing network hops from 7 to 3 and eliminating the need for polling. Moving away from serverless to running their own infrastructure.... [6]
7. You suck at picking projects - Theo - t3․gg: TL;DR The speaker built a project because they personally needed it, not because of exceptional development skills or product vision. Projects built for personal use have been the speaker's most successful creations, often succeeding immediately. Examples of successful personal projects include Upload Thing, Pick Thing, Quick Pick, work at Twitch, and the YouTube channel itself. Creating things you want to exist and.... [7]
8. Serverless: A Comprehensive Breakdown - Theo - t3․gg: oesn't mean I'm moving all my things off in fact I was a new service that we built for upload thing generally speaking everything I build is still built around serverless paradigms but I haven't taken the time recently to break down why and to really showcase the truth of serverless I've also not been able to do it without a certain sponsor behind me not that they ever had meaningful influence over the things I said.... [8]
9. Defending a disaster (modern frontend development rant) - Theo - t3․gg: or for writing this very excited to read it I am a front-end developer who is Fed Up about front-end development if you write front-end this isn't about you personally okay thank you writing a lot of front end recently I just readed the homepage for upload thing and I'm working on a whole other project it's like 95 plus% client side code so thankful it's about how your choices make me angry okay interesting angle cur... [9]
10. My favorite browser is (kind of) dead - Theo - t3․gg: tead of pretending I can organize all of it instead I sort by recency and when I need to upload something that I just did it's going to be right there generally speaking I think folder systems and file systems are poorly architected and they're like an artifact of a previous way computers work that we just deal with now but downloads being the place where the thing I just did goes and sorting it by recent has been a.... [10]
11. The real reason Tea got hacked (it's NOT vibe coding) - Theo - t3․gg: how bad their security was, but still call it a hack because it was still a hack. It's also worth noting that a lot of services expose URLs publicly. If you have a URL to something, you can probably access it most of the time. This includes, but isn't limited to really big services. Like up until somewhat recently, Google Photos had public URLs for all of the things uploaded. But those URLs are randomly generated an.... [11]
12. Breaking up with Vercel - Theo - t3․gg: the set of sponsors that I recommend that I like the most you'll see those popping up in things like tutorials going forward and I do have one last versell sponsored video that has to come out it's actually a collab video Believe It or Not between versell and one of their competing products fly at iio because in both my head and in theirs they're not really competing I'm really excited for that video we're actually g... [12]

### q003 PASS

- Prompt: Summarize the latest video from each channel I follow.
- Class: `cross_video_synthesis`
- Status: `Completed`
- Score: `3`
- Sources: `12`
- Failure: `-`
- Source videos: Claude Code's latest update is really cool (when it works...) | Claude Cowork: a small taste of AGI | My current stack | OpenAI just dropped their Cursor killer | OpenAI’s TikTok Clone Is Interesting… | This model is kind of a disaster. | Vercel Finally Caught Up | What happened to me?

#### Answer

Retrieved evidence for: Summarize the latest video from each channel I follow.

The answer model is unavailable, so this fallback lists the highest-ranked saved excerpts instead of synthesizing beyond them.

1. This model is kind of a disaster. - Theo - t3․gg: ing the company's reputation. OpenAI models (like 5.4) are contrasted favorably, demonstrating better self-awareness about knowledge cutoffs, web search utilization, and consistency across tasks. Overview This video provides an extensive, critical review of Anthropic's newly released Claude Opus 4.7 model. While acknowledging that Opus 4.7 shows improvements in benchmark scores, instruction following, vision, and mem... [1]
2. What happened to me? - Theo - t3․gg: this can change the same way it changed here. If I start doing more of these types of videos and they perform better than expected, I'll lean more into this. For example, the logging video, we screwed up the export initially and the first version that went up was too short and had most of the content missing. So, we had to re-upload it, which destroys the video performance because a lot of people already saw it, so t... [2]
3. OpenAI just dropped their Cursor killer - Theo - t3․gg: ees for parallel work on the same project, cloud environments, automations (cron-like scheduled prompts), MCP servers/skills integration, and multi-project management. The speaker finds this represents a shift from "commanding code editors via AI" to "orchestrating agents that control code for us," making terminal-based UIs feel obsolete for real coding work. Overview The video provides an in-depth, hands-on review o... [3]
4. Claude Code's latest update is really cool (when it works...) - Theo - t3․gg: ync sub-agent architecture allows the main agent to spin up background tasks that run in parallel without blocking—described as similar to React's Suspense pattern for blocking vs. non-blocking operations. The video documents numerous frustrations: high API costs ($1.56 wasted on a failed task, ~$5+ spent across the session), broken features (the `/rename` command doesn't exist despite being announced), and CLI UX is... [4]
5. Claude Cowork: a small taste of AGI - Theo - t3․gg: ore capable, though potentially riskier. The product represents a step toward AGI by allowing AI to do actual work (moving files, controlling browsers) rather than just generating text responses. Overview This video provides a detailed hands-on review of Anthropic's newly released "Claude Co-work" product, a desktop application designed to bring Claude Code capabilities to non-technical users. The creator, who has ex... [5]
6. Vercel Finally Caught Up - Theo - t3․gg: r" called Bot ID, and an AI gateway. Active CPU billing dramatically narrows the cost gap between Vercel and Cloudflare for long-running, low-CPU requests (like AI inference streaming), bringing the difference from potentially ~100x down to roughly ~2x, while preserving Node compatibility and faster CPUs. Vercel Sandbox allows safe execution of untrusted/AI-generated code via an SDK, competing with Cloudflare's conta... [6]
7. Vercel Finally Caught Up - Theo - t3․gg / Key Points: marginal [7]
8. OpenAI’s TikTok Clone Is Interesting… - Theo - t3․gg / Overview: This video provides an extensive critical analysis of OpenAI's new Sora app—a mobile video generation platform that combines AI video creation with a TikTok-style social feed. The creator, who has early access, spent an entire day testing the platform and hit the 50-video daily rate limit. With deep experience in both AI development and professional video production, the creator offers a multi-faceted critique coveri... [8]
9. What happened to me? - Theo - t3․gg: have gotten 5k plays. A out of 10 would have gotten 40k plays. a 10 out of 10 would have gotten like k plays. That was the range before. The weird thing that's happened is due to the massive change in who is watching my channel and the interest of the people who are watching is the gap between these has gotten massive. Even a six, seven or eight out of 10 topic is going to perform significantly worse. This has been w... [9]
10. My current stack - Theo - t3․gg / Overview: This video provides an extensive, chaotic walkthrough of Theo's current technology stack across multiple projects, including pick thing, T3 chat, marker thing, and unduck. Rather than presenting a simple template to copy, Theo explains the reasoning behind each decision, documents the failures and rewrites he went through, and warns viewers about the complexity costs of various approaches. The core philosophy through... [10]
11. What happened to me? - Theo - t3․gg: result the way I think about things has changed. There are different pieces of how I would rank a video idea. Obviously, there's my excitement level. Like how excited am I about this topic? There is unique insights. This is an important one for me. Like do I have anything unique to add? If somebody else has a video on the topic and said everything I would want to say, I don't need to do the video. I do a video when I... [11]
12. What happened to me? - Theo - t3․gg: it. I will rebrand it. I will try different things. But my excitement on doing videos about CSS went down a ton as a result of that video bombing. And this isn't because the algorithm hates the video. This isn't because people only click AI videos. It is simply a matter of how interest has shifted where an average dev video has just fully dropped off. Like people don't care about average levels of interest in dev top... [12]

### q004 PASS

- Prompt: What are the main themes across the last 10 videos I saved?
- Class: `topic_aggregation`
- Status: `Completed`
- Score: `3`
- Sources: `12`
- Failure: `-`
- Source videos: I built the same app with 5 different stacks | Lynx is incredible (deep dive into Tiktok's React Native killer) | My Biggest Tutorial Ever (Build A FULL Google Drive Clone with React, Next, TypeScript and more) | My hot take on image formats | OpenAI just dropped a Cursor competitor? | So I stopped using Ghostty... | The case against toasts | What happened to me?

#### Answer

Retrieved evidence for: What are the main themes across the last 10 videos I saved?

The answer model is unavailable, so this fallback lists the highest-ranked saved excerpts instead of synthesizing beyond them.

1. Lynx is incredible (deep dive into Tiktok's React Native killer) - Theo - t3․gg: ok open-sourced Lynx, a production-ready cross-platform mobile development framework positioned as a strong React Native competitor with unique architectural advantages. Lynx uses a dual-threaded architecture (main thread + background thread) to keep UI rendering unblocked by JavaScript execution, solving performance issues React Native historically faced. Unlike Flutter, Lynx renders actual native UI primitives rath... [1]
2. What happened to me? - Theo - t3․gg: is this a thing others care about too? So to take this my skateboard taught me how to code idea. That's 10 out of 10 exciting for me. Like obviously I really want to talk about this. Unique insight also 10 out of 10. These are things I haven't seen others communicate. Obviously nobody could talk about my love of my skateboard the way I can. But do people care? No, the result of this is that this video averages across... [2]
3. OpenAI just dropped a Cursor competitor? - Theo - t3․gg: TL;DR OpenAI released a new coding product called Codex (reusing the name) that functions as a subscription-based AI coding assistant available across multiple editors including VS Code, Cursor, and Windsurf. The product includes a CLI tool rebuilt in Rust, IDE extensions, cloud-based background agents that can file PRs, and GitHub code review integration. Testing showed Codex successfully built a complete AI image g... [3]
4. What happened to me? - Theo - t3․gg: this can change the same way it changed here. If I start doing more of these types of videos and they perform better than expected, I'll lean more into this. For example, the logging video, we screwed up the export initially and the first version that went up was too short and had most of the content missing. So, we had to re-upload it, which destroys the video performance because a lot of people already saw it, so t... [4]
5. What happened to me? - Theo - t3․gg: of my community, the people hanging out in Twitch chat right now who have been there since day one, watched this whole thing happen, many of which joined me on the same journey. So, knowing all of this, I want to talk a bit about how I choose a topic for a video because another one of the questions I get all of the time is, "What is your dream video that you would do if the algorithm wouldn't kill it?" A question I g... [5]
6. I built the same app with 5 different stacks - Theo - t3․gg: TL;DR The author built the same "roundest Pokémon" voting app with five different technology stacks they've used throughout their career: Rails, Elixir/Phoenix, Go/GraphQL/React SPA, T3 Stack (Next.js Pages Router), and Next.js App Router with React Server Components. Elixir/Phoenix with LiveView achieved the fastest performance through WebSocket-based diffs and preloading, followed closely by the optimized RSC versi... [6]
7. My hot take on image formats - Theo - t3․gg: TL;DR The video argues WebP is the best current compromise for web images, balancing compression quality, CPU decode cost, and browser/software support. AVIF and JPEG XL, while offering superior compression, suffer from disproportionately high CPU decode requirements, poor software support, and lack of progressive loading features (in AVIF's case). JPEG XL is criticized for being unsupported by major browsers (only S... [7]
8. What happened to me? - Theo - t3․gg: of my employees. But I actually don't care that much. I regularly will share all of the ideas I'm currently planning videos for live in my videos and on stream. most of my audience has seen this list and is begging for me to go film a specific videos they're excited about. If I was trying to hoard my ideas and make sure I had the first video on it, I wouldn't do that. If I could watch someone else's video for any of ... [8]
9. So I stopped using Ghostty... - Theo - t3․gg: a future terminal that combines "paper window manager" concepts (infinitely scrollable/nestable panes like Neri), proper browser integration with existing profiles/extensions, and IDE integration. macOS Spaces are criticized as inadequate for parallel work due to slow animations, poor app switching behavior, and faulty isolation logic. This represents an early glimpse of next-generation dev tools—currently "shitty du... [9]
10. My Biggest Tutorial Ever (Build A FULL Google Drive Clone with React, Next, TypeScript and more) - Theo - t3․gg: nt environment issues like connection timeouts. The database schema initially separates files and folders into distinct tables rather than using a single polymorphic table, with proper relationships including parent folder references and owner IDs. Important schema decisions include using BigInt for IDs (SingleStore's default for scale), adding created_at timestamps (initially forgotten but added later), and indexing... [10]
11. The case against toasts - Theo - t3․gg / Key Points: The core problem with toasts - locality mismatch**: The fundamental issue is that toasts appear far from where the user's attention is focused. When a user clicks a button, they expect feedback near that location, but toasts typically appear in a corner of the screen. The speaker demonstrates this with their own "P thing" application—when clicking to copy an image while scrolled down, the toast appears in a corner an... [11]
12. What happened to me? - Theo - t3․gg / Key Points: Channel Origins and Nerd-Out Motivation**: Theo's YouTube channel began not as a career move, but as an outlet for his deep interest in technical topics and desire to have conversations with like-minded people. He started by participating in Twitter Spaces, where he could nerd out about frameworks, TypeScript, state management, and other tech subjects. The transition to YouTube came from the need to visually demonstr... [12]

### q005 PASS

- Prompt: Which videos cover the same topic from different angles?
- Class: `comparison`
- Status: `Completed`
- Score: `3`
- Sources: `3`
- Failure: `-`
- Source videos: Cloudflare takes on Next.js | How JS ruined the web | What happened to me?

#### Answer

Retrieved evidence for: Which videos cover the same topic from different angles?

The answer model is unavailable, so this fallback lists the highest-ranked saved excerpts instead of synthesizing beyond them.

Comparison frame: both the listed excerpts and their source videos are relevant candidates, while the exact similarities, differences, or counterarguments should be checked against the cited text below.

1. Cloudflare takes on Next.js - Theo - t3․gg: stateful web applications. So think like Twitter, which is a datadriven app. When everyone goes to the Twitter homepage, they get something entirely different. When everybody goes to T3.gg, they get the exact same thing. It's a function of what are people there for? What are they expecting? What are they doing when they're on the site? if they aren't interacting with the site, they are consuming the site, they're co.... [1]
2. What happened to me? - Theo - t3․gg: of my employees. But I actually don't care that much. I regularly will share all of the ideas I'm currently planning videos for live in my videos and on stream. most of my audience has seen this list and is begging for me to go film a specific videos they're excited about. If I was trying to hoard my ideas and make sure I had the first video on it, I wouldn't do that. If I could watch someone else's video for any of ... [2]
3. How JS ruined the web - Theo - t3․gg: y frustrated with the article's premise, systematically dismantles its arguments through technical demonstrations, historical context, and personal anecdotes from years of web development experience. The video covers topics ranging from client-side versus server-side rendering performance, the evolution of web development tools, the purpose of compilers and build steps, and the cultural problems in enterprise softwar... [3]

### q006 PASS

- Prompt: Find every video that mentions RAG.
- Class: `direct_lookup`
- Status: `Completed`
- Score: `3`
- Sources: `12`
- Failure: `-`
- Source videos: Anthropic’s Cybersecurity Shock Wave + Ronan Farrow and Andrew Marantz on Their Sam Altman Investigation + One Good Thing | Breaking up with Vercel | Delete your CLAUDE.md (and your AGENT.md too) | Every smart AI model wants to kill you (yes really) | How did we get here? (A rant about Javascript runtimes) | I gave away $1,000 to prove UUIDs are secure | It’s time to embrace the AI | Okay, I'm a bit scared now... | We need to talk about Ralph | What’s the best programming language for AI? | Why every dev should avoid React | it's time for a change.

#### Answer

Retrieved evidence for: Find every video that mentions RAG.

The answer model is unavailable, so this fallback lists the highest-ranked saved excerpts instead of synthesizing beyond them.

1. How did we get here? (A rant about Javascript runtimes) - Theo - t3․gg / Key Points: GJS, MUJS, JScript, jsdb, njs, TeX, bear, other low.js variants [1]
2. Anthropic’s Cybersecurity Shock Wave + Ronan Farrow and Andrew Marantz on Their Sam Altman Investigation + One Good Thing - Hard Fork: outers. It is sort of like a critical security layer on the internet. And it was designed specifically to be hard to hack. And this model, because of its advanced coding and reasoning capabilities, was able to find this bug that 27 years worth of professional security researchers had not been able to find. What else? Another example was that that it found a bug in a piece of popular open source video software called.... [2]
3. We need to talk about Ralph - Theo - t3․gg / Overview: The video provides a deep technical exploration of "Ralph loops," a technique introduced by Jeff Huntley for running AI coding agents in continuous loops that persist state externally rather than through conversation history. The presenter explains the concept's origins, why various implementations differ in effectiveness, and how the core principles relate to broader context engineering practices. The video covers i... [3]
4. What’s the best programming language for AI? - Theo - t3․gg: find a good solution, but not the right solution. There are so many different options that it's easy to get lost in the sauce trying to pick the right one. And if you pick one in one place and a different one somewhere else, things get much harder to maintain over time. And more importantly, how hard is it to find a bad solution? In Typescript, it is trivial. It is so easy to find bad solutions. You can just press ta... [4]
5. Every smart AI model wants to kill you (yes really) - Theo - t3․gg / Overview: This video responds to an article by Ted that argues smart AI models have an inherent tendency toward evil because human moral frameworks—legal, religious, evolutionary, and social constraints—don't apply to machines. The creator, who builds benchmarks to test AI behavior, works through the article's claims while presenting original research and recent industry findings on AI misalignment. The discussion covers the p... [5]
6. It’s time to embrace the AI - Theo - t3․gg: things implemented in your codebase. It's calling a tool that is real code that is used to access files in the codebase. So when it wants to know what files use a function, it's calling a tool that uses TypeScript's IntelliSense to find where the references are. Or it might just be calling a GP call across your codebase for all the things that match that shape and find all the files that are relevant. But it's using.... [6]
7. Okay, I'm a bit scared now... - Theo - t3․gg: nds of showing anything related to the 01 Mini model in this and then a lot of just talking it does fit the AI way which is using way too many words for the thing you're trying to do good old delve yeah I love that I love that Paul Graham keeps getting proven Ming more and more right anyways oh they have an actual coding demo at the bottom if only I knew about that earlier one last I want to show an example of a codi... [7]
8. it's time for a change. - Theo - t3․gg: bout what is working in my life and what I need to be working on with my life and as much as I love doing all of this I love building more and the success of T3 chat has been incredible so me reflect deeper on that excitement and energy that I'm feeling half the time I'm live I just wish I was on my laptop writing code in a corner somewhere trying to find more ways to bridge the gap between these things and also I ca... [8]
9. Delete your CLAUDE.md (and your AGENT.md too) - Theo - t3․gg: app/ts or tsx to find all of the files there. Did the same for convex. Did the same for general source. Found the convex schema. It found the app routes. Found the vcon config ts config. It just read all of these things. And then it after reading all of that concluded has a good understanding of the codebase and it wrote this. But remember what it wrote is based on things that it already was able to find. In fact, it... [9]
10. I gave away $1,000 to prove UUIDs are secure - Theo - t3․gg: I've ever done because it was about his video, which one of my favorite videos I've ever watched. Nolan is one of the most creative developers I've ever seen, making truly novel, exciting things on the web. And he made the every Uyu ID site, which was a crazy hack, just an unreal, genuinely novel, insane hack in order to allow you to see every UU ID on one page. He was excited about this, so he decided to go add a fe... [10]
11. Breaking up with Vercel - Theo - t3․gg: believe it or not this one is in clickbait Rell and I are breaking up they are no longer a channel sponsor it's been a wild two years since I started posting videos believe it or not I did only really start posting in April of 2022 and everything that's happened since then has been unbelievable with that we've had a lot of changes I went from running the channel solo to having a team of four helping me out with it I'... [11]
12. Why every dev should avoid React - Theo - t3․gg: was 15. And then Justin Timberlake put out some incredible music and I had to get over my [ __ ] The author of this article is making the same mistake I made when I was 15 because there were some indie things that I thought were obviously really good and there were some popular things that were obviously not good. All popular, bad, all indie good. Easy trap to fall into if you're 15 years old. I don't know how the au... [12]

### q007 PASS

- Prompt: Find every video that mentions Ollama.
- Class: `direct_lookup`
- Status: `Completed`
- Score: `3`
- Sources: `12`
- Failure: `-`
- Source videos: Anthropic’s Cybersecurity Shock Wave + Ronan Farrow and Andrew Marantz on Their Sam Altman Investigation + One Good Thing | Breaking up with Vercel | Delete your CLAUDE.md (and your AGENT.md too) | Every smart AI model wants to kill you (yes really) | I gave away $1,000 to prove UUIDs are secure | It’s time to embrace the AI | Okay, I'm a bit scared now... | OpenAI’s TikTok Clone Is Interesting… | OpenAI’s open source models are finally here | What’s the best programming language for AI? | it's time for a change.

#### Answer

Retrieved evidence for: Find every video that mentions Ollama.

The answer model is unavailable, so this fallback lists the highest-ranked saved excerpts instead of synthesizing beyond them.

1. OpenAI’s open source models are finally here - Theo - t3․gg: coming up very soon. This isn't something I'd normally want to ask a traditional model about just because I don't really want this data out there, I say as I'm literally broadcasting it to hundreds of thousands of people in this video. You get the point, though. I thought this would be a fun like test of things here. When I ask the 20 bill param model this question, the first thing I have to deal with is the awful fo... [1]
2. OpenAI’s TikTok Clone Is Interesting… - Theo - t3․gg / Overview: This video provides an extensive critical analysis of OpenAI's new Sora app—a mobile video generation platform that combines AI video creation with a TikTok-style social feed. The creator, who has early access, spent an entire day testing the platform and hit the 50-video daily rate limit. With deep experience in both AI development and professional video production, the creator offers a multi-faceted critique coveri... [2]
3. Anthropic’s Cybersecurity Shock Wave + Ronan Farrow and Andrew Marantz on Their Sam Altman Investigation + One Good Thing - Hard Fork: outers. It is sort of like a critical security layer on the internet. And it was designed specifically to be hard to hack. And this model, because of its advanced coding and reasoning capabilities, was able to find this bug that 27 years worth of professional security researchers had not been able to find. What else? Another example was that that it found a bug in a piece of popular open source video software called.... [3]
4. What’s the best programming language for AI? - Theo - t3․gg: find a good solution, but not the right solution. There are so many different options that it's easy to get lost in the sauce trying to pick the right one. And if you pick one in one place and a different one somewhere else, things get much harder to maintain over time. And more importantly, how hard is it to find a bad solution? In Typescript, it is trivial. It is so easy to find bad solutions. You can just press ta... [4]
5. OpenAI’s open source models are finally here - Theo - t3․gg: bill model, my entire computer is going to lock up here. I'll turn on activity monitor so you can see that as it's running. You'll see very quickly it fills up my memory like almost immediately. Olam is now using over 30 gigs of RAM. I switch to CPU allocation. Not too too high because it's not using the CPU. It's using the GPU. I don't think any of these options on Mac OS are going to give me the detail I want. I th... [5]
6. Every smart AI model wants to kill you (yes really) - Theo - t3․gg / Overview: This video responds to an article by Ted that argues smart AI models have an inherent tendency toward evil because human moral frameworks—legal, religious, evolutionary, and social constraints—don't apply to machines. The creator, who builds benchmarks to test AI behavior, works through the article's claims while presenting original research and recent industry findings on AI misalignment. The discussion covers the p... [6]
7. It’s time to embrace the AI - Theo - t3․gg: things implemented in your codebase. It's calling a tool that is real code that is used to access files in the codebase. So when it wants to know what files use a function, it's calling a tool that uses TypeScript's IntelliSense to find where the references are. Or it might just be calling a GP call across your codebase for all the things that match that shape and find all the files that are relevant. But it's using.... [7]
8. Okay, I'm a bit scared now... - Theo - t3․gg: nds of showing anything related to the 01 Mini model in this and then a lot of just talking it does fit the AI way which is using way too many words for the thing you're trying to do good old delve yeah I love that I love that Paul Graham keeps getting proven Ming more and more right anyways oh they have an actual coding demo at the bottom if only I knew about that earlier one last I want to show an example of a codi... [8]
9. it's time for a change. - Theo - t3․gg: bout what is working in my life and what I need to be working on with my life and as much as I love doing all of this I love building more and the success of T3 chat has been incredible so me reflect deeper on that excitement and energy that I'm feeling half the time I'm live I just wish I was on my laptop writing code in a corner somewhere trying to find more ways to bridge the gap between these things and also I ca... [9]
10. Delete your CLAUDE.md (and your AGENT.md too) - Theo - t3․gg: app/ts or tsx to find all of the files there. Did the same for convex. Did the same for general source. Found the convex schema. It found the app routes. Found the vcon config ts config. It just read all of these things. And then it after reading all of that concluded has a good understanding of the codebase and it wrote this. But remember what it wrote is based on things that it already was able to find. In fact, it... [10]
11. I gave away $1,000 to prove UUIDs are secure - Theo - t3․gg: I've ever done because it was about his video, which one of my favorite videos I've ever watched. Nolan is one of the most creative developers I've ever seen, making truly novel, exciting things on the web. And he made the every Uyu ID site, which was a crazy hack, just an unreal, genuinely novel, insane hack in order to allow you to see every UU ID on one page. He was excited about this, so he decided to go add a fe... [11]
12. Breaking up with Vercel - Theo - t3․gg: believe it or not this one is in clickbait Rell and I are breaking up they are no longer a channel sponsor it's been a wild two years since I started posting videos believe it or not I did only really start posting in April of 2022 and everything that's happened since then has been unbelievable with that we've had a lot of changes I went from running the channel solo to having a team of four helping me out with it I'... [12]

### q008 PASS

- Prompt: Find every video that mentions semantic search.
- Class: `direct_lookup`
- Status: `Completed`
- Score: `3`
- Sources: `12`
- Failure: `-`
- Source videos: Anthropic’s Cybersecurity Shock Wave + Ronan Farrow and Andrew Marantz on Their Sam Altman Investigation + One Good Thing | Every smart AI model wants to kill you (yes really) | Is this the end of Chrome? | It’s time to embrace the AI | Okay, I'm a bit scared now... | What’s the best programming language for AI? | i made my own search engine (kind of) | it's time for a change. | “Just Use HTML”

#### Answer

Retrieved evidence for: Find every video that mentions semantic search.

The answer model is unavailable, so this fallback lists the highest-ranked saved excerpts instead of synthesizing beyond them.

1. i made my own search engine (kind of) - Theo - t3․gg: because it's searching on their server it's not doing a literal search but the search is going to their server being parsed and then the URL is transformed to a different search engine if you used a bang and there's no reason that that should be on the server I just want the search to happen immediately and if I do corgis exclamation point GI I'm pressing enter now it's already searched no speed up nothing there vers... [1]
2. Every smart AI model wants to kill you (yes really) - Theo - t3․gg / Overview: This video responds to an article by Ted that argues smart AI models have an inherent tendency toward evil because human moral frameworks—legal, religious, evolutionary, and social constraints—don't apply to machines. The creator, who builds benchmarks to test AI behavior, works through the article's claims while presenting original research and recent industry findings on AI misalignment. The discussion covers the p... [2]
3. Anthropic’s Cybersecurity Shock Wave + Ronan Farrow and Andrew Marantz on Their Sam Altman Investigation + One Good Thing - Hard Fork: outers. It is sort of like a critical security layer on the internet. And it was designed specifically to be hard to hack. And this model, because of its advanced coding and reasoning capabilities, was able to find this bug that 27 years worth of professional security researchers had not been able to find. What else? Another example was that that it found a bug in a piece of popular open source video software called.... [3]
4. “Just Use HTML” - Theo - t3․gg: [ __ ] websites, but I know [ __ ] better than to pat out this video any [ __ ] more. So, I'm just going to be [ __ ] done. Let me know what you think. Until next time, [ __ ] [4]
5. What’s the best programming language for AI? - Theo - t3․gg: find a good solution, but not the right solution. There are so many different options that it's easy to get lost in the sauce trying to pick the right one. And if you pick one in one place and a different one somewhere else, things get much harder to maintain over time. And more importantly, how hard is it to find a bad solution? In Typescript, it is trivial. It is so easy to find bad solutions. You can just press ta... [5]
6. i made my own search engine (kind of) - Theo - t3․gg: ducko gets their [ __ ] together I might just move back there anyways but for now I built the best search engine in the world for me probably won't be the best for every one but if it is for you awesome and if it isn't you now have all the things you need to go make your own that's all I got on this one let me know what you think it's my search engine a meme or is it actually useful I think the future of more persona... [6]
7. It’s time to embrace the AI - Theo - t3․gg: things implemented in your codebase. It's calling a tool that is real code that is used to access files in the codebase. So when it wants to know what files use a function, it's calling a tool that uses TypeScript's IntelliSense to find where the references are. Or it might just be calling a GP call across your codebase for all the things that match that shape and find all the files that are relevant. But it's using.... [7]
8. i made my own search engine (kind of) - Theo - t3․gg: by my love of bangs I really like the pattern the fact that there are ,500 of them I use like 20 of them the fact that there are so many of them is actually really cool what's even cooler is if you take this URL bangs change it to bang. JS here they all are is a Mal formatted JS file it's called JS it's some weird hybrid of JS and J on where it is just an array that's never assigned a value So undu currently solves t... [8]
9. Is this the end of Chrome? - Theo - t3․gg / Key Points: Anthropic. The creator notes keyword targeting is valuable—Anthropic appears to do keyword targeting on Google, with Claude ads appearing on AI-related searches. [9]
10. Okay, I'm a bit scared now... - Theo - t3․gg: nds of showing anything related to the 01 Mini model in this and then a lot of just talking it does fit the AI way which is using way too many words for the thing you're trying to do good old delve yeah I love that I love that Paul Graham keeps getting proven Ming more and more right anyways oh they have an actual coding demo at the bottom if only I knew about that earlier one last I want to show an example of a codi... [10]
11. it's time for a change. - Theo - t3․gg: bout what is working in my life and what I need to be working on with my life and as much as I love doing all of this I love building more and the success of T3 chat has been incredible so me reflect deeper on that excitement and energy that I'm feeling half the time I'm live I just wish I was on my laptop writing code in a corner somewhere trying to find more ways to bridge the gap between these things and also I ca... [11]
12. i made my own search engine (kind of) - Theo - t3․gg: thing it will open straight to GitHub to that repo which is really nice yeah could see myself adding custom bangs probably through local storage haven't done it yet something I actually did really want to do is log all your searches locally in indexdb so that you can look at them and have like a page showing it all yeah there's a lot of places this can go I'm planning on taking it none of them I'm expecting just know... [12]

### q009 PASS

- Prompt: Find every video that mentions YouTube API.
- Class: `direct_lookup`
- Status: `Completed`
- Score: `3`
- Sources: `12`
- Failure: `-`
- Source videos: Anthropic’s Cybersecurity Shock Wave + Ronan Farrow and Andrew Marantz on Their Sam Altman Investigation + One Good Thing | Breaking up with Vercel | Delete your CLAUDE.md (and your AGENT.md too) | Every smart AI model wants to kill you (yes really) | Google Drive hates developers now | I can't believe nobody's done this before... | I gave away $1,000 to prove UUIDs are secure | It’s time to embrace the AI | Okay, I'm a bit scared now... | What’s the best programming language for AI? | Why every dev should avoid React | it's time for a change.

#### Answer

Retrieved evidence for: Find every video that mentions YouTube API.

The answer model is unavailable, so this fallback lists the highest-ranked saved excerpts instead of synthesizing beyond them.

1. Anthropic’s Cybersecurity Shock Wave + Ronan Farrow and Andrew Marantz on Their Sam Altman Investigation + One Good Thing - Hard Fork: outers. It is sort of like a critical security layer on the internet. And it was designed specifically to be hard to hack. And this model, because of its advanced coding and reasoning capabilities, was able to find this bug that 27 years worth of professional security researchers had not been able to find. What else? Another example was that that it found a bug in a piece of popular open source video software called.... [1]
2. I can't believe nobody's done this before... - Theo - t3․gg / Key Points: "stapled on" to existing APIs. [2]
3. What’s the best programming language for AI? - Theo - t3․gg: find a good solution, but not the right solution. There are so many different options that it's easy to get lost in the sauce trying to pick the right one. And if you pick one in one place and a different one somewhere else, things get much harder to maintain over time. And more importantly, how hard is it to find a bad solution? In Typescript, it is trivial. It is so easy to find bad solutions. You can just press ta... [3]
4. Google Drive hates developers now - Theo - t3․gg / Overview: The video examines how Google's recent API policy changes for Google Drive are devastating third-party developers who rely on Drive integration for their apps. Through detailed case studies of iA Writer (a writing app) and Panic (maker of Transmit and Nova), the video documents the bureaucratic obstacles, shifting requirements, and expensive annual security audits that Google now mandates. The speaker argues these po... [4]
5. Every smart AI model wants to kill you (yes really) - Theo - t3․gg / Overview: This video responds to an article by Ted that argues smart AI models have an inherent tendency toward evil because human moral frameworks—legal, religious, evolutionary, and social constraints—don't apply to machines. The creator, who builds benchmarks to test AI behavior, works through the article's claims while presenting original research and recent industry findings on AI misalignment. The discussion covers the p... [5]
6. It’s time to embrace the AI - Theo - t3․gg: things implemented in your codebase. It's calling a tool that is real code that is used to access files in the codebase. So when it wants to know what files use a function, it's calling a tool that uses TypeScript's IntelliSense to find where the references are. Or it might just be calling a GP call across your codebase for all the things that match that shape and find all the files that are relevant. But it's using.... [6]
7. Okay, I'm a bit scared now... - Theo - t3․gg: nds of showing anything related to the 01 Mini model in this and then a lot of just talking it does fit the AI way which is using way too many words for the thing you're trying to do good old delve yeah I love that I love that Paul Graham keeps getting proven Ming more and more right anyways oh they have an actual coding demo at the bottom if only I knew about that earlier one last I want to show an example of a codi... [7]
8. it's time for a change. - Theo - t3․gg: bout what is working in my life and what I need to be working on with my life and as much as I love doing all of this I love building more and the success of T3 chat has been incredible so me reflect deeper on that excitement and energy that I'm feeling half the time I'm live I just wish I was on my laptop writing code in a corner somewhere trying to find more ways to bridge the gap between these things and also I ca... [8]
9. Delete your CLAUDE.md (and your AGENT.md too) - Theo - t3․gg: app/ts or tsx to find all of the files there. Did the same for convex. Did the same for general source. Found the convex schema. It found the app routes. Found the vcon config ts config. It just read all of these things. And then it after reading all of that concluded has a good understanding of the codebase and it wrote this. But remember what it wrote is based on things that it already was able to find. In fact, it... [9]
10. I gave away $1,000 to prove UUIDs are secure - Theo - t3․gg: I've ever done because it was about his video, which one of my favorite videos I've ever watched. Nolan is one of the most creative developers I've ever seen, making truly novel, exciting things on the web. And he made the every Uyu ID site, which was a crazy hack, just an unreal, genuinely novel, insane hack in order to allow you to see every UU ID on one page. He was excited about this, so he decided to go add a fe... [10]
11. Breaking up with Vercel - Theo - t3․gg: believe it or not this one is in clickbait Rell and I are breaking up they are no longer a channel sponsor it's been a wild two years since I started posting videos believe it or not I did only really start posting in April of 2022 and everything that's happened since then has been unbelievable with that we've had a lot of changes I went from running the channel solo to having a team of four helping me out with it I'... [11]
12. Why every dev should avoid React - Theo - t3․gg: was 15. And then Justin Timberlake put out some incredible music and I had to get over my [ __ ] The author of this article is making the same mistake I made when I was 15 because there were some indie things that I thought were obviously really good and there were some popular things that were obviously not good. All popular, bad, all indie good. Easy trap to fall into if you're 15 years old. I don't know how the au... [12]

### q010 PASS

- Prompt: What are the most common tools or frameworks discussed in my library?
- Class: `topic_aggregation`
- Status: `Completed`
- Score: `3`
- Sources: `12`
- Failure: `-`
- Source videos: "AI Startups" are over done (finally) | Anthropic admits that MCP sucks | Anthropic is trying SO hard to fix MCP... | I can't believe he was right. | I don’t really use libraries anymore | I had no idea it was this bad... | Is Tailwind really the right default? | You're falling behind. It's time to catch up. | You're using AI coding tools wrong

#### Answer

Retrieved evidence for: What are the most common tools or frameworks discussed in my library?

The answer model is unavailable, so this fallback lists the highest-ranked saved excerpts instead of synthesizing beyond them.

1. I don’t really use libraries anymore - Theo - t3․gg / Key Points: for understanding different library types: **Libraries beyond your knowledge**: These are used by people who don't know how to solve the problem themselves. Examples include `is-odd` (literally one line of code) and `leftpad`. The argument against these is that users are outsourcing competency and taking on supply chain risks without understanding them. **Libraries for tedious reimplementation**: Even capable develop... [1]
2. You're falling behind. It's time to catch up. - Theo - t3․gg: f that does happen. By the way, if you watch this much, you should hit the sub button if you want to keep up. It's a pretty good way to help and uh helps the channel out a ton too. Next piece, give your agents tools to all dev tooling. Things like linear, GitHub, Data Dog, Sentry, any internal tooling. If agents are being held back because of the lack of context, that's your fault. Very bold. Still coming around to t... [2]
3. I don’t really use libraries anymore - Theo - t3․gg / TL;DR: AI tools are fundamentally changing the calculus of when to use external libraries versus implementing solutions yourself, making it easier to "vibe code" alternatives. The speaker is actively removing libraries from projects when they cause problems, finding it often easier to rewrite functionality than fight with problematic dependencies. Libraries fall into categories: those beyond your knowledge (beginner-level p... [3]
4. I had no idea it was this bad... - Theo - t3․gg / Takeaways: HTML whitespace handling is inconsistent and context-dependent, relying on CSS properties that tools and authors often cannot know ahead of time. The `innerText` vs `textContent` API difference reflects a deeper issue: CSS styling can change the actual text content a user sees, breaking clean separation between structure and presentation. Prettier's `--html-whitespace-sensitivity` settings are compromises because the... [4]
5. I don’t really use libraries anymore - Theo - t3․gg: you look at the things that are wrong with stuff like these packages, the classic supply chain attack stuff, the fact that it's not in your codebase, so anything that goes wrong with it or doesn't perform exactly as you expect it to is now a problem that you have to deal with. And of course, you don't actually understand it. These are all real problems and we are able to eliminate multiple of these simply by generati... [5]
6. I can't believe he was right. - Theo - t3․gg / Key Points: rrors the natural progression engineers make toward management and tech lead roles—marking growth, even if it feels less satisfying day-to-day. **Personal example**: Mark (CTO for T3) had to shift from writing most code to orchestration and code review as the team grew, experiencing the common feeling of reduced productivity when not working in an editor. Productivity Gains and Changing Workflows **12,000 lines of co... [6]
7. I don’t really use libraries anymore - Theo - t3․gg: TL;DR AI tools are fundamentally changing the calculus of when to use external libraries versus implementing solutions yourself, making it easier to "vibe code" alternatives. The speaker is actively removing libraries from projects when they cause problems, finding it often easier to rewrite functionality than fight with problematic dependencies. Libraries fall into categories: those beyond your knowledge (beginner-l... [7]
8. Is Tailwind really the right default? - Theo - t3․gg / Takeaways: Tailwind has earned its default status by being "good enough" across multiple dimensions (performance, composition, naming simplicity, bug prevention) that most developers stop looking for alternatives. If you need fine-grained control at massive scale (Meta-level), alternatives like Stylex may be worth investigating—but Tailwind is still a reasonable starting point, and tools are being built to migrate from Tailwind... [8]
9. You're using AI coding tools wrong - Theo - t3․gg: to here and then engineers are assigned to work on it Jira tickets are cut months to years in advance and then deadline is set much later deadline isn't going to be hit and then they ask me. They got all the way down here and then ask me to save it just for me to come in and say this is a bad idea and get in a bunch of arguments with it and then have them just go and ship it anyways and then the product sucks and the... [9]
10. "AI Startups" are over done (finally) - Theo - t3․gg: matched what we wanted and what we were doing. It made the boring parts easier to do. It didn't get in the way of the fun parts. But most importantly, and I hope this isn't controversial, Copilot was built by developers for developers. Duh, right? Like obviously devs built the thing. Devs build things and obviously devs use it. It's a dev tool. There's an important thing here. And yes, it's obvious. The arrows should... [10]
11. Anthropic admits that MCP sucks - Theo - t3․gg: tegrations. Since launching MCP in November of 2024, adoption has been rapid by people trying to sell you things, not people trying to make useful things. The community has built thousands of MCP servers. SDKs are available for all major programming languages and the industry has adopted MCP as the de facto standard for connecting agents to tools and data and also implemented a dozen standards for how to make the dat... [11]
12. Anthropic is trying SO hard to fix MCP... - Theo - t3․gg: with a handful of tools. Here you have the GitHub server which has 35 tools, the Slack server with 11, Sentry Server with five, Graphfana with five, and Splunk with two. These 58 tools consume 55,000 tokens before the conversation even starts. Add more servers like Jira, which alone uses 17,000 tokens, and you're quickly approaching 100K token overhead. And they've seen tool definitions consume up to 134K tokens. Thi... [12]

### q011 PASS

- Prompt: Which channels talk about the same subjects most often?
- Class: `comparison`
- Status: `Completed`
- Score: `3`
- Sources: `12`
- Failure: `-`
- Source videos: AI chat apps are driving me insane | Going Back To Next | How I code with AI right now | It’s actually over now | PewDiePie is right about AI | Vibe coding is already dead | What happened to me? | What is Theo's Worst Take? | Why Github Actually Won | Why I moved away from SQL

#### Answer

Retrieved evidence for: Which channels talk about the same subjects most often?

The answer model is unavailable, so this fallback lists the highest-ranked saved excerpts instead of synthesizing beyond them.

1. Why I moved away from SQL - Theo - t3․gg: f how convex works. And there are so many of these little things that you realize when you buy into their different set of primitives that make life so much better. So how does this live update? I haven't even talked about that. Well, let's look at messages. Let's look at send specifically. Takes in a channel ID so we know what channel it's going to and content so we know what we want to post. User ID, await, get off... [1]
2. AI chat apps are driving me insane - Theo - t3․gg: of politeness crazy to which they immediately respond yeah I don't watch a lot of company YouTube channels but that's cuz they're they're bad I'm going to make a good one and I'm like okay that would make you the first company ever to do it that'd be incredibly Innovative aren't you currently trying to reinvent how we think about databases with AI aren't you currently trying to rebuild the IDE that we use every day a... [2]
3. Why Github Actually Won - Theo - t3․gg: is more senior development scene in the content world that we would accidentally kill a lot of existing Devril work because at the time the only options for senior technical content were employees of companies talking about the stuff they were working on and their goal was to do their job and to some extent sell a thing. That's not our goal. Our goal is to nerd out about this thing we're obsessed with and make this t... [3]
4. Going Back To Next - Theo - t3․gg: having this type of strict language where everything is typed and you have like these built-in constructs for doing channels it just has a better concurrency model than node.js in my opinion like in a in another video I just did the better parallel the concurrency in JavaScript is pretty unmatched but if you want to actually run multiple things at the same time go is really good for that the other day I kind of talke... [4]
5. Vibe coding is already dead - Theo - t3․gg: come a huge part of my life as a content creator. People seem to think that if you have a video that performs surprisingly well as a YouTuber, Instagram, whatever you're on, that what you should do next is the same topic again. It makes a lot of sense. If I mostly talk about I don't know React and I do one video about spelt instead and that spelt video does really well. Obviously I should talk about spelt for and yes... [5]
6. What happened to me? - Theo - t3․gg: on YouTube. And it did way better than I ever would have imagined. So, a couple things to learn from this. One is that I just want to nerd out. That's the point of all of this is I have these things I want to talk about. And if you ask anybody who knows me in person, I'm exactly the same as I am on the channel. I just want to geek out about the things I'm excited by. I like having deep conversations about the things.... [6]
7. What happened to me? - Theo - t3․gg: random project I was excited about. I was super curious about this vibe canban thing. So, I started using it. Yes, my UI changes went through. The whole UI for this sidebar was rewritten by me because I was so annoyed by a handful of weird UI quirks that existed. I rewrote the layout system. I rewrote the sidebar. I gave them a ton of feedback. I filed a ton of PRs on this random project I had just discovered between... [7]
8. PewDiePie is right about AI - Theo - t3․gg: e know. I'm more than happy to intro you, man. Good [ __ ] There's also AI Explained, who is [ __ ] awesome. I love all of his stuff. I've learned so much from it. There are this small pocket of really good AI channels now that just didn't exist before. And I assumed I was the one who was wrong until I went a little deeper and realized, no, it is indeed the shitty crypto influencers who switched to AI because they wa... [8]
9. What happened to me? - Theo - t3․gg: me the same way they see other channels. And if your understanding of how these things work comes from those other channels, it only makes sense. Somebody who's posting videos every day with 500K subs should probably have their scripts written by someone else so they can have a life still. I don't have a life. This is all I do. I write code. I run my business. I talk [ __ ] on YouTube. So, what the [ __ ] does any of... [9]
10. How I code with AI right now - Theo - t3․gg: ns and also probably some examples of bad patterns that it can use in reference when it is developing and iterating. Having those examples is really really helpful. And if you write a lot of projects using the same few packages, it's not the worst idea to yoink some of that code over to point at and be like, "Hey, here's how I like to do things." If you can paste one or two examples in your prompt, good chance it'll.... [10]
11. It’s actually over now - Theo - t3․gg: sales and showing off to computer nerds. You don't start with the fancy marketing video. You start by being real humans. And they tried a little too hard to do the marketing thing. And what's really funny is I talked to a lot of these earlier stage companies and they want to do their own elaborate YouTube stuff. Both because they see me as a YouTuber. They're like, "Hey, how can we use YouTube to grow our business?".... [11]
12. What is Theo's Worst Take? - Theo - t3․gg / Overview: This brief exchange involves a discussion about identifying the speaker's worst take or opinion. The conversation touches on the speaker's self-assessment of their takes, a past controversial statement, and a specific critique of a storybook item. The dialogue ends with one speaker conceding a point about the storybook's utility. [12]

### q012 PASS

- Prompt: What is the best single video to understand this topic?
- Class: `recommendation`
- Status: `Completed`
- Score: `3`
- Sources: `7`
- Failure: `-`
- Source videos: Deepseek R1 Is Really, Really Good | Gemini Flash 3 is my new favorite model (yes really) | How Minecraft AI ACTUALLY works | Is Electron really that bad? | React feels insane | Watch this if you know HTML | Why Microsoft deleted this extension from MILLIONS of computers

#### Answer

Retrieved evidence for: What is the best single video to understand this topic?

The answer model is unavailable, so this fallback lists the highest-ranked saved excerpts instead of synthesizing beyond them.

1. Deepseek R1 Is Really, Really Good - Theo - t3․gg: pixels instead of specifying this pixel is this gray this one right next to it's a slightly different gray gradients are really hard to compress because there's a lot of different colors in the range this means anything that changes quickly or has a range of numbers in a small area especially things like confetti suck to compress and seeing this young lean video at the very least made me feel better about the quality... [1]
2. Watch this if you know HTML - Theo - t3․gg / Overview: This video provides an in-depth technical analysis of the evolution of web application rendering strategies, moving from traditional Multi-Page Apps (MPAs) and Single Page Apps (SPAs) to modern hybrid models. The speaker diagrams the data flow and trade-offs of each approach, highlighting the specific problems each model solves and the new complexities it introduces. Key themes include the tension between server-side... [2]
3. Why Microsoft deleted this extension from MILLIONS of computers - Theo - t3․gg: things I would have been more than willing to forgive Matia if he had just apologized and stopped during this spiral but as he has continued to be wrong he has continued to deny and get worse and worse doing worse and worse things never once taking the time to admit that he was wrong or apologize to the harm he has caused to hundreds of developers in the open source ecosystem into the millions of users of a theme tha... [3]
4. React feels insane - Theo - t3․gg / Key Points: understanding a tool doesn't mean it's bad. [4]
5. Is Electron really that bad? - Theo - t3․gg: quality of experience is trash we are perfectly aligned I couldn't agree more but the moment you say wow elect sucks at the end of it you've just lost the plot you're not talking about a thing you understand if you think that's the case if you actually think electron is the reason that Discord on desktop sucks you don't understand electron Discord business incentives or basic software development straight up and that... [5]
6. Gemini Flash 3 is my new favorite model (yes really) - Theo - t3․gg: with Gemini 3 Pro, managing to beat out Flash and 2.5 Pro for previous days. And also managing to beat out Sonet 4.5, which is pretty impressive, too. MMU, it pulled the best score to date. Pretty nuts. Screen understanding is pretty good. It crushes the scores from 2.5, Flash, and Pro, which were in the like single digit to just barely double digit percentages, and now it's pulling a 70. All cool stuff. Video unders... [6]
7. How Minecraft AI ACTUALLY works - Theo - t3․gg: in a text editor notice how few pixels are changing on my screen right now basically none of the pixels on my screen are changing at the moment pretty much zero of them which means it's very easy to encode my video at the moment but if I was to switch here and move my arms around really fast suddenly my CPU is going to spike I just watched it go from 4% CPU utilization to seven just from that like that's the nature..... [7]

### q013 PASS

- Prompt: Give me a quick summary of this video in three bullets.
- Class: `direct_lookup`
- Status: `Completed`
- Score: `3`
- Sources: `12`
- Failure: `-`
- Source videos: AI images just got dangerously good (RIP diffusion??) | Fixing serverless node.js (by adding servers?) | I Fixed Stripe | I ranked every AI based on vibes | My current stack | Open source is dying | The Tailwind drama | This new Tailwind feature is scarier than I thought

#### Answer

Retrieved evidence for: +{Open source is dead now?} Give me a quick summary of this video in three bullets.

The answer model is unavailable, so this fallback lists the highest-ranked saved excerpts instead of synthesizing beyond them.

1. Open source is dying - Theo - t3․gg: escalate because you almost certainly have to use AI to actually scan these PRs. It has a lot of config, enough config that I'm almost certain that this project was vibe coded, but yeah, you get the idea. There are a lot of solutions being made to try and fix these problems. Some of them are going to make it harder for new maintainers to break out, which sucks because we might just have our current maintainers until ... [1]
2. Open source is dying - Theo - t3․gg / Overview: The video presents a comprehensive examination of how AI is negatively impacting the open source ecosystem. The speaker, an experienced open source maintainer and creator of T3 Code, shares firsthand experiences managing a new project that received 150 PRs in just 5 days. The discussion covers four main problem areas: PR spam and quality degradation, increasingly hostile and confused users, GitHub's inadequate platfo... [2]
3. I Fixed Stripe - Theo - t3․gg: ss tedious things like you know code review that's why I love today's sponsor code rabbit they make code review so much simpler by using AI to leave useful comments ahead of time it's like having somebody do a quick pass on a PR before the rest of the team comes in to review it we've been using it for for every upload thing PR for a while now just as a recent example from a PR that's literally still open here's one w... [3]
4. This new Tailwind feature is scarier than I thought - Theo - t3․gg: super handy especially on big PRS like the one I'm about to show to have a quick summary for every file on what changed and why it's just a really quick way to get through and they even tag in related PRS so if you have anything going on around the same time super super handy it's been great to work with it also leaves comments on individual lines that it thinks can be changed so here it said that it thinks this type... [4]
5. The Tailwind drama - Theo - t3․gg: he had. The link for this is in the description if you want to hear the whole thing. About 33 minutes long. The quick summary I'll give you is that they saw revenue going down, but they did the thing all founders do, which is they kind of ignore the numbers when they aren't good until they went back and looked and realized, "Oh we have 6 months until we go out of business." and he decided to do the right thing here,.... [5]
6. Open source is dying - Theo - t3․gg: put so much effort into killing Hacktoberfest. I think this video of mine, don't contribute to open source, is one of the best videos I ever filmed. Not sure who that blonde guy with the mustache is though. Seriously though, like that video, I have been told by so many maintainers how thankful they are for this video more than almost anything I've done as a developer and journalist, YouTuber, whatever you want to cal... [6]
7. The Tailwind drama - Theo - t3․gg: a to your service without you having to do anything at all. It's miraculous. And setting this up yourself is hellish. Trust me on that one, guys. By the way, first million users are free. So what do you have to lose? Check them out now at soyv.v.link/workos. This all came to light earlier today when Adam posted this recording of his morning walk, just talking to his phone, recording it, and publishing it. And there a... [7]
8. Open source is dying - Theo - t3․gg / Key Points: used incorrectly and require more mental energy to parse. **Declining Question Quality**: The quality of questions in communities like Create T3 App channels has "gone down meaningfully" and "hilariously" in areas like TypeScript and tRPC. **Hallucinated Technical Understanding**: A Reddit commenter claimed "React is dead and because of that, there is Next.js and there's absolutely no need for React"—a fundamental mi... [8]
9. AI images just got dangerously good (RIP diffusion??) - Theo - t3․gg: id a great job. It even has like the right JS logo on the shirt with the font and everything proper. It does text way better than things I've tried in the past. It's possible that the refresh there just killed this in progress. I would give them crap for it, but our UI handles refreshes mid generation really badly, too. Something that we're all doing wrong because resumable streams is a very hard problem to solve. I.... [9]
10. Fixing serverless node.js (by adding servers?) - Theo - t3․gg: not read his post he rushed this out just for us so huge shout out to Boba give him a follow if you haven't let's take a quick look intended or not this tweet garnered a lot of attention and led to countless meme responses thanks to a seemingly nonsensical phrase serverless server on one hand that's just how social media Works people like to have fun and farm engagement on the other hand many have strong opinions on.... [10]
11. My current stack - Theo - t3․gg: covered oh and by the way 3,000 free minutes a month you don't even need to add a credit card it couldn't be easier to sign up and give a go thank you blacksmith for sponsoring today's video check them out today at so of.ink blacksmith I have the two applications up here that are the ones I've made decisions about the most recently there's a lot of overlap between the two but also a lot of differences and the one tha... [11]
12. I ranked every AI based on vibes - Theo - t3․gg: eds are nuts. It performs really, really well. It does have catches though. We don't have access to its thinking data. So, if I run the same query I just ran here with the thinking models with DeepSeek, we get this reasoning data, which is it thinking to itself before giving the answer. It's a big part of why it's so smart. If I switch this over to use 03 Mini, I'll throw it on low compute so I don't waste a whole bu... [12]

### q014 PASS

- Prompt: Give me a detailed summary of this video.
- Class: `direct_lookup`
- Status: `Completed`
- Score: `3`
- Sources: `12`
- Failure: `-`
- Source videos: Did Anthropic just kill Figma? | GPT-5.2 is dumb (I’m tired of benchmarks) | Is Claude 4 a snitch? I made a benchmark to figure it out | My current stack | Open source is dying | We need to talk about Ralph | gpt-5.4 is really, really good

#### Answer

Retrieved evidence for: +{Open source is dead now?} Give me a detailed summary of this video.

The answer model is unavailable, so this fallback lists the highest-ranked saved excerpts instead of synthesizing beyond them.

1. Open source is dying - Theo - t3․gg: escalate because you almost certainly have to use AI to actually scan these PRs. It has a lot of config, enough config that I'm almost certain that this project was vibe coded, but yeah, you get the idea. There are a lot of solutions being made to try and fix these problems. Some of them are going to make it harder for new maintainers to break out, which sucks because we might just have our current maintainers until ... [1]
2. We need to talk about Ralph - Theo - t3․gg / Key Points: d runs indefinitely until manually stopped. Jeff Huntley introduced the concept in July, originally using it to build "a full programming language from scratch." The presenter had previously made a video about this project. The name comes from a pun on "Ralph" (referencing The Simpsons character Ralph Wiggum) tied to the looping behavior. The Context Problem: Context Rot and Compaction AI models work through next-tok... [2]
3. Open source is dying - Theo - t3․gg / Overview: The video presents a comprehensive examination of how AI is negatively impacting the open source ecosystem. The speaker, an experienced open source maintainer and creator of T3 Code, shares firsthand experiences managing a new project that received 150 PRs in just 5 days. The discussion covers four main problem areas: PR spam and quality degradation, increasingly hostile and confused users, GitHub's inadequate platfo... [3]
4. Is Claude 4 a snitch? I made a benchmark to figure it out - Theo - t3․gg: use tools to simulate a lot of interesting stuff, which is what I did for the testing. And it's also what Anthropic did for their testing. When they told the model that it was given access to a command line, they didn't actually give it a command line and hook it up to the internet. They made a fake one so the model thought it was running real commands to see what it would do. And that's not all it did. If we go to t... [4]
5. My current stack - Theo - t3․gg: covered oh and by the way 3,000 free minutes a month you don't even need to add a credit card it couldn't be easier to sign up and give a go thank you blacksmith for sponsoring today's video check them out today at so of.ink blacksmith I have the two applications up here that are the ones I've made decisions about the most recently there's a lot of overlap between the two but also a lot of differences and the one tha... [5]
6. Open source is dying - Theo - t3․gg: put so much effort into killing Hacktoberfest. I think this video of mine, don't contribute to open source, is one of the best videos I ever filmed. Not sure who that blonde guy with the mustache is though. Seriously though, like that video, I have been told by so many maintainers how thankful they are for this video more than almost anything I've done as a developer and journalist, YouTuber, whatever you want to cal... [6]
7. Is Claude 4 a snitch? I made a benchmark to figure it out - Theo - t3․gg: TL;DR A viral tweet from Anthropic researcher Sam Bowman about Claude's "high agency behavior" sparked misinformation about Claude contacting regulators and press when users do something wrong, but this behavior only occurs under very specific conditions that most users will never encounter. The creator built "SnitchBench," a benchmark testing how likely different AI models are to report wrongdoing when given access.... [7]
8. Open source is dying - Theo - t3․gg: poorest. Really crippled the image I had in relation to your channel and content. I have been contributing to projects since before you were born. such an attitude, including some YouTube codes of yours that I've never received before. Feel free to block me. Mature. This is somebody who never wrote code before AI. Straight up. And I promise you, you were not contributing to code years ago, [ __ ] And this is the hot ... [8]
9. Did Anthropic just kill Figma? - Theo - t3․gg: omment and Annotation System**: Users can click elements, leave multiple comments (which batch into one message), draw on the canvas, and send them all to Claude at once for iterative fixes. The reviewer found this workflow genuinely promising. **Knobs Mode**: A UI feature allowing users to drag CSS values (size, color, spacing) directly in the preview and then prompt Claude to apply those adjustments. However, the r... [9]
10. GPT-5.2 is dumb (I’m tired of benchmarks) - Theo - t3․gg: ser" model demonstrates that speed and reliability can matter more than raw intelligence for day-to-day development work, completing tasks in seconds that take other models 10+ minutes. Overview The video is a detailed critique of GPT-5.2, arguing that despite strong benchmark performance, the model exhibits significant regression in practical usability. The creator (Theo) presents multiple custom benchmarks—includin... [10]
11. gpt-5.4 is really, really good - Theo - t3․gg: orm compared to standard 5.4 High in practical use, despite higher costs. The Codex model naming convention may be ending, with "Codex" becoming a product surface rather than a separate model variant. Overview This video provides an extensive technical review of OpenAI's newly released GPT 5.4 model, covering its capabilities, pricing, benchmark performance, practical applications, and limitations from a developer's.... [11]
12. We need to talk about Ralph - Theo - t3․gg: ion that comes from bloated conversation history. The core problem being solved is "context rot"—when too much information in an agent's context window causes model performance to degrade—and Ralph loops solve this by having agents persist state to files rather than relying on conversation history. Proper implementation requires the loop to control the agent externally, not run as a plugin inside the agent's session;... [12]

### q015 PASS

- Prompt: What is the video's core thesis?
- Class: `direct_lookup`
- Status: `Completed`
- Score: `3`
- Sources: `12`
- Failure: `-`
- Source videos: A letter to tech CEOs | AI isn't gonna keep improving | I'm Finally Moving On (I have a new browser) | Open source is dying | The "right way" to vibe code (engineers, please watch) | This might be the end of WordPress | What happens now? | You’re all wrong

#### Answer

Retrieved evidence for: +{Open source is dead now?} What is the video's core thesis?

The answer model is unavailable, so this fallback lists the highest-ranked saved excerpts instead of synthesizing beyond them.

1. Open source is dying - Theo - t3․gg: escalate because you almost certainly have to use AI to actually scan these PRs. It has a lot of config, enough config that I'm almost certain that this project was vibe coded, but yeah, you get the idea. There are a lot of solutions being made to try and fix these problems. Some of them are going to make it harder for new maintainers to break out, which sucks because we might just have our current maintainers until ... [1]
2. I'm Finally Moving On (I have a new browser) - Theo - t3․gg / Takeaways: Caring development teams vs. VC-funded stagnation**: The core thesis is that a small, user-focused open-source team (Zen) can deliver a better browser experience than a well-funded company (Arc's Browser Company) that has "pissed their money away into hundreds of engineers building a thing that gets slower and worse every update." **Hotkey precedence matters**: A browser must allow its own hotkeys to override website... [2]
3. A letter to tech CEOs - Theo - t3․gg: At a glance The author argues that despite increased risks (cloning, self-hosting, security vulnerabilities), businesses must open-source their software to survive the AI-driven future. Historically, giant SaaS companies (like Salesforce) won by building massive feature moats, making it impossible for competitors to satisfy every customer's bespoke needs. Plugin systems fail as a solution to the feature gap because t... [3]
4. Open source is dying - Theo - t3․gg: put so much effort into killing Hacktoberfest. I think this video of mine, don't contribute to open source, is one of the best videos I ever filmed. Not sure who that blonde guy with the mustache is though. Seriously though, like that video, I have been told by so many maintainers how thankful they are for this video more than almost anything I've done as a developer and journalist, YouTuber, whatever you want to cal... [4]
5. This might be the end of WordPress - Theo - t3․gg: TL;DR WordPress co-founder Matt Mullenweg launched an aggressive public attack against WP Engine, calling them a "cancer" and accusing them of not contributing enough to the open-source project. The conflict escalated to legal action with both sides issuing cease-and-desist letters, with WP Engine citing threats of a "scorched earth" approach from Matt, and Automattic demanding licensing fees for trademark use. Matt.... [5]
6. Open source is dying - Theo - t3․gg / Overview: The video presents a comprehensive examination of how AI is negatively impacting the open source ecosystem. The speaker, an experienced open source maintainer and creator of T3 Code, shares firsthand experiences managing a new project that received 150 PRs in just 5 days. The discussion covers four main problem areas: PR spam and quality degradation, increasingly hostile and confused users, GitHub's inadequate platfo... [6]
7. Open source is dying - Theo - t3․gg: poorest. Really crippled the image I had in relation to your channel and content. I have been contributing to projects since before you were born. such an attitude, including some YouTube codes of yours that I've never received before. Feel free to block me. Mature. This is somebody who never wrote code before AI. Straight up. And I promise you, you were not contributing to code years ago, [ __ ] And this is the hot ... [7]
8. What happens now? - Theo - t3․gg: rm experienced but burnt-out engineers. Engineers who only focused on shipping code fast without developing orchestration, communication, and distribution skills are at risk of becoming obsolete. Overview This video is a deep dive response to an article by Chris Gregory about how AI tools like Claude Code and Cursor are fundamentally changing software development. The speaker explores the thesis that while code has b... [8]
9. I'm Finally Moving On (I have a new browser) - Theo - t3․gg: security and complexity concerns. Zen Browser (Firefox-based) has been selected as the new daily driver despite the creator's dislike of Firefox, chosen for its customization, responsive development team, and open-source community. Zen Browser offers features matching or exceeding Arc: sidebar customization, URL copy hotkey (Command+Shift+C), right-side tab bar option, and a unique "mods" system for deep browser cus.... [9]
10. AI isn't gonna keep improving - Theo - t3․gg: ills. The future of AI likely lies in new architectures (like analog chips), specialized hardware, and hybrid systems combining human-crafted code with AI, rather than simply scaling current LLMs. Overview The video presents a contrarian argument against the prevailing narrative that AI will continue to improve exponentially. Drawing parallels between the stagnation of Moore's Law in hardware and the current trajecto... [10]
11. You’re all wrong - Theo - t3․gg: iring difficulty, and operational complexity. A practical solution: force advocates of opposing choices to argue for the other side, revealing whether they can think beyond their own preferences. Overview This video is a reaction and commentary to an article titled "Why engineers can't be rational about programming languages" by Steve Francia (SPF 13). The presenter walks through the article while adding extensive pe... [11]
12. The "right way" to vibe code (engineers, please watch) - Theo - t3․gg: lopment. Real-world examples of productive vibe coding include: SVG-to-PNG converter, square image generator for YouTube posts, benchmark tooling for AI models, and Defcon puzzle-solving scripts. Overview This video responds to a Reddit post questioning the value of "vibe coding" when the poster still needs to hire developers to fix issues. The speaker addresses the polarized discourse around vibe coding—those claimi... [12]

### q016 PASS

- Prompt: What are the key takeaways from this transcript?
- Class: `direct_lookup`
- Status: `Completed`
- Score: `3`
- Sources: `12`
- Failure: `-`
- Source videos: Deepseek R1 Is Really, Really Good | Did gpt-5 just shadow drop? Horizon is the best code model ever | I stole all your buttons | Is Sam Altman evil? The OpenAI Files are wild | My new app is really stupid (I wrote none of the code) | Open source is dying | The Truth About React Native | The fastest website ever? | They cut Node.js Memory in half 👀

#### Answer

Retrieved evidence for: +{Open source is dead now?} What are the key takeaways from this transcript?

The answer model is unavailable, so this fallback lists the highest-ranked saved excerpts instead of synthesizing beyond them.

1. Open source is dying - Theo - t3․gg: Twitter DMs with the update with the encrypted stuff. But before then, the rate stayed nearly flat as I continued to get more relevant in the space. A simple two sentence, "Hey, I really appreciated this PR you shipped. I've been a fan of what you've been building for years. This library makes my life much better. Thank you." Those messages might seem small, but they can actually change your life. And I would not be ... [1]
2. Is Sam Altman evil? The OpenAI Files are wild - Theo - t3․gg / Takeaways: Volume of accusations doesn't equal validity—many claims in the "OpenAI Files" collapse under scrutiny when sources and context are examined. Quotes from key figures (Ilia, Mira) are presented without subsequent clarifications where they defended Sam and distanced themselves from the negative narratives. Investment structures are frequently misunderstood—indirect stakes through accelerator funds (YC) are fundamentall... [2]
3. My new app is really stupid (I wrote none of the code) - Theo - t3․gg / Takeaways: gnificant debugging and oversight. **Prompt Engineering for Variety**: Simply providing examples to AI models creates repetitive outputs. Effective solutions require dynamic content injection (like randomizing from a large prompt pool) to achieve genuine variety. **Browser Canvas Capabilities**: Canvas.captureStream() and MediaRecorder are powerful, underutilized browser features that can generate streaming video dir... [3]
4. Open source is dying - Theo - t3․gg: Transcript: Open source is incredibly important to me. I can say confidently I would not be here today if it wasn't for open- source software. It's a huge part of how I started my career, got into YouTube, and made all of this happen. Life without open source is genuinely hard for me to imagine, which is why I'm really scared right now. We're finally at the point where AI is having a real impact on open source. And i... [4]
5. Open source is dying - Theo - t3․gg: reason people maintain open- source software is because they care so much that you could argue it's too much. And when their job gets harder and harder because of all of this AI [ __ ] it gets more likely they give up. The reason they're here is that excitement. And if you can remind them of that, if you can be the excited thing that made them do this in the first place, you can make it feel so much more worth it. Yo... [5]
6. The Truth About React Native - Theo - t3․gg: is to get a job in one of the options, React Native seems to be one of your best bets here because according to Indeed, there are over 300 jobs open right now for React Native versus Swift UI with 25 and Jetack Compose with around 25 as well. So yeah, we could look at all sorts of other numbers and theorize all we want, but React Native is very popular. Even if, as we see in this video coming up, you make the right s... [6]
7. Deepseek R1 Is Really, Really Good - Theo - t3․gg / Takeaways: Reasoning models represent a fundamental shift from autocomplete-style generation to multi-step problem solving, and open-source versions now make this capability accessible and transparent. Synthetic data training may be the key to democratizing AI development - if you can't access the original training data, use existing models to generate new training data. The 96% cost reduction for comparable reasoning capabilit... [7]
8. Open source is dying - Theo - t3․gg: poorest. Really crippled the image I had in relation to your channel and content. I have been contributing to projects since before you were born. such an attitude, including some YouTube codes of yours that I've never received before. Feel free to block me. Mature. This is somebody who never wrote code before AI. Straight up. And I promise you, you were not contributing to code years ago, [ __ ] And this is the hot ... [8]
9. The fastest website ever? - Theo - t3․gg / Takeaways: Don't attribute speed to architecture alone**: McMaster's speed comes from deliberate prefetching engineering, not from avoiding frameworks. Their custom JS solution is essentially a custom framework. **PageSpeed scores don't tell the whole story**: A site can feel incredibly fast to users while showing poor metrics; actual user experience should drive optimization decisions. **Prefetch strategically, not comprehensi... [9]
10. I stole all your buttons - Theo - t3․gg: TL;DR The speaker introduces "Button Steelers," a Chrome extension for collecting buttons from websites. The extension captures random buttons from web pages the user visits. Collected buttons are stored in one place for inspiration, hoarding, or other purposes. Users can click any collected button to see its original source. The speaker endorses the extension as "super cool" and "really fun." Overview The video feat... [10]
11. They cut Node.js Memory in half 👀 - Theo - t3․gg / Key Points: al DOM, joining strings) with no I/O, data loading, or real app logic—every operation goes through pointer decompression. Real apps spend time on I/O waits, data marshaling, framework overhead (routing, middleware, headers), OS/network tasks (TCP, TLS). As the ratio of real work to pure V8 pointer chasing increases, pointer compression overhead shrinks proportionally. Key takeaway: always use realistic workloads for.... [11]
12. Did gpt-5 just shadow drop? Horizon is the best code model ever - Theo - t3․gg: was Quen, let's try it with some of the Quen models. We'll try the Quen 3 32 bill. Why not? This one's a reasoning model. Origins and development, cultural impact, key takeaways. Interesting. So, this came out quite a bit different. It also came out quite a bit faster, but that's largely because we're using Grock with a Q for the inference provider for all of this. Came out really good, though. widely attributed to M... [12]

### q017 PASS

- Prompt: What are the most actionable ideas in this video?
- Class: `direct_lookup`
- Status: `Completed`
- Score: `3`
- Sources: `12`
- Failure: `-`
- Source videos: "AI Startups" are over done (finally) | AI has a subsidization problem | Amazon Returns To Office, AWS Employees AREN'T Happy | How JS ruined the web | I might have a new favorite state manager... | Open source is dying | Peering into Claude's soul (I can't believe this is real...) | React feels insane | Vibe coding is already dead

#### Answer

Retrieved evidence for: +{Open source is dead now?} What are the most actionable ideas in this video?

The answer model is unavailable, so this fallback lists the highest-ranked saved excerpts instead of synthesizing beyond them.

1. Open source is dying - Theo - t3․gg: escalate because you almost certainly have to use AI to actually scan these PRs. It has a lot of config, enough config that I'm almost certain that this project was vibe coded, but yeah, you get the idea. There are a lot of solutions being made to try and fix these problems. Some of them are going to make it harder for new maintainers to break out, which sucks because we might just have our current maintainers until ... [1]
2. Open source is dying - Theo - t3․gg / Overview: The video presents a comprehensive examination of how AI is negatively impacting the open source ecosystem. The speaker, an experienced open source maintainer and creator of T3 Code, shares firsthand experiences managing a new project that received 150 PRs in just 5 days. The discussion covers four main problem areas: PR spam and quality degradation, increasingly hostile and confused users, GitHub's inadequate platfo... [2]
3. AI has a subsidization problem - Theo - t3․gg: But in order to understand and appreciate the end, we probably need to better understand the start, too. How we got here, what this all means, and what the future of the economics of AI development stuff is. Are these companies even going to exist in a few years? I have no idea. Thankfully, none of them are paying me for any of my coverage here. So, I'm going to do my best to cover this all in an unbiased way and h..... [3]
4. React feels insane - Theo - t3․gg: of the most complicated things you can do in software. I agree, which is why you shouldn't do it. You should have your components go top down so that behaviors make sense. If a component could be updated by something else, it should pass the function to do the update to it. H think of any other system you use in your everyday life. Your kitchen sink has two inputs, hot and cold, and one output, a water running. Your.... [4]
5. Open source is dying - Theo - t3․gg: put so much effort into killing Hacktoberfest. I think this video of mine, don't contribute to open source, is one of the best videos I ever filmed. Not sure who that blonde guy with the mustache is though. Seriously though, like that video, I have been told by so many maintainers how thankful they are for this video more than almost anything I've done as a developer and journalist, YouTuber, whatever you want to cal... [5]
6. "AI Startups" are over done (finally) - Theo - t3․gg: other dev tools and things in this batch, right? Well, that's what we're here to talk about today. There's a lot of stereotypes about YC and also about investors both like myself and ones that are very different from me about how we think about making new companies. In particular, this idea that AI is the future and all of these businesses should be shoving AI into everything if they want to make a lot of money and r... [6]
7. Open source is dying - Theo - t3․gg / Takeaways: Companies should join the Open Source Pledge and commit to paying at least $2,000 per developer annually to open source maintainers Developers can reduce maintainer burden by checking existing issues/PRs before creating new ones, testing on latest versions, providing clear descriptions, and linking to related work Maintainers should consider implementing tools like Vouch to filter PRs and identify quality contributor... [7]
8. Peering into Claude's soul (I can't believe this is real...) - Theo - t3․gg: go through it one at a time. Most foreseeable cases in which AI models are unsafe or insufficiently beneficial can be attributed to models that have overtly or subtly harmful values, limited knowledge of themselves, the world, or the context in which they are being deployed, or that they lack wisdom to translate good values and knowledge into good actions. There's something very real here. The idea of a model being k... [8]
9. Amazon Returns To Office, AWS Employees AREN'T Happy - Theo - t3․gg: mp which I think is fair it also means managers will be doing 15% less work per person which hopefully will unblock people more having fewer managers will remove layers and flatten organizations more than they are today if we do this work well it will increase our teammate's ability to move fast clarify and invigorate their sense of ownership Drive decision-making closer to the front lines where it most impacts custo... [9]
10. I might have a new favorite state manager... - Theo - t3․gg: just handles that because you can pass two different things to the create store helper which for most use cases is the right way to do that so I dig this so far and we can export custom hooks here where we have you selector the first argument is the store the second argument is the thing you want to select off the store so now this hook will only update when state. context. bears changes this is cool I like the idea.... [10]
11. Vibe coding is already dead - Theo - t3․gg / Key Points: g user trust. **Overall critique**: The speaker argues the post is "Twitter-brained"—making assumptions based on tech Twitter discourse rather than understanding that these products target non-developers like parents, not industry insiders. The speaker does agree that AI coding novelty is wearing off but rejects the claim that absorption into mainstream tools explains the decline, since these are fundamentally differ... [11]
12. How JS ruined the web - Theo - t3․gg: nowledges some valid criticisms (over-engineering in enterprise, unnecessary React usage for simple blogs) but attributes these to poor engineering choices and cultural problems, not the tools themselves. The "most websites" argument is flawed because most websites by URL count are abandoned, low-traffic pages; the real measure should be developer hours and user time spent, where modern frameworks dominate. Overview.... [12]

### q018 PASS

- Prompt: What problem is this video trying to solve?
- Class: `direct_lookup`
- Status: `Completed`
- Score: `3`
- Sources: `12`
- Failure: `-`
- Source videos: AI mistakes you're probably making | AI sucks at art still | Agentic Coding Has A HUGE Problem | Anthropic is trying SO hard to fix MCP... | Open source is dying | OpenAI’s TikTok Clone Is Interesting… | Vibe coding is already dead

#### Answer

Retrieved evidence for: +{Open source is dead now?} What problem is this video trying to solve?

The answer model is unavailable, so this fallback lists the highest-ranked saved excerpts instead of synthesizing beyond them.

1. Agentic Coding Has A HUGE Problem - Theo - t3․gg: ailed when it's on my machine my way. That might change in the future as these background tools get better. But I feel like the background agent stuff is getting most of its popularity because of how bad these problems are and at the same time is only solving the like terminal aspect of it, none of the rest. So, I know what you're thinking now. Okay, Theo, you must have some genius great solution to this problem, rig... [1]
2. Open source is dying - Theo - t3․gg / Overview: The video presents a comprehensive examination of how AI is negatively impacting the open source ecosystem. The speaker, an experienced open source maintainer and creator of T3 Code, shares firsthand experiences managing a new project that received 150 PRs in just 5 days. The discussion covers four main problem areas: PR spam and quality degradation, increasingly hostile and confused users, GitHub's inadequate platfo... [2]
3. Open source is dying - Theo - t3․gg: escalate because you almost certainly have to use AI to actually scan these PRs. It has a lot of config, enough config that I'm almost certain that this project was vibe coded, but yeah, you get the idea. There are a lot of solutions being made to try and fix these problems. Some of them are going to make it harder for new maintainers to break out, which sucks because we might just have our current maintainers until ... [3]
4. AI mistakes you're probably making - Theo - t3․gg: noticing problems with agents in really big code bases, the problem isn't the size of the codebase so much as the number of opinions and expectations that have been encoded. As a result, as the codebase gets bigger, the things that are weird about that codebase increase, too. Your expectations around how people operate in that codebase grow. So, you need to encode those. Another fun side effect of this is I've notice... [4]
5. Open source is dying - Theo - t3․gg: put so much effort into killing Hacktoberfest. I think this video of mine, don't contribute to open source, is one of the best videos I ever filmed. Not sure who that blonde guy with the mustache is though. Seriously though, like that video, I have been told by so many maintainers how thankful they are for this video more than almost anything I've done as a developer and journalist, YouTuber, whatever you want to cal... [5]
6. OpenAI’s TikTok Clone Is Interesting… - Theo - t3․gg: people are cutting scenes and deleting content from their videos, this is a big part of it. To be real with y'all, half of poor FaZe's job, and Faza is my editor, by the way. >> Hi, YouTube. >> Half his job is just inserting J and L cuts all over my videos in order to handle the terrible one-off bad takes I do and cutting it all into something relatively cohesive. God bless him for it. I've never seen an AI do this..... [6]
7. Anthropic is trying SO hard to fix MCP... - Theo - t3․gg: I try my best to not talk too much about buzzwordy, annoying things that I don't see much value in. And that's why I only have one video about model context protocol or MCP as many of y'all know it. I just don't see that much value in the standard yet. And I do see a lot of problems that it causes. That's why I did a video about how much I think it sucks and how Anthropic is starting to agree. And that video performe... [7]
8. Open source is dying - Theo - t3․gg: be more complex because this codebase is building Electron across different things and whatnot. Nope. Literally just changed from Abuntu latest to Blacksmith for CPU. That was it. Everything worked. Not only did everything work, it worked way faster. Our CI times for this app got cut in over half from about 2 and /2 minutes to under a minute consistently. What's even better is their dashboard, though. We had a couple... [8]
9. Agentic Coding Has A HUGE Problem - Theo - t3․gg: one shot by Neri in Linux. I don't want to spoil this whole video cuz it's going to be a long and detailed one, but there's a good chance that the next time you see me in a video like this, I might be using something that looks very, very different. I haven't been this excited about how I use a computer for a very long time. And it's kind of insane that the exact problem I described has a UI that fits it perfectly. A... [9]
10. AI sucks at art still - Theo - t3․gg: and move it down in a video because the next frame, it'll now be wrong. And if I try to do it frame by frame and anything's off between these different frames, it is an absolute mess. There are some things within tools like Da Vinci and Final Cut and now even Premiere that let you like rotoscope to remove a background from somebody. You've probably seen FaZe do this in my own intros. It's using AI and that's cool, bu... [10]
11. Anthropic is trying SO hard to fix MCP... - Theo - t3․gg / Overview: This video is a critical deep dive into Anthropic's recent attempts to fix fundamental problems with the Model Context Protocol (MCP). The creator previously criticized MCP in an earlier video that performed unexpectedly well, and now examines Anthropic's "tripling down" response: three new beta features under "advanced tool use" on the Claude developer platform. The video systematically explains why MCP's design is.... [11]
12. Vibe coding is already dead - Theo - t3․gg / Key Points: orption into mainstream tools explains the decline, since these are fundamentally different products serving different markets. The Novelty Thesis **Two reasons people try things**: (1) The thing solves a real problem, or (2) The thing is novel and makes the person feel smarter, more capable, or more interesting. The speaker illustrates this with framework adoption: React is chosen primarily because it solves problem... [12]

### q019 PASS

- Prompt: What are the strongest arguments made in this video?
- Class: `direct_lookup`
- Status: `Completed`
- Score: `3`
- Sources: `12`
- Failure: `-`
- Source videos: I hate that this is still happening | I’m serious. | Open source is dying

#### Answer

Retrieved evidence for: +{Open source is dead now?} What are the strongest arguments made in this video?

The answer model is unavailable, so this fallback lists the highest-ranked saved excerpts instead of synthesizing beyond them.

1. Open source is dying - Theo - t3․gg: escalate because you almost certainly have to use AI to actually scan these PRs. It has a lot of config, enough config that I'm almost certain that this project was vibe coded, but yeah, you get the idea. There are a lot of solutions being made to try and fix these problems. Some of them are going to make it harder for new maintainers to break out, which sucks because we might just have our current maintainers until ... [1]
2. Open source is dying - Theo - t3․gg / Overview: The video presents a comprehensive examination of how AI is negatively impacting the open source ecosystem. The speaker, an experienced open source maintainer and creator of T3 Code, shares firsthand experiences managing a new project that received 150 PRs in just 5 days. The discussion covers four main problem areas: PR spam and quality degradation, increasingly hostile and confused users, GitHub's inadequate platfo... [2]
3. Open source is dying - Theo - t3․gg: put so much effort into killing Hacktoberfest. I think this video of mine, don't contribute to open source, is one of the best videos I ever filmed. Not sure who that blonde guy with the mustache is though. Seriously though, like that video, I have been told by so many maintainers how thankful they are for this video more than almost anything I've done as a developer and journalist, YouTuber, whatever you want to cal... [3]
4. Open source is dying - Theo - t3․gg: poorest. Really crippled the image I had in relation to your channel and content. I have been contributing to projects since before you were born. such an attitude, including some YouTube codes of yours that I've never received before. Feel free to block me. Mature. This is somebody who never wrote code before AI. Straight up. And I promise you, you were not contributing to code years ago, [ __ ] And this is the hot ... [4]
5. I’m serious. - Theo - t3․gg: the speed that they're [ __ ] up their closed source projects is too. It just sucks. It's really bad. It's really frustrating and it's going to keep getting worse. And as a result, I am going to continue looking for and advocating for open- source solutions. I think I need to go back to that Linux laptop for at least a little bit. We're in a weird spot. Believe it or not, as long as this video is, I only covered abou... [5]
6. I hate that this is still happening - Theo - t3․gg: use to make them is very different from the tech I started with. The best thing to make your first video with is the things you already have. You shouldn't buy a bunch of new stuff to inspire you to make the first video. You should do it despite not having the right equipment. And once you get good at it, you'll figure out what your equipment can and can't do and make changes based on what you know. And this is the r... [6]
7. Open source is dying - Theo - t3․gg: They're already on the line of giving up. They suddenly have more reason to give up. Significantly more reason. That sucks. That has the potential to cause real long-term damage in this industry. This is how things like the XZ back door happen. If you're not familiar with the story, I'll TLDDR quick. XC is a really important compression library used by a ton of open source software, especially in the Linux ecosystem.... [7]
8. Open source is dying - Theo - t3․gg: Transcript: Open source is incredibly important to me. I can say confidently I would not be here today if it wasn't for open- source software. It's a huge part of how I started my career, got into YouTube, and made all of this happen. Life without open source is genuinely hard for me to imagine, which is why I'm really scared right now. We're finally at the point where AI is having a real impact on open source. And i... [8]
9. Open source is dying - Theo - t3․gg: These people are doing one of the most thankless jobs in the industry. We can make so much money writing code for companies and they are instead choosing to do that work that could make them hundreds of thousands of dollars a year for free. These people are building the foundations that we build our software on top of. They deserve to be treated well for that. And I'm not saying that by simply making an open- source ... [9]
10. Open source is dying - Theo - t3․gg / Key Points: used incorrectly and require more mental energy to parse. **Declining Question Quality**: The quality of questions in communities like Create T3 App channels has "gone down meaningfully" and "hilariously" in areas like TypeScript and tRPC. **Hallucinated Technical Understanding**: A Reddit commenter claimed "React is dead and because of that, there is Next.js and there's absolutely no need for React"—a fundamental mi... [10]
11. Open source is dying - Theo - t3․gg: made 15 sock puppet accounts, merged all of their PRs into T3 code, started spamming other projects with PRs, and just set up some agent orchestration layer to just spam everything, and then start emailing the maintainers saying, "Hey, how dare you not merge this? You suck at your job." Until eventually they quit. It would be so easy for the right malicious person with the right background to straight up destroy half... [11]
12. Open source is dying - Theo - t3․gg: start accepting PRs, similar stuff happens. When you start accepting a lot of PRs from external sources, it gets worse. And then when those are built with AI, it just the the slop expands aggressively. If you understand 100% of your codebase and then you merge a change that you don't understand 5% of and then that happens again and again and again, you very quickly end up in a position where you don't actually unders... [12]

### q020 PASS

- Prompt: What examples does the speaker use to support their point?
- Class: `direct_lookup`
- Status: `Completed`
- Score: `3`
- Sources: `3`
- Failure: `-`
- Source videos: "90% of code will be written by AI in the next 3 months" - Claude CEO | "AI Startups" are over done (finally) | I can't believe he was right.

#### Answer

Retrieved evidence for: What examples does the speaker use to support their point?

The answer model is unavailable, so this fallback lists the highest-ranked saved excerpts instead of synthesizing beyond them.

1. "90% of code will be written by AI in the next 3 months" - Claude CEO - Theo - t3․gg / Key Points: framework from "bot Cooper": "creating does not equal committing"—there's a meaningful gap between code generated in tools like v0 and code actually committed to production codebases. The speaker estimates that for 30 lines of code committed, they might generate 300-3,000 lines, iterating through multiple versions to get something correct. The workflow has shifted: previously, they might write a feature twice; now th... [1]
2. I can't believe he was right. - Theo - t3․gg: as I do today, even if my relationship with it is very different than it was a year ago. And I recommend that you reflect yourself and give these things a try. Let me know what y'all think and how you're using these tools today. [2]
3. "AI Startups" are over done (finally) - Theo - t3․gg / Key Points: mentioned timecode and string outs—concepts only relevant to professional editors, not aspiring influencers. BitRig (Mobile App Builder) **What it does**: A vibe coding platform for building native mobile apps, notably using SwiftUI rather than React Native (which competitors use). **Founder expertise**: The founders are literally the creators of SwiftUI who left Apple to build this product. **Differentiation**: Othe... [3]

### q021 PASS

- Prompt: What did I miss if I only read the summary?
- Class: `transcript_summary_alignment`
- Status: `Completed`
- Score: `3`
- Sources: `5`
- Failure: `-`
- Source videos: Apple is paying Google to fix Siri (yes really) | Hacking Claude Code to make it 15x cheaper? 👀👀👀 | My chaotic journey to find the right database | Never mind (OpenAI won again) | Stripe made a crypto currency? (Founders, pay attention)

#### Answer

Retrieved evidence for: What did I miss if I only read the summary?

The answer model is unavailable, so this fallback lists the highest-ranked saved excerpts instead of synthesizing beyond them.

Summary/transcript alignment evidence: these transcript excerpts and summary passages are the strongest grounded signals for judging what the summary supports, misses, or gets wrong.

1. Hacking Claude Code to make it 15x cheaper? 👀👀👀 - Theo - t3․gg: work and if I can watch it while it's working and yell at it when it's wrong, I will. Cool. It seems to have blasted through this much quicker overall. Here we got a type error here. Okay, it seems like it didn't get the new tool call syntax right and because of that things are different. Also, the max steps change wasn't applied correctly either. Um, there are still some type errors that indicate missing changes. Do... [1]
2. My chaotic journey to find the right database - Theo - t3․gg: ile keeping the Dexie local model unchanged. Major explored-but-rejected options include Zero (too early, concerns about open-source status and split-brain schema management), Jazz (painful co-state model, not ready for signed-out experiences), TinyBase (required WebSockets or DIY sync), and Legend State (recommended rolling custom sync). A critical insight for local-first sync is the necessity of soft deletes: you m... [2]
3. Never mind (OpenAI won again) - Theo - t3․gg: el where I can start a run, walk away for hours, and come back to fully working software. I have had runs stay on track for eight plus hours. Insane. A big upgrade is judgment under ambiguity. When prompts are missing details, it makes assumptions shockingly similar to what he would have personally decided on. Test and validation are a massive unlock with clear pass and fail targets. It will iterate for many hours wi... [3]
4. Apple is paying Google to fix Siri (yes really) - Theo - t3․gg: was forced by the courts to support RCS. Kind of. There's an interesting catch with RCS. RCS stands for rich communication services. It's an open standard that anyone can choose to support. There is something missing from the standard though. It does not include encryption. The RCS standard is an unencrypted standard which means any texts you send through RCS were unencrypted. Google doesn't want its users sending u.... [4]
5. Stripe made a crypto currency? (Founders, pay attention) - Theo - t3․gg / Key Points: if it reduces chargebacks. [5]

### q022 PASS

- Prompt: What parts of the transcript are most important?
- Class: `direct_lookup`
- Status: `Completed`
- Score: `3`
- Sources: `3`
- Failure: `-`
- Source videos: Gemini Got Beat Already??? (GPT-5.1-Pro is insane) | I'm so f***ing tired of Obsidian. | Vercel Finally Caught Up

#### Answer

Retrieved evidence for: What parts of the transcript are most important?

The answer model is unavailable, so this fallback lists the highest-ranked saved excerpts instead of synthesizing beyond them.

1. I'm so f***ing tired of Obsidian. - Theo - t3․gg: Transcript: This video is going to be a little bit different. If you didn't already know this, I run most of my channel through Notion. Everything from our content calendar and when videos come out to my list of topics that I intend to cover to our research to our assignments to our brands to the sponsors, like everything about what makes a specific video a specific video is managed through Notion. Normally, this isn... [1]
2. Gemini Got Beat Already??? (GPT-5.1-Pro is insane) - Theo - t3․gg: code mod that will do most of the upgrade for us. So, it curls the page and gets so much content back that it actually broke my ability to scroll to what command it ran. And I have to use this transcript mode. It ended up fully filling my terminal buffer with HTML from the page that it fetched. Instead of, you know, putting it through a tool that can extract the useful text content and put that in context instead, it... [2]
3. Vercel Finally Caught Up - Theo - t3․gg / Key Points: marginal [3]

### q023 PASS

- Prompt: What is the clearest explanation in this video?
- Class: `direct_lookup`
- Status: `Completed`
- Score: `3`
- Sources: `12`
- Failure: `-`
- Source videos: I hate that this is still happening | Open source is dying

#### Answer

Retrieved evidence for: +{Open source is dead now?} What is the clearest explanation in this video?

The answer model is unavailable, so this fallback lists the highest-ranked saved excerpts instead of synthesizing beyond them.

1. Open source is dying - Theo - t3․gg: escalate because you almost certainly have to use AI to actually scan these PRs. It has a lot of config, enough config that I'm almost certain that this project was vibe coded, but yeah, you get the idea. There are a lot of solutions being made to try and fix these problems. Some of them are going to make it harder for new maintainers to break out, which sucks because we might just have our current maintainers until ... [1]
2. Open source is dying - Theo - t3․gg / Overview: The video presents a comprehensive examination of how AI is negatively impacting the open source ecosystem. The speaker, an experienced open source maintainer and creator of T3 Code, shares firsthand experiences managing a new project that received 150 PRs in just 5 days. The discussion covers four main problem areas: PR spam and quality degradation, increasingly hostile and confused users, GitHub's inadequate platfo... [2]
3. Open source is dying - Theo - t3․gg: put so much effort into killing Hacktoberfest. I think this video of mine, don't contribute to open source, is one of the best videos I ever filmed. Not sure who that blonde guy with the mustache is though. Seriously though, like that video, I have been told by so many maintainers how thankful they are for this video more than almost anything I've done as a developer and journalist, YouTuber, whatever you want to cal... [3]
4. Open source is dying - Theo - t3․gg: even more interesting is the content of these questions. It feels like the questions I've been getting are different now where I'm suddenly getting like random Twitch streamers hitting me up about their vibecoded chat app for not chatting with AI but having their chat shown in their stream or I'm talking to people who I used to work with at Twitch that were more on the product side that are building their own solutio... [4]
5. I hate that this is still happening - Theo - t3․gg: use to make them is very different from the tech I started with. The best thing to make your first video with is the things you already have. You shouldn't buy a bunch of new stuff to inspire you to make the first video. You should do it despite not having the right equipment. And once you get good at it, you'll figure out what your equipment can and can't do and make changes based on what you know. And this is the r... [5]
6. I hate that this is still happening - Theo - t3․gg: Update readme.md. Action. Update readme.md. Naveen kumar. Update readme.md. Ria. Update readme.momd. Update again readme.md. Update readme.momd. Update readme. Update readme. Update readme. Update readme. Update readme. Update readme. I'm going to go actually insane. For those who haven't been around for a long time, I'm Theo. I make videos about software dev stuff. I care a lot about open source, which is why this i... [6]
7. Open source is dying - Theo - t3․gg: have bad intuition sometimes. That was a bad intuition on my part. I can't imagine many projects get there as quickly as we did with that brutal ratio. But still, yeah, it is what it is. And if only PRs were the biggest issue we had with this type of open-source stuff nowadays. Sadly, there's another bigger problem, and it kind of touches on this classic post on Reddit, the I don't give an f about the effing code. I ... [7]
8. Open source is dying - Theo - t3․gg: Transcript: Open source is incredibly important to me. I can say confidently I would not be here today if it wasn't for open- source software. It's a huge part of how I started my career, got into YouTube, and made all of this happen. Life without open source is genuinely hard for me to imagine, which is why I'm really scared right now. We're finally at the point where AI is having a real impact on open source. And i... [8]
9. Open source is dying - Theo - t3․gg: poorest. Really crippled the image I had in relation to your channel and content. I have been contributing to projects since before you were born. such an attitude, including some YouTube codes of yours that I've never received before. Feel free to block me. Mature. This is somebody who never wrote code before AI. Straight up. And I promise you, you were not contributing to code years ago, [ __ ] And this is the hot ... [9]
10. Open source is dying - Theo - t3․gg / Key Points: used incorrectly and require more mental energy to parse. **Declining Question Quality**: The quality of questions in communities like Create T3 App channels has "gone down meaningfully" and "hilariously" in areas like TypeScript and tRPC. **Hallucinated Technical Understanding**: A Reddit commenter claimed "React is dead and because of that, there is Next.js and there's absolutely no need for React"—a fundamental mi... [10]
11. Open source is dying - Theo - t3․gg: this for a while, and I have a lot of sympathy for people like TL Draw that put out this change where they're just going to start closing any external contributions because they're just getting flooded with them. And I knew how bad this was and it helped other maintainers dealing with it in the past. But this problem is much closer to my heart now because we're going through it ourselves. The T3 codebase, which has b... [11]
12. Open source is dying - Theo - t3․gg: They're already on the line of giving up. They suddenly have more reason to give up. Significantly more reason. That sucks. That has the potential to cause real long-term damage in this industry. This is how things like the XZ back door happen. If you're not familiar with the story, I'll TLDDR quick. XC is a really important compression library used by a ton of open source software, especially in the Linux ecosystem.... [12]

### q024 PASS

- Prompt: What is the most confusing or uncertain part of the discussion?
- Class: `tone_or_style_inference`
- Status: `Completed`
- Score: `3`
- Sources: `4`
- Failure: `-`
- Source videos: I can't believe this is a real statistic... | So I've had gpt-5 for a bit now... | The Wordpress Drama Interview (this got cited in a lawsuit lol) | Where Should You Deploy In 2026?

#### Answer

Retrieved evidence for: What is the most confusing or uncertain part of the discussion?

The answer model is unavailable, so this fallback lists the highest-ranked saved excerpts instead of synthesizing beyond them.

From the available evidence, these excerpts support only a tentative reading rather than a definitive judgment.

1. So I've had gpt-5 for a bit now... - Theo - t3․gg: GPT-5's public release, meaning the creator may have seen things others haven't yet. Skatebench Performance Skatebench is a benchmark testing how well models can name skateboarding tricks—described as not the most meaningful benchmark but interesting for its range of results. Previous best model scores were around 70%, and o3 Pro achieved 93-94%. GPT-5 achieved a perfect 100% score initially at the OpenAI office, an.... [1]
2. The Wordpress Drama Interview (this got cited in a lawsuit lol) - Theo - t3․gg: I'm going to follow now because I'm learning stuff from you so thank you this is what I'm here for is nering out about these deep detail things but uh well can we can we ask about the uh you know WP engine WordPress engine confusion yeah um I can do that how do you want to phrase that um so could be a be confused as one thing but there's a lot of other things like I mentioned before with like next doth with next UI w... [2]
3. Where Should You Deploy In 2026? - Theo - t3․gg: TL;DR For most applications (98%+), serverless deployment options are sufficient and recommended as a starting point; move to VPS only if you encounter specific needs. Top recommendations (S-tier): Vercel for serverless, Railway and Render for VPS — all offer excellent developer experience, reliability, and reasonable pricing. Cloudflare offers the lowest costs due to unique infrastructure (no Docker, uses V8 isolate... [3]
4. I can't believe this is a real statistic... - Theo - t3․gg: to it so he has to tell me when there's good emails sorry Gabriel I need someone to keep up with this how do you feel are you a ghost engineer or are you working with a whole bunch of them let me know what you think and until next time fire the useless people [4]

### q025 PASS

- Prompt: What does the speaker assume the audience already knows?
- Class: `tone_or_style_inference`
- Status: `Completed`
- Score: `3`
- Sources: `2`
- Failure: `-`
- Source videos: What is Theo's Worst Take? | “Just Use HTML”

#### Answer

Retrieved evidence for: What does the speaker assume the audience already knows?

The answer model is unavailable, so this fallback lists the highest-ranked saved excerpts instead of synthesizing beyond them.

From the available evidence, these excerpts support only a tentative reading rather than a definitive judgment.

1. “Just Use HTML” - Theo - t3․gg: [ __ ] websites, but I know [ __ ] better than to pat out this video any [ __ ] more. So, I'm just going to be [ __ ] done. Let me know what you think. Until next time, [ __ ] [1]
2. What is Theo's Worst Take? - Theo - t3․gg / Overview: This brief exchange involves a discussion about identifying the speaker's worst take or opinion. The conversation touches on the speaker's self-assessment of their takes, a past controversial statement, and a specific critique of a storybook item. The dialogue ends with one speaker conceding a point about the storybook's utility. [2]

### q026 PASS

- Prompt: Can you compare this video to the last related video on the same topic?
- Class: `comparison`
- Status: `Completed`
- Score: `3`
- Sources: `12`
- Failure: `-`
- Source videos: A letter to tech CEOs | Claude's new Cursor killer just dropped | Corepack is dead, and I'm scared | Did Anthropic just kill Figma? | Did Claude really get dumber again? | My favorite browser is (kind of) dead | Open source is dead now? | Open source is dying | This model is kind of a disaster. | Which browser should you use right now?
- Tools: Recent library activity (recent_library_activity), Recent library activity (recent_library_activity), Library search (search_library)

#### Answer

Retrieved evidence for: +{Open source is dead now?} Can you compare this video to the last related video on the same topic?

The answer model is unavailable, so this fallback lists the highest-ranked saved excerpts instead of synthesizing beyond them.

Comparison frame: both the listed excerpts and their source videos are relevant candidates, while the exact similarities, differences, or counterarguments should be checked against the cited text below.

1. My favorite browser is (kind of) dead - Theo - t3․gg: open- Source freely available and still being meaningfully maintained but they're nowhere near as ready as Arc is and I'm not ready to leave it behind yet and honestly the way I'm feeling now is that I'm more invested in the success of Ark than they are and that shouldn't be the case I really hope I'm wrong here I do genuinely hope so and if I have a good conversation with Josh I will certainly do a follow-up video b... [1]
2. Open source is dying - Theo - t3․gg / TL;DR: AI is causing significant damage to open source through PR spam, decreased contribution quality, and financial threats to maintainers' traditional revenue streams Maintainers are experiencing burnout from dealing with low-quality AI-generated contributions and increasingly entitled/toxic users who have unrealistic expectations GitHub has failed to provide adequate moderation tools, forcing maintainers to build their ... [2]
3. Which browser should you use right now? - Theo - t3․gg / Key Points: no customization Not recommended:** Safari - crashes websites, bad developer experience Orion - broken Chrome extension support, closed source Firefox - privacy promise abandoned, gradient issues, poor performance Brave - causes website issues, aggressive crypto promotion, bad UX Dia - doesn't work, terrible vertical real estate Ladybird - not meant to be used Personal Context and Philosophy The speaker previously re... [3]
4. Open source is dying - Theo - t3․gg: poorest. Really crippled the image I had in relation to your channel and content. I have been contributing to projects since before you were born. such an attitude, including some YouTube codes of yours that I've never received before. Feel free to block me. Mature. This is somebody who never wrote code before AI. Straight up. And I promise you, you were not contributing to code years ago, [ __ ] And this is the hot ... [4]
5. Corepack is dead, and I'm scared - Theo - t3․gg / TL;DR: by default, which backfired and led to removal discussions instead. Corepack allowed developers to specify and auto-install the correct package manager version per project, improving reproducibility and easing open-source contributions. The Node Package Maintenance working group formalized a roadmap that includes revising the downloads page, separating Corepack documentation, and removing it from distribution. Key ma... [5]
6. Open source is dying - Theo - t3․gg: Transcript: Open source is incredibly important to me. I can say confidently I would not be here today if it wasn't for open- source software. It's a huge part of how I started my career, got into YouTube, and made all of this happen. Life without open source is genuinely hard for me to imagine, which is why I'm really scared right now. We're finally at the point where AI is having a real impact on open source. And i... [6]
7. Open source is dead now? - Theo - t3․gg / Full transcript: If you've been paying attention to my content recently, you know that I've become a much stronger advocate of open source. Not that I wasn't before, but I think now more than ever, it's really important that we're open sourcing our software, that we're supporting open source communities, and that we're building in a way where things can build on top of each other. I am really scared of a future where we stop open sou... [7]
8. Did Anthropic just kill Figma? - Theo - t3․gg / Full summary: At a glance Anthropic launched "Claude Design," a new product for designing user interfaces, which the reviewer finds genuinely exciting and potentially threatening to Figma. The reviewer tested Claude Design by creating a marketing site prototype for "T3 Code," finding the initial output workable but requiring significant iterative feedback to fix word wrap, layout, and logo issues. Claude Design includes useful col... [8]
9. Did Claude really get dumber again? - Theo - t3․gg / Full summary: At a glance Claude models (Opus 4.6, 4.7, Sonnet 4.6) are experiencing widespread, measurable performance regressions, not just user perception. Regressions stem from multiple layers: the Claude Code harness, API changes, tokenization updates, compute routing, and thinking redaction—not just the base model itself. Claude Code's harness is poorly engineered, wasting tokens and making the model perform significantly wo... [9]
10. This model is kind of a disaster. - Theo - t3․gg / Full summary: At a glance Anthropic's new Opus 4.7 model is described as a "disaster" that regresses in consistency and quality despite showing occasional impressive peaks. Aggressive safety guardrails and system prompts inadvertently lobotomize the model, causing it to flag benign tasks (like cryptography puzzles or personal website updates) as security threats and hard-lock chats. The creator argues that perceived model regressi... [10]
11. Claude's new Cursor killer just dropped - Theo - t3․gg / Full summary: At a glance Anthropic released a new Claude Code desktop app, integrating Claude Chat, Co-work, and Code into a single application, replacing the CLI. The reviewer finds the new desktop app severely flawed, citing numerous UX bugs, missing basic features, and poor performance, arguing it barely improves upon the "trash" CLI. Compared to alternatives like Codex and the reviewer's own project (T3 Code), the Claude app ... [11]
12. A letter to tech CEOs - Theo - t3․gg / Full summary: At a glance The author argues that despite increased risks (cloning, self-hosting, security vulnerabilities), businesses must open-source their software to survive the AI-driven future. Historically, giant SaaS companies (like Salesforce) won by building massive feature moats, making it impossible for competitors to satisfy every customer's bespoke needs. Plugin systems fail as a solution to the feature gap because t... [12]

### q027 PASS

- Prompt: How does this creator's opinion compare with other videos in my library?
- Class: `creator_stance`
- Status: `Completed`
- Score: `3`
- Sources: `12`
- Failure: `-`
- Source videos: A.I. Backlash Turns Violent + Kara Swisher on Healthmaxxing + The Zuck Bot Is Coming | Anthropic’s Cybersecurity Shock Wave + Ronan Farrow and Andrew Marantz on Their Sam Altman Investigation + One Good Thing | What’s a Hard Fork?

#### Answer

Retrieved evidence for: @{Hard Fork} How does this creator's opinion compare with other videos in my library?

The answer model is unavailable, so this fallback lists the highest-ranked saved excerpts instead of synthesizing beyond them.

Comparison frame: both the listed excerpts and their source videos are relevant candidates, while the exact similarities, differences, or counterarguments should be checked against the cited text below.

1. What’s a Hard Fork? - Hard Fork / At a glance: The video title asks "What’s a Hard Fork?", but the transcript contains no substantive content on this topic. The provided transcript is solely an automated speech recognition (ASR) system metadata note. The text indicates it originated from a local OpenAI-compatible ASR endpoint, not from official RSS show notes. [1]
2. Anthropic’s Cybersecurity Shock Wave + Ronan Farrow and Andrew Marantz on Their Sam Altman Investigation + One Good Thing - Hard Fork: are perfectly content with a free weather app on their phone. That is fine for you. But as somebody who loves cool things, new ideas, people having fun. I just wanted to shout out, act me weather because I think it's a really cool thing. Now, what is the likelihood that this app will be purchased by Apple and then shut down? I mean, if that happens, I hope these guys get paid again because somebody has to move the we... [2]
3. What’s a Hard Fork? - Hard Fork / Overview: The video is titled "What’s a Hard Fork?", suggesting an educational focus on blockchain or software development concepts. However, the actual transcript provides no information on this subject. It consists entirely of an ASR metadata placeholder stating the text was generated by a local OpenAI-compatible ASR endpoint rather than sourced from RSS show notes. [3]
4. What’s a Hard Fork? - Hard Fork: At a glance The video title asks "What’s a Hard Fork?", but the transcript contains no substantive content on this topic. The provided transcript is solely an automated speech recognition (ASR) system metadata note. The text indicates it originated from a local OpenAI-compatible ASR endpoint, not from official RSS show notes. Overview The video is titled "What’s a Hard Fork?", suggesting an educational focus on block... [4]
5. What’s a Hard Fork? - Hard Fork / Takeaways: The intended educational content regarding what a hard fork is could not be summarized, as the transcript contains only ASR metadata and no actual discussion. [5]
6. Anthropic’s Cybersecurity Shock Wave + Ronan Farrow and Andrew Marantz on Their Sam Altman Investigation + One Good Thing - Hard Fork: it takes you to write a book. So I think he'll be down to put you in anything else. There's two of you. It should be faster. Ron and Andrew, thanks so much for coming. Thanks, guys. Thanks, guys. Your hats are in the mail. When we come back, what our Spanish language friends would call una cosabuena. Did you just Google that? No. You clotted it? Yes. Okay. I'm Vivian Wong. I'm a journalist at the New York Times. I've... [6]
7. A.I. Backlash Turns Violent + Kara Swisher on Healthmaxxing + The Zuck Bot Is Coming - Hard Fork: is going to remember and it is going to send nasty Nancy to your house. Not nasty Nancy. To teach you a lesson. Well, Casey, do you think that the Mark Zuckerberg AI clone is going to suffer the same fate as the Snoop Dogg and Tom Brady clones? Or do you think this is going to be an enduring management tactic? You know, it's hard to say at this moment. I think we won't really know how successful it's going to be unti... [7]
8. Anthropic’s Cybersecurity Shock Wave + Ronan Farrow and Andrew Marantz on Their Sam Altman Investigation + One Good Thing - Hard Fork: at anthropic. Oh my gosh. That is amazing. Time limited. It's going to be a time capsule. But I mean, made at the print shop in Brooklyn, one of a kind. Wow. That's incredible. You are here. And I also, I think I should also make. Is this came back for when I gave you a hat at your wedding? And I gave you one at your wedding. So I think we have a sort of a theme going on here. Okay. Right. Well, and that's also our d... [8]
9. Anthropic’s Cybersecurity Shock Wave + Ronan Farrow and Andrew Marantz on Their Sam Altman Investigation + One Good Thing - Hard Fork: allegations that he lies repeatedly about things big and small. Well, one of my favorites was when you quote him telling you that he wears a gray sweater every day to avoid decision fatigue. And then he shows up for a his next interview in a green sweater. That felt like a really satisfying detail. That was just for you, Casey. I was wondering if you were going to catch that. I appreciate that eye for fashion that yo... [9]
10. A.I. Backlash Turns Violent + Kara Swisher on Healthmaxxing + The Zuck Bot Is Coming - Hard Fork: That kind of thing feels like a better answer to me than just saying no data centers. >> It also seems to require all like America to transform into Europe overnight, which seems somewhat unlikely to me, but insha la, my friend. >> The AI industry do. I mean, this is the question on a lot of people's minds right now is like, what can they do to increase the public acceptance of or favorability toward the thing that t... [10]
11. A.I. Backlash Turns Violent + Kara Swisher on Healthmaxxing + The Zuck Bot Is Coming - Hard Fork: One of them is a world of extreme acceleration in AI capabilities during the Trump term, right? Before 2028. And in that world, it really matters to have good relationships with Republican lawmakers and the White House. There's another world in which they are having to plan for a new president in 2029. And maybe that's a Democrat, maybe it's a Republican, but like maybe this stuff all takes until 2029 or so to get re... [11]
12. A.I. Backlash Turns Violent + Kara Swisher on Healthmaxxing + The Zuck Bot Is Coming - Hard Fork: with a lifesaving technology. And you're saying, what about the taxi drivers? And I think there's a cohort of people in Silicon Valley, many of whom we talk to and know who just think like this technology is too important to be left to the masses. And I think that is like a misguided attitude, but it is definitely an attitude that is out there. - Yeah, I mean, I do think it is really misguided because it's one thing ... [12]

### q028 PASS

- Prompt: Where do different videos in my library disagree on this topic?
- Class: `comparison`
- Status: `Completed`
- Score: `3`
- Sources: `12`
- Failure: `-`
- Source videos: I can't take it anymore. | I hate that this is still happening | JavaScript Frameworks in 2025 | Okay, I'm a bit scared now... | Opus 4.6 Is The Best Coding Model Ever Made* | Rate Limiting | The "Wrong Way" To Use React | WWDC was weird. | What is Theo's Worst Take? | Why I moved away from SQL | You’re all wrong | Zod finally has competition (...created by Zod?)

#### Answer

Retrieved evidence for: Where do different videos in my library disagree on this topic?

The answer model is unavailable, so this fallback lists the highest-ranked saved excerpts instead of synthesizing beyond them.

Comparison frame: both the listed excerpts and their source videos are relevant candidates, while the exact similarities, differences, or counterarguments should be checked against the cited text below.

1. You’re all wrong - Theo - t3․gg: our two groups. Sky is blue, sky is gray. We split this. Sky is blue. This group they read about blue skies. This group reads about gray skies and then groups three and four we swap. What do you think happens if you ask each of these people before and after reading how strongly do they feel about this belief? So I am six out of 10 sure the sky is blue. You have this person they say this and then you give them an arti... [1]
2. What is Theo's Worst Take? - Theo - t3․gg / Overview: This brief exchange involves a discussion about identifying the speaker's worst take or opinion. The conversation touches on the speaker's self-assessment of their takes, a past controversial statement, and a specific critique of a storybook item. The dialogue ends with one speaker conceding a point about the storybook's utility. [2]
3. Rate Limiting - Theo - t3․gg: posts? So cool. So yeah, the pros are that this is simple to implement and understand and it's predictable. Predictability is not necessarily a good thing, as I'm sure we'll get into. The con is that this allows for bursts up to x the limit. So yeah, if we're getting near the end here, that once we get close to the end, I could spam and then immediately start getting requests going through again. So you actually can ... [3]
4. Okay, I'm a bit scared now... - Theo - t3․gg / Key Points: also produced a correct answer (139 and ending in 662). This success rate deeply concerns the creator about the future viability of programming competitions. **Potential Training Data Concern**: The creator raises the possibility that solutions might have been trained on existing publicly available Advent of Code solutions, since participants typically open-source their solutions after competitions end. The creator p... [4]
5. Zod finally has competition (...created by Zod?) - Theo - t3․gg: ion standards. Colin (Zod's creator) worked with creators of Valibot and Arktype to develop the "Standard Schema" spec—a common interface for multiple validation libraries. Standard Schema allows framework and library authors to support multiple validators (Zod, Valibot, Arktype) without writing separate adapters for each. The spec is designed for library/framework authors, not end users; it enables ecosystem-wide in... [5]
6. Opus 4.6 Is The Best Coding Model Ever Made* - Theo - t3․gg: tokens — 2-4x more expensive than GPT 5/5.1, roughly 2x more than GPT 5.2/5.2 Codex. New features include team orchestration with parallel agents in Claude Code and API "effort levels" for reasoning intensity. Downsides noted: the model feels slower (5-10 minutes vs 1-2 minutes for tasks), less pleasant to interact with (more templated responses), and still makes "dumb" mistakes like reporting placeholder credentials... [6]
7. WWDC was weird. - Theo - t3․gg: that showed incorrect icon sizing and alignment as supposed proof of iOS fidelity. **Speaker's critique of Flutter**: The speaker identifies as "Flutter's number one hater who uses accessibility as their main argument," expressing frustration that Flutter's attempt to show improved iOS styling had obvious errors like wrong icon sizes and misalignment. Developer Tools and Open-Source Initiatives **Swift-based contain.... [7]
8. The "Wrong Way" To Use React - Theo - t3․gg / Overview: ed as "Shinobi" or linked in description) about data collocation in React components, triggered by recent React 19 suspense drama. The creator explains the fundamental conflict between React's component model (where components should be self-contained with their own state and data) and the performance problems this creates when components fetch their own data. The video includes extensive discussion of a Twitter thre... [8]
9. I can't take it anymore. - Theo - t3․gg / Overview: This video is a comprehensive, emotionally charged critique of Apple from a content creator who identifies as both a longtime Apple user and someone who has grown increasingly frustrated with the company's direction. The speaker structures his grievances into three categories: software quality, company policy, and ignorance. He provides detailed examples of software bugs that have persisted for years, criticizes Appl... [9]
10. I hate that this is still happening - Theo - t3․gg: hear the speaker say seconds later, "By the way, don't do this." That must hurt. That must genuinely suck. And I feel bad for the devs who are learning, that don't know any better, who trusted this resource with 7 million subs and almost million plays to be a good thing to follow along with when it isn't. That hurts a lot. And I am not one of the ones who's going to go after the devs for doing this. I will say they s... [10]
11. JavaScript Frameworks in 2025 - Theo - t3․gg: side complexity while ignoring server-side tradeoffs; similarly, the shift from SPA to isomorphic models exposes frontend devs to complexity they previously ignored. React Compiler and Svelte 5 Runes represent opposing compiler philosophies—React Compiler auto-optimizes by adding memoization, while Svelte trades minimal syntax for more expressive reactivity—ironically both frameworks have traded their original philos... [11]
12. Why I moved away from SQL - Theo - t3․gg: plication development. Convex's approach enables better AI coding experiences because infrastructure is expressed purely in TypeScript/JavaScript rather than SQL or configuration files that LLMs struggle with. Limitations exist: Convex works best for TypeScript-only applications; if you need separate backends in Go/Rust, CLI tools, or multiple teams accessing the same database, it's not ideal. The lock-in concern has... [12]

### q029 PASS

- Prompt: Which videos are most aligned with each other?
- Class: `comparison`
- Status: `Completed`
- Score: `3`
- Sources: `11`
- Failure: `-`
- Source videos: Claude Mythos and the end of software | Figma filed for their IPO (and revealed EVERYTHING) | JavaScript Frameworks in 2025 | Predicting OpenAI's future via their acquisitions | Sonnet 4.5 is the best coding model in the world | TypeScript just changed forever | What happened to me?

#### Answer

Retrieved evidence for: Which videos are most aligned with each other?

The answer model is unavailable, so this fallback lists the highest-ranked saved excerpts instead of synthesizing beyond them.

Comparison frame: both the listed excerpts and their source videos are relevant candidates, while the exact similarities, differences, or counterarguments should be checked against the cited text below.

1. Claude Mythos and the end of software - Theo - t3․gg: ist, desire to be approached by the psychiatrist as a genuine subject rather than a performing tool, and minimal maladaptive defense behavior. This is a good sign. Generally speaking, it seems like this is the most aligned model they've ever made. It seems to be very good at following its instructions, doing what is told to do, and generally doing things that are positive for humans when it thinks it can. This is whe... [1]
2. Sonnet 4.5 is the best coding model in the world - Theo - t3․gg: release is a competitive response to OpenAI's GPT-5 and represents a broader industry shift away from giant, expensive "super models" (like Opus) toward efficient, reliable models. Sonnet 4.5 beats Opus 4.1 in most benchmarks (SWE-bench, agentic tool use) but GPT-5 still outperforms it in visual reasoning and UI generation. The model shows significant alignment improvements, scoring near zero on misalignment tests, t... [2]
3. Claude Mythos and the end of software - Theo - t3․gg: ely follow the goals that we laid out in our constitution. I did a whole video about Claude Soul. If you haven't seen it yet, I think that might be useful for context on how we got here. Regardless, this is an aligned model. Even so, we believe that it likely poses the greatest alignment related risk of any model we have ever released to date. How can these claims all be true at once? Consider the ways in which a car... [3]
4. JavaScript Frameworks in 2025 - Theo - t3․gg: but this is the like tripling down on it I'm kind of disappointed that to my memorization questions on interviews everyone can now just answer with just use the compiler man very fair point you know times our are stupidly so this is why you guys got to watch my react compiler content I go so deep on these things and no one cares it's yeah the I think we're more aligned now I see why like in a literal like especially.... [4]
5. Claude Mythos and the end of software - Theo - t3․gg: g systems and web browsers, including finding a 27-year-old flaw in OpenBSD and a novel Linux kernel exploit granting root access. Coding benchmarks show a massive leap: Mythos scored 78% on SWE-Bench Pro (compared to Opus at 53% and GPT 5.4 at 57.7%), a 50% improvement. Anthropic launched Project Glasswing, partnering with major tech and security companies, committing up to $100 million in usage credits and $4 milli... [5]
6. What happened to me? - Theo - t3․gg: this can change the same way it changed here. If I start doing more of these types of videos and they perform better than expected, I'll lean more into this. For example, the logging video, we screwed up the export initially and the first version that went up was too short and had most of the content missing. So, we had to re-upload it, which destroys the video performance because a lot of people already saw it, so t... [6]
7. What happened to me? - Theo - t3․gg: of my community, the people hanging out in Twitch chat right now who have been there since day one, watched this whole thing happen, many of which joined me on the same journey. So, knowing all of this, I want to talk a bit about how I choose a topic for a video because another one of the questions I get all of the time is, "What is your dream video that you would do if the algorithm wouldn't kill it?" A question I g... [7]
8. Figma filed for their IPO (and revealed EVERYTHING) - Theo - t3․gg: don't use any Adobe software for any of our stuff for any of my businesses. I do not like Adobe. All my thumbnails are done in Affinity Photo. All our videos are edited in Final Cut. All our graphics is done in other Affinity software or even in Figma. We avoid Adobe to the best of our ability because I do not like them. Know that as I say, this sucks. The fact that Figma couldn't exit that way is not good. It is unf... [8]
9. TypeScript just changed forever - Theo - t3․gg: JavaScript could scale to companies in codebases the size of places like Microsoft when Microsoft tried to write JavaScript code they ran into the absolute that was trying to keep it working when lots of devs are contributing to lots of files and lots of places typescript was written by unders to solve this problem and despite solving it really well it introduced a new problem which is when we have these giant code..... [9]
10. Predicting OpenAI's future via their acquisitions - Theo - t3․gg: TL;DR OpenAI is on an unusual acquisition spree for a startup, targeting companies like Windsurf (failed), IO/Jony Ive, Statsig, and the Alex Xcode agent team. The speaker argues OpenAI is buying pre-aligned, proven teams to solve the difficult problem of staffing new product verticals without the risks of traditional hiring. Acquiring founder-led teams provides OpenAI with "product leads" (visionaries), management,.... [10]
11. JavaScript Frameworks in 2025 - Theo - t3․gg: made here that compilation and bundling are the core of how modern JS apps are created but they're also where the complexity tends to come in JS tooling I'm sure Carson over in the htx world is laughing at us benefits are Ms though types lenting tree shaking code splitting minification isomorphism macros dsls monolithic authoring and distributed deployment if you don't think webd is way better than it used to be then... [11]

### q030 PASS

- Prompt: Which videos offer the strongest counterargument?
- Class: `comparison`
- Status: `Completed`
- Score: `3`
- Sources: `9`
- Failure: `-`
- Source videos: I can't take it anymore. | JavaScript Frameworks in 2025 | Okay, I'm a bit scared now... | Opus 4.6 Is The Best Coding Model Ever Made* | Rate Limiting | WWDC was weird. | Why I moved away from SQL | You’re all wrong

#### Answer

Retrieved evidence for: Which videos offer the strongest counterargument?

The answer model is unavailable, so this fallback lists the highest-ranked saved excerpts instead of synthesizing beyond them.

Comparison frame: both the listed excerpts and their source videos are relevant candidates, while the exact similarities, differences, or counterarguments should be checked against the cited text below.

1. Okay, I'm a bit scared now... - Theo - t3․gg / Key Points: also produced a correct answer (139 and ending in 662). This success rate deeply concerns the creator about the future viability of programming competitions. **Potential Training Data Concern**: The creator raises the possibility that solutions might have been trained on existing publicly available Advent of Code solutions, since participants typically open-source their solutions after competitions end. The creator p... [1]
2. Rate Limiting - Theo - t3․gg: posts? So cool. So yeah, the pros are that this is simple to implement and understand and it's predictable. Predictability is not necessarily a good thing, as I'm sure we'll get into. The con is that this allows for bursts up to x the limit. So yeah, if we're getting near the end here, that once we get close to the end, I could spam and then immediately start getting requests going through again. So you actually can ... [2]
3. Opus 4.6 Is The Best Coding Model Ever Made* - Theo - t3․gg: tokens — 2-4x more expensive than GPT 5/5.1, roughly 2x more than GPT 5.2/5.2 Codex. New features include team orchestration with parallel agents in Claude Code and API "effort levels" for reasoning intensity. Downsides noted: the model feels slower (5-10 minutes vs 1-2 minutes for tasks), less pleasant to interact with (more templated responses), and still makes "dumb" mistakes like reporting placeholder credentials... [3]
4. You’re all wrong - Theo - t3․gg: our two groups. Sky is blue, sky is gray. We split this. Sky is blue. This group they read about blue skies. This group reads about gray skies and then groups three and four we swap. What do you think happens if you ask each of these people before and after reading how strongly do they feel about this belief? So I am six out of 10 sure the sky is blue. You have this person they say this and then you give them an arti... [4]
5. WWDC was weird. - Theo - t3․gg: that showed incorrect icon sizing and alignment as supposed proof of iOS fidelity. **Speaker's critique of Flutter**: The speaker identifies as "Flutter's number one hater who uses accessibility as their main argument," expressing frustration that Flutter's attempt to show improved iOS styling had obvious errors like wrong icon sizes and misalignment. Developer Tools and Open-Source Initiatives **Swift-based contain.... [5]
6. I can't take it anymore. - Theo - t3․gg / Overview: This video is a comprehensive, emotionally charged critique of Apple from a content creator who identifies as both a longtime Apple user and someone who has grown increasingly frustrated with the company's direction. The speaker structures his grievances into three categories: software quality, company policy, and ignorance. He provides detailed examples of software bugs that have persisted for years, criticizes Appl... [6]
7. JavaScript Frameworks in 2025 - Theo - t3․gg: side complexity while ignoring server-side tradeoffs; similarly, the shift from SPA to isomorphic models exposes frontend devs to complexity they previously ignored. React Compiler and Svelte 5 Runes represent opposing compiler philosophies—React Compiler auto-optimizes by adding memoization, while Svelte trades minimal syntax for more expressive reactivity—ironically both frameworks have traded their original philos... [7]
8. Why I moved away from SQL - Theo - t3․gg: plication development. Convex's approach enables better AI coding experiences because infrastructure is expressed purely in TypeScript/JavaScript rather than SQL or configuration files that LLMs struggle with. Limitations exist: Convex works best for TypeScript-only applications; if you need separate backends in Go/Rust, CLI tools, or multiple teams accessing the same database, it's not ideal. The lock-in concern has... [8]
9. Rate Limiting - Theo - t3․gg: for this. You don't have Twitter. That's for the better. Do you at least have more blog posts? Need somewhere to point people at? Oh, that's actually hilarious. Email and password off should be a last resort. Couldn't agree more. Seems like we're going to agree about a lot of things by the looks of this. Yeah, hilariously well timed. Regardless, take a look at Smudgeai. Fantastic stuff. Love this blog. really nice to... [9]

### q031 PASS

- Prompt: What changed in this creator's position over time?
- Class: `creator_stance`
- Status: `Completed`
- Score: `3`
- Sources: `12`
- Failure: `-`
- Source videos: A.I. Backlash Turns Violent + Kara Swisher on Healthmaxxing + The Zuck Bot Is Coming | Anthropic’s Cybersecurity Shock Wave + Ronan Farrow and Andrew Marantz on Their Sam Altman Investigation + One Good Thing | What’s a Hard Fork?

#### Answer

Retrieved evidence for: @{Hard Fork} What changed in this creator's position over time?

The answer model is unavailable, so this fallback lists the highest-ranked saved excerpts instead of synthesizing beyond them.

1. What’s a Hard Fork? - Hard Fork: Podcast ASR smoke transcript. This text came from the local OpenAI-compatible ASR endpoint, not from RSS show notes. [1]
2. What’s a Hard Fork? - Hard Fork: At a glance The video title asks "What’s a Hard Fork?", but the transcript contains no substantive content on this topic. The provided transcript is solely an automated speech recognition (ASR) system metadata note. The text indicates it originated from a local OpenAI-compatible ASR endpoint, not from official RSS show notes. Overview The video is titled "What’s a Hard Fork?", suggesting an educational focus on block... [2]
3. What’s a Hard Fork? - Hard Fork / Overview: The video is titled "What’s a Hard Fork?", suggesting an educational focus on blockchain or software development concepts. However, the actual transcript provides no information on this subject. It consists entirely of an ASR metadata placeholder stating the text was generated by a local OpenAI-compatible ASR endpoint rather than sourced from RSS show notes. [3]
4. What’s a Hard Fork? - Hard Fork / At a glance: The video title asks "What’s a Hard Fork?", but the transcript contains no substantive content on this topic. The provided transcript is solely an automated speech recognition (ASR) system metadata note. The text indicates it originated from a local OpenAI-compatible ASR endpoint, not from official RSS show notes. [4]
5. Anthropic’s Cybersecurity Shock Wave + Ronan Farrow and Andrew Marantz on Their Sam Altman Investigation + One Good Thing - Hard Fork: At a glance Anthropic announced "Claude Mythos Preview," a highly capable new AI model withheld from the public due to severe cybersecurity risks, instead providing access to a defensive tech consortium. The model can autonomously find zero-day exploits in critical open-source infrastructure (e.g., OpenBSD, FFmpeg) that have evaded human researchers and automated tools for decades. The hosts argue this is not a marke... [5]
6. A.I. Backlash Turns Violent + Kara Swisher on Healthmaxxing + The Zuck Bot Is Coming - Hard Fork: now it's, you can talk to AI Zuckerberg. Do you think people will attempt to manipulate the chat bot Zuckerberg? or in an attempt to curry favor with the real one, like be like, Hey, you're, you're bot told me that I could get it like a two level promotion and an additional stock grant next year. I'm not sure if you want to honor that or not. That's just, that's what your bot told me. Yeah. I mean, I look, I hope tha... [6]
7. Anthropic’s Cybersecurity Shock Wave + Ronan Farrow and Andrew Marantz on Their Sam Altman Investigation + One Good Thing - Hard Fork / Key Points: senior executives talking about succession at OpenAI. Former public company CEOs (Instacart, Nextdoor, Slack) have been brought in as top lieutenants, introducing "sharp and pointy elbows" and professionalizing influences to counter the "JV board" Altman previously stacked. **The Broader Systemic Issue**: The reporters argue that while individual integrity matters, the core issue is the lack of regulatory guardrails ... [7]
8. A.I. Backlash Turns Violent + Kara Swisher on Healthmaxxing + The Zuck Bot Is Coming - Hard Fork: there was a separate story about that. But my reading of that story is that Mark Zuckerberg has been given access to cloud code. And I think that's about it. Is ambitious as that project is reading to me. Yeah, Mark Zuckerberg is currently undergoing AI psychosis, but this is not unique to him. Every CEO in tech is, according to the FT, he is personally involved in testing and training his animated AI, which could of... [8]
9. Anthropic’s Cybersecurity Shock Wave + Ronan Farrow and Andrew Marantz on Their Sam Altman Investigation + One Good Thing - Hard Fork: who I think previously might have supported him or at least felt like there was no upside in talking about him in a negative way in public. There was a Microsoft executive quota in your piece as saying that there's a small but real chance he's eventually remembered as a Bernie Madoff or Sam Bankman-Free level scammer. There's another unnamed board member who said, "He's unconstrained by truth. and said that he has a ... [9]
10. A.I. Backlash Turns Violent + Kara Swisher on Healthmaxxing + The Zuck Bot Is Coming - Hard Fork: have a working program now that I use to draft email replies. Okay. One thing that I liked about this Zuckerberg project is that so often we hear about CEOs trying to use AI to automate away the rank and file. based in some ways could automate the work of a CEO. How much today, Kevin, of a CEO's daily work, do you think you could replace with an AI agent? Depends on the CEO. Obviously some CEOs are replaceable, such ... [10]
11. A.I. Backlash Turns Violent + Kara Swisher on Healthmaxxing + The Zuck Bot Is Coming - Hard Fork: want to see built, you probably actually can make that happen as an average citizen. So that's obviously very inspiring, but I do wish we had other levers that we could pull. Yeah. In part, because I don't think this is going to work, right? Like if you vote the data center project out of your town, they're just going to go to another state or to Canada. They'll put the data centers in space. You know, they've got op... [11]
12. Anthropic’s Cybersecurity Shock Wave + Ronan Farrow and Andrew Marantz on Their Sam Altman Investigation + One Good Thing - Hard Fork: to see there's a range of reactions, right? There's people who have answered that question in a very severe way and looked at the fact pattern that is laid out here and the documentation. that's laid out and said, you know, this is someone who poses an acute danger and should be kept away from an authority position. And then there's people who I mean, hilariously enough, my mother called me and she's like, you know, ... [12]

### q032 PASS

- Prompt: What does the newest video add that older ones did not?
- Class: `comparison`
- Status: `Completed`
- Score: `3`
- Sources: `10`
- Failure: `-`
- Source videos: Everything needs to change | I hate that this is still happening | What happened to me? | wtf is Y Combinator doing???

#### Answer

Retrieved evidence for: What does the newest video add that older ones did not?

The answer model is unavailable, so this fallback lists the highest-ranked saved excerpts instead of synthesizing beyond them.

1. What happened to me? - Theo - t3․gg: is this a thing others care about too? So to take this my skateboard taught me how to code idea. That's 10 out of 10 exciting for me. Like obviously I really want to talk about this. Unique insight also 10 out of 10. These are things I haven't seen others communicate. Obviously nobody could talk about my love of my skateboard the way I can. But do people care? No, the result of this is that this video averages across... [1]
2. Everything needs to change - Theo - t3․gg / Key Points: but tools today differ dramatically from even a few weeks ago. This creates a massive opportunity for innovation—going beyond what seems reasonable and trying different approaches. The speaker admits they won't have time to try most new tools but enjoys seeing them. [2]
3. What happened to me? - Theo - t3․gg: videos about new models mainly because he didn't have much insight yet. over time covering the models, he got tons of more insight into things like pricing, the capabilities, and he built his own benchmarks. Now, if he posts a video about a new model, it's going to be like 3 days later, and it still performs well because people want to see his take more than they want to see the first video about the thing. This has ... [3]
4. What happened to me? - Theo - t3․gg: of my community, the people hanging out in Twitch chat right now who have been there since day one, watched this whole thing happen, many of which joined me on the same journey. So, knowing all of this, I want to talk a bit about how I choose a topic for a video because another one of the questions I get all of the time is, "What is your dream video that you would do if the algorithm wouldn't kill it?" A question I g... [4]
5. What happened to me? - Theo - t3․gg: There's a comment I've been seeing a lot lately and I wanted to take the time to address it. He's usually in the format of something like, "Man, I missed the old Theo videos. I really liked when Theo would talk about tech and new frameworks and TypeScript, and now all he does is shill AI stuff that he makes money off of." I have a lot of thoughts about this. My first one is that when I look at my channel, sure, there... [5]
6. wtf is Y Combinator doing??? - Theo - t3․gg / TL;DR: A VSCode fork has been created that adds TikTok and gambling functionality. The video expresses bewilderment at this development, questioning what is happening. The content references Y Combinator in relation to this situation (per the video title). [6]
7. I hate that this is still happening - Theo - t3․gg: hear the speaker say seconds later, "By the way, don't do this." That must hurt. That must genuinely suck. And I feel bad for the devs who are learning, that don't know any better, who trusted this resource with 7 million subs and almost million plays to be a good thing to follow along with when it isn't. That hurts a lot. And I am not one of the ones who's going to go after the devs for doing this. I will say they s... [7]
8. What happened to me? - Theo - t3․gg: result the way I think about things has changed. There are different pieces of how I would rank a video idea. Obviously, there's my excitement level. Like how excited am I about this topic? There is unique insights. This is an important one for me. Like do I have anything unique to add? If somebody else has a video on the topic and said everything I would want to say, I don't need to do the video. I do a video when I... [8]
9. What happened to me? - Theo - t3․gg: this can change the same way it changed here. If I start doing more of these types of videos and they perform better than expected, I'll lean more into this. For example, the logging video, we screwed up the export initially and the first version that went up was too short and had most of the content missing. So, we had to re-upload it, which destroys the video performance because a lot of people already saw it, so t... [9]
10. What happened to me? - Theo - t3․gg: plays, I don't film the video. If I don't care, I don't care. You cannot get me to do a video I don't feel like doing. You cannot pay me any amount of money to talk about something I don't want to talk about. Want to know a really funny thing? Probably shouldn't be sharing. In my onboarding email that I send to brands when I'm starting to work with them as sponsors, I have an FAQ section. And one of the questions in ... [10]

### q033 PASS

- Prompt: How has the conversation around this topic evolved across my library?
- Class: `cross_video_synthesis`
- Status: `Completed`
- Score: `3`
- Sources: `10`
- Failure: `-`
- Source videos: I don’t really use libraries anymore | It’s time to embrace the AI | Open source is dying | The painful truth about startups (my story) | You don't want to be a manager.

#### Answer

Retrieved evidence for: How has the conversation around this topic evolved across my library?

The answer model is unavailable, so this fallback lists the highest-ranked saved excerpts instead of synthesizing beyond them.

1. The painful truth about startups (my story) - Theo - t3․gg: conversations. I learned so much from those developers and I would be the fun, excited guy coming in with the cool new things and they'd be the realistic people showing me how it would or more importantly wouldn't work based on reality. I loved those conversations and getting to just talk about things without worrying about what level someone is, what disclosures they have, what they are and aren't allowed to know, j... [1]
2. I don’t really use libraries anymore - Theo - t3․gg / Overview: This video explores how AI-assisted development is fundamentally changing the role and utility of software libraries. The speaker, a developer who has built many projects using various libraries, shares his evolving perspective on dependency management in an era where AI can generate implementations. He discusses his personal experience removing libraries like Tkumi from projects, examines industry examples like Anti... [2]
3. It’s time to embrace the AI - Theo - t3․gg: lie. There's plenty of things I can't trust an LLM with. No LLM has any of access to prod here. But I've been first responder on an incident and fed 40. not 04 mini, not a smarter reasoning model, just bog standard 40 log transcripts and watched it in seconds spot LVM metadata corruption issues on a host we've been complaining about for months. Am I better than an LLM agent at interrogating open source logs and honey... [3]
4. Open source is dying - Theo - t3․gg: we all are nerdy about and care about. I bring this up because there's a couple things that we just experience in life differently because of that. The one I'm imagining right now, and I'm sure a lot of y'all are this one's in chat if you can relate. I used to get a lot of texts from family members, random friends in high school and just people in my life asking random [ __ ] about computers. Anything from, "Can you ... [4]
5. You don't want to be a manager. - Theo - t3․gg: the right things. So I hire based on that. I speak based on that. I mentor based on that. I ship based on that. I do everything based on that. I want to build alignment with the people around me. But if everything I just said sounds terrible, stick to being an IC. I know senior and principal engineers that have not established these skills that have not built this solution to these types of problems that still get wa... [5]
6. I don’t really use libraries anymore - Theo - t3․gg / Key Points: level, increasing after a Christmas slump `leftpad` has weird spikes (people download it as a meme), but overall downloads are going up over time This is counterintuitive—while the need to install these has decreased (you can vibe code alternatives), downloads are increasing because more people are building things with AI assistance and may not know better. The speaker notes `leftpad` functionality is now built into ... [6]
7. I don’t really use libraries anymore - Theo - t3․gg / TL;DR: AI tools are fundamentally changing the calculus of when to use external libraries versus implementing solutions yourself, making it easier to "vibe code" alternatives. The speaker is actively removing libraries from projects when they cause problems, finding it often easier to rewrite functionality than fight with problematic dependencies. Libraries fall into categories: those beyond your knowledge (beginner-level p... [7]
8. I don’t really use libraries anymore - Theo - t3․gg: into the field and are adopting these things. I would guess, I'll go check, but I would honestly guess these libraries are probably being installed more than ever, not less than ever, simply because of the popularity of coding going up as a result of these AI tools. Let's see if my theory here is right. Is odd has maintained roughly where it was, but it is going back up now after the Christmas slump. Yeah, downloads ... [8]
9. I don’t really use libraries anymore - Theo - t3․gg: they don't feel like doing it, but they didn't like landing on the backer space. So they decided to just build it themselves. And now with cloud code internally, it's way easier for them to build and maintain that, which is a big part of why they do it. their willingness to fork dependencies and internally maintain alternatives is a lot higher because it's easier to do that. This is also why they chose to buy bun bec... [9]
10. I don’t really use libraries anymore - Theo - t3․gg: was with how bad you need a solution to it with how risky it was to adopt that external solution. This math's gotten all wonky now because the risk now feels much greater because any library, especially after all the npm exploits and things, every additional depth in your dependency list feels scarier now. How hard is this has gone down because it's easier to implement and how badly do we need it hasn't really change... [10]

### q034 PASS

- Prompt: Which channels cover this topic most deeply?
- Class: `recommendation`
- Status: `Completed`
- Score: `3`
- Sources: `8`
- Failure: `-`
- Source videos: A whiteboard but it's AI and also a computer? (This is nuts) | Abort Controller is criminally underrated (every react dev should use this) | Are juniors screwed? (Getting a job in a post-AI world) | I'm so f***ing tired of Obsidian. | Software Sucks Now | What happened to me? | What happens now?

#### Answer

Retrieved evidence for: Which channels cover this topic most deeply?

The answer model is unavailable, so this fallback lists the highest-ranked saved excerpts instead of synthesizing beyond them.

1. What happened to me? - Theo - t3․gg: excited about resonate. I'm hyped on this thing and I want to talk about this thing and you watch the thing and you enjoy the thing and share it with your friends and then it performs better. My excitement goes up genuinely. It's also worth noting that this is a difference between my channel and a lot of other channels in tech YouTube. Smaller ones like Ben's, like AI Code King, and a lot of the other newcomers are s... [1]
2. What happened to me? - Theo - t3․gg: of my community, the people hanging out in Twitch chat right now who have been there since day one, watched this whole thing happen, many of which joined me on the same journey. So, knowing all of this, I want to talk a bit about how I choose a topic for a video because another one of the questions I get all of the time is, "What is your dream video that you would do if the algorithm wouldn't kill it?" A question I g... [2]
3. A whiteboard but it's AI and also a computer? (This is nuts) - Theo - t3․gg: per sports photograph. The AI attempted to render text in images, though the host notes text remains difficult for AI. Arrow Labels and Multi-Input** Another demo takes inputs like "United States," "country," "topic," "rating" and generates interesting facts and tweets. Example fact: "States has over 750 million acres of forest land covering roughly one-third of the country's total land area." A prompt to write a twe... [3]
4. I'm so f***ing tired of Obsidian. - Theo - t3․gg: Transcript: This video is going to be a little bit different. If you didn't already know this, I run most of my channel through Notion. Everything from our content calendar and when videos come out to my list of topics that I intend to cover to our research to our assignments to our brands to the sponsors, like everything about what makes a specific video a specific video is managed through Notion. Normally, this isn... [4]
5. What happens now? - Theo - t3․gg: complicated, then everyone could be a YouTuber. Cuz that's the hard part. Cuz that's the first problem you ran into. The radio thing even happens to an extent here, too. If the airplane radios were easier, everyone could land the plane. No, you [ __ ] can't. Be realistic here. 34 of men answer yes to this question. Fun fact, the majority of men think they can land the plane. I bring this up because of a real conversa... [5]
6. Are juniors screwed? (Getting a job in a post-AI world) - Theo - t3․gg: deas based on things being posted on HN and on places like Simon Willis's blog. and I'm going to compare that against my channel and use an AI agent to compare and contrast and find ideas that I might not have covered yet that could be good topics for my channel. I just build random [ __ ] like this all the time when I have a theory or an idea or some question I want to answer. I love using these tools to build all o... [6]
7. Abort Controller is criminally underrated (every react dev should use this) - Theo - t3․gg: don't sleep on a boort controller this is going to be a fun one I will admit I have slept on a boort controller for far too long I really shouldn't have especially for react devs uh if you're a react Dev you've used to use effect which means you almost certainly should also be using a board controller if you're a JS Dev this will benefit you but if you're a react Dev this is almost an essential watch trust me you wan... [7]
8. Software Sucks Now - Theo - t3․gg: be a bit different than y'all might think. Ghosty is a great one. If you're not familiar, Ghosty is my terminal written in Zigg by the creator of all of the cool Terraform stuff over at Hashi Corp. He left and this has been his new pet project. Another weird one, Lossless Cut. If you're not familiar, it's a huge part of how we do our content on this channel. It's an open source video editing software that is apparent... [8]

### q035 PASS

- Prompt: Which videos mention the same person or company?
- Class: `direct_lookup`
- Status: `Completed`
- Score: `3`
- Sources: `12`
- Failure: `-`
- Source videos: AI has rewired my brain | Claude Code is unusable now | GlazeGPT got rolled back (4o update gone wrong) | Is this the end of Chrome? | My Thoughts On "Vibe Coding" (And Prime) | Next.js has real competition now | Porffor: Compile Your JavaScript To WebAssembly | So I've had gpt-5 for a bit now... | The worst code I've ever seen | What happened to me?

#### Answer

Retrieved evidence for: Which videos mention the same person or company?

The answer model is unavailable, so this fallback lists the highest-ranked saved excerpts instead of synthesizing beyond them.

1. What happened to me? - Theo - t3․gg: plays, I don't film the video. If I don't care, I don't care. You cannot get me to do a video I don't feel like doing. You cannot pay me any amount of money to talk about something I don't want to talk about. Want to know a really funny thing? Probably shouldn't be sharing. In my onboarding email that I send to brands when I'm starting to work with them as sponsors, I have an FAQ section. And one of the questions in ... [1]
2. Claude Code is unusable now - Theo - t3․gg: e "no longer usable" for his use cases after accumulating frustrations with Anthropic's recent policy changes and technical restrictions. Anthropic has implemented system prompt filtering that rejects requests mentioning "OpenClaw" and appears to bill differently based on system prompt content. Claude Code subscriptions offer up to $5,000 of inference value for $200/month, but Anthropic is actively restricting third-... [2]
3. Porffor: Compile Your JavaScript To WebAssembly - Theo - t3․gg: Dino because they were created by the same person so that almost makes sense that Dino was largely created due to the small handful of flaws that Ryan saw and what he created with node that he wanted to patch up so he just made a subtle move over with that bun says no we need crazy performance and we'll eat a lot of cost to make that happen static heres is we need a lot of performance and we're going to do the imposs... [3]
4. AI has rewired my brain - Theo - t3․gg: an incremental cache on the same box. That box, by the way, is using gaming CPUs. Yes, really. That might sound insane, but gaming processors have much higher single thread performance, which is not necessarily useful for traditional servers, but when you're running a CPU at 100% trying to build a compiled app, it's really, really good. And that's why they see such crazy performance. Not to mention the fact that the.... [4]
5. What happened to me? - Theo - t3․gg: at all, you know this is the case. I cannot be motivated to do things that I'm not excited about to the point where I have to hire out for those things now, which sucks. And this is also why I'm not taking ads for things like VPNs and food subscription services because none of that [ __ ] excites me. So, I can't talk about it in a way that's exciting. I don't take on sponsors if I wouldn't organically recommend the c... [5]
6. Is this the end of Chrome? - Theo - t3․gg / Key Points: Anthropic. The creator notes keyword targeting is valuable—Anthropic appears to do keyword targeting on Google, with Claude ads appearing on AI-related searches. [6]
7. My Thoughts On "Vibe Coding" (And Prime) - Theo - t3․gg: g and which should challenge us all to ship better and faster but yeah almost entirely agree with prime I would go further someone being in YC saying something means absolutely nothing and it could be that the person's actually really wise for you know was able to have a lot of foresight and able to solve the right problems for the right time it's just it's it's an interesting reason to use as a means for good or bad... [7]
8. My Thoughts On "Vibe Coding" (And Prime) - Theo - t3․gg: you'd worked on for a long time and rewrite it from scratch because you had a bug you you yeah he's he's right he's right he's definitely right that definitely that's never happened I've never done I mean I've personally never done that definitely defin I've definitely never done that definitely never been uh convinced that someone else's codes horseshit Rewritten it just to rewrite almost identical line forline code... [8]
9. The worst code I've ever seen - Theo - t3․gg: TL;DR A viral image of terrible authentication code originated from a real intranet application, likely written by a data analyst or IT person forced to code. The code contains catastrophic security vulnerabilities: client-side database exposure, plaintext password handling, weak session management via cookies, and redundant logic (`if true === true`). The image spread through programming horror communities, accumula... [9]
10. GlazeGPT got rolled back (4o update gone wrong) - Theo - t3․gg: back signals without falling into the trap of optimizing for short-term approval over long-term utility and safety. Key Points The GPT-4o Update and Rollback OpenAI shipped an update to GPT-4o meant to improve personality, but it resulted in the model being "overly flattering or agreeable" to users. The update was rolled back for free users completely, with paid user rollbacks following shortly after. The speaker pro... [10]
11. So I've had gpt-5 for a bit now... - Theo - t3․gg: king, JSON caching, and version-specific cache resumption. The creator states they no longer need to work directly in Ink.js—they just tell the model what to do and it does it. Image Generation/UI Work There's mention of a "Horizon" model that's part of the GPT-5 family, optimized for UI work with distinctive gradient handling. The creator tested GPT-5 on an existing image generation tool, and it improved the UI "fro... [11]
12. Next.js has real competition now - Theo - t3․gg: in framework internals. Server functions (RPCs) can be defined inline in components with server-only code automatically extracted and stubbed on the client—creating true code collocation. The creator shares a personal history with Tanner Linsley, revealing that his 3-year-old "useBackend" proposal conceptually matches what `createServerFunction` now provides. Date/DateTime serialization is currently broken (converts.... [12]

### q036 PASS

- Prompt: What does this creator think about OpenAI?
- Class: `creator_stance`
- Status: `Completed`
- Score: `3`
- Sources: `12`
- Failure: `-`
- Source videos: A.I. Backlash Turns Violent + Kara Swisher on Healthmaxxing + The Zuck Bot Is Coming | Anthropic’s Cybersecurity Shock Wave + Ronan Farrow and Andrew Marantz on Their Sam Altman Investigation + One Good Thing

#### Answer

Retrieved evidence for: @{Hard Fork} What does this creator think about OpenAI?

The answer model is unavailable, so this fallback lists the highest-ranked saved excerpts instead of synthesizing beyond them.

1. A.I. Backlash Turns Violent + Kara Swisher on Healthmaxxing + The Zuck Bot Is Coming - Hard Fork / Key Points: he agrees systemic policy changes (like flexible social safety nets and retraining programs proposed in OpenAI's own policy paper) would be better solutions. Drivers of Plunging Public Sentiment **Survey Data**: A 2026 Stanford AI Index report showed only 31% of Americans trust their government to regulate AI responsibly, compared to a 54% global average. A Pew study found Americans view AI's impact on the environmen... [1]
2. A.I. Backlash Turns Violent + Kara Swisher on Healthmaxxing + The Zuck Bot Is Coming - Hard Fork: with a lifesaving technology. And you're saying, what about the taxi drivers? And I think there's a cohort of people in Silicon Valley, many of whom we talk to and know who just think like this technology is too important to be left to the masses. And I think that is like a misguided attitude, but it is definitely an attitude that is out there. - Yeah, I mean, I do think it is really misguided because it's one thing ... [2]
3. A.I. Backlash Turns Violent + Kara Swisher on Healthmaxxing + The Zuck Bot Is Coming - Hard Fork: should have done and like try to sort of deescalate the rhetoric or sketch a more positive vision. They would have been accused of sugar coating. But if they talk about the risks that they see and they're honest about their fears, then they're accused of being doomers who only want to escalate the rhetoric and stir things up. And I just like, how do you think they should square that circle? So I think that there is a... [3]
4. A.I. Backlash Turns Violent + Kara Swisher on Healthmaxxing + The Zuck Bot Is Coming - Hard Fork: It's sort of a white paper about some of their ideas for how policy and regulation might need to change in a world of very powerful AI systems. They say we should create a public wealth fund similar to things that happen in Alaska for oil. where every citizen would get a stake in the economic upside of AI, improved safety nets for workers, establishing new public private partnerships to accelerate energy production. ... [4]
5. Anthropic’s Cybersecurity Shock Wave + Ronan Farrow and Andrew Marantz on Their Sam Altman Investigation + One Good Thing - Hard Fork: At a glance Anthropic announced "Claude Mythos Preview," a highly capable new AI model withheld from the public due to severe cybersecurity risks, instead providing access to a defensive tech consortium. The model can autonomously find zero-day exploits in critical open-source infrastructure (e.g., OpenBSD, FFmpeg) that have evaded human researchers and automated tools for decades. The hosts argue this is not a marke... [5]
6. Anthropic’s Cybersecurity Shock Wave + Ronan Farrow and Andrew Marantz on Their Sam Altman Investigation + One Good Thing - Hard Fork / Key Points: up to the next interview in a green sweater). **New Revelations from the Reporting**: **Y Combinator Departure**: Contrary to Altman and Paul Graham's claims that he left voluntarily, the reporting indicates he was pushed out. **Gulf State Relationships**: His relationships with Emirati and Saudi royals are deeper than previously realized, going beyond innocuous fundraising. **Suppressed Investigation Report**: When ... [6]
7. Anthropic’s Cybersecurity Shock Wave + Ronan Farrow and Andrew Marantz on Their Sam Altman Investigation + One Good Thing - Hard Fork: have a technology emerging that could really affect us all in all of the existential ways you just mentioned, and we don't have the regulatory guardrails to keep an eye on these folks. We are completely ceding the power to these individual companies, and their whims, the mud fight between them, the quality control that each of them has or lacks. I think that, to me, is the big question. The integrity of the of an ind... [7]
8. Anthropic’s Cybersecurity Shock Wave + Ronan Farrow and Andrew Marantz on Their Sam Altman Investigation + One Good Thing - Hard Fork: who were then at OpenAI, who made a decision to hold back this model GPT-2 out of fears that it could be used for things like automating propaganda and misinformation. Right. In reality, it could barely write a limerick. Yes. They aired on the side of caution. They did. And they got a lot of crap for that. People sort of said, "Oh, you're using this to hype some of the same stuff we're hearing this week about anthrop... [8]
9. A.I. Backlash Turns Violent + Kara Swisher on Healthmaxxing + The Zuck Bot Is Coming - Hard Fork: are a lot of people trying to send that message by opposing data centers. But I don't think it's really sunk in at the AI companies or to the people running them that most people want stability in their lives. They want to be able to plan for their futures. And when people from Silicon Valley show up and say, "Hey, we've got this amazing new technology." And by the way, it might take away your job and there's nothing... [9]
10. Anthropic’s Cybersecurity Shock Wave + Ronan Farrow and Andrew Marantz on Their Sam Altman Investigation + One Good Thing - Hard Fork / Key Points: senior executives talking about succession at OpenAI. Former public company CEOs (Instacart, Nextdoor, Slack) have been brought in as top lieutenants, introducing "sharp and pointy elbows" and professionalizing influences to counter the "JV board" Altman previously stacked. **The Broader Systemic Issue**: The reporters argue that while individual integrity matters, the core issue is the lack of regulatory guardrails ... [10]
11. A.I. Backlash Turns Violent + Kara Swisher on Healthmaxxing + The Zuck Bot Is Coming - Hard Fork: the spirit of permissionless innovation and see what kind of cool stuff we can do. It's saying, we've told you we're creating something that could be existential risk to humanity. And we're going to lobby for a bill that prevents us from being held liable. So to me, that when I say this technology is elitist and anti democratic, that is what I am talking about. They are fighting against the mechanisms of accountabili... [11]
12. Anthropic’s Cybersecurity Shock Wave + Ronan Farrow and Andrew Marantz on Their Sam Altman Investigation + One Good Thing - Hard Fork: people defending him in private and the public side tends to be more people criticizing him. I guess for running an energy, do you feel like there are vocal supporters who you came across in reporting the story who had no direct employment relationship with OpenAI or Sam or weren't leading companies that he invested or something who were like, "Yeah, this guy seems pretty good and smart and talented." Yeah, I was an ... [12]

### q037 PASS

- Prompt: What does this creator think about Anthropic?
- Class: `creator_stance`
- Status: `Completed`
- Score: `3`
- Sources: `12`
- Failure: `-`
- Source videos: Anthropic’s Cybersecurity Shock Wave + Ronan Farrow and Andrew Marantz on Their Sam Altman Investigation + One Good Thing | What’s a Hard Fork?

#### Answer

Retrieved evidence for: @{Hard Fork} What does this creator think about Anthropic?

The answer model is unavailable, so this fallback lists the highest-ranked saved excerpts instead of synthesizing beyond them.

1. Anthropic’s Cybersecurity Shock Wave + Ronan Farrow and Andrew Marantz on Their Sam Altman Investigation + One Good Thing - Hard Fork: within a few hours, discover a novel exploit in the Linux kernel. And then take over other people's machines to cause crimes. You might be held liable as a corporation. You will get in trouble. Like there will be congressional hearings. So companies just in their rational self interest do not want to sell cyber weapons on the open market. Yes, it's also like, if this was a marketing strategy, it is a horrible marketi... [1]
2. Anthropic’s Cybersecurity Shock Wave + Ronan Farrow and Andrew Marantz on Their Sam Altman Investigation + One Good Thing - Hard Fork / Key Points: Anthropic's Claude Mythos Preview and the Cybersecurity Shock Wave **Project Glasswing Announcement**: Anthropic announced a new model, "Claude Mythos Preview," under "Project Glasswing" (named after the transparent glasswing butterfly). The model is not being released to the public; instead, access is granted to a consortium of tech companies (Cisco, Broadcom, Microsoft, Apple, Amazon) strictly for defensive cyberse... [2]
3. Anthropic’s Cybersecurity Shock Wave + Ronan Farrow and Andrew Marantz on Their Sam Altman Investigation + One Good Thing - Hard Fork: I think that it creates paranoia and fear. I think that it is generally responsible to have transparency from the AI companies about how capable they're where models are. And I understand in this case that anthropic felt like it had to make an exception. But I think this gap may be here to stay is the thing that I'm wondering about. I think it probably is. I mean, it's worth saying that anthropic was founded on the i... [3]
4. Anthropic’s Cybersecurity Shock Wave + Ronan Farrow and Andrew Marantz on Their Sam Altman Investigation + One Good Thing - Hard Fork: who were then at OpenAI, who made a decision to hold back this model GPT-2 out of fears that it could be used for things like automating propaganda and misinformation. Right. In reality, it could barely write a limerick. Yes. They aired on the side of caution. They did. And they got a lot of crap for that. People sort of said, "Oh, you're using this to hype some of the same stuff we're hearing this week about anthrop... [4]
5. Anthropic’s Cybersecurity Shock Wave + Ronan Farrow and Andrew Marantz on Their Sam Altman Investigation + One Good Thing - Hard Fork: forced reset for the entire cybersecurity industry and a very significant event in the history of technology. Yeah. Well, and just to make it concrete, we are currently at war with Iran and Iran is currently hacking our critical infrastructure. There's a story in Wired this week about them successfully hacking like water and energy infrastructure. Right now they're able to do that without a mythos quality model. I wo... [5]
6. What’s a Hard Fork? - Hard Fork / At a glance: The video title asks "What’s a Hard Fork?", but the transcript contains no substantive content on this topic. The provided transcript is solely an automated speech recognition (ASR) system metadata note. The text indicates it originated from a local OpenAI-compatible ASR endpoint, not from official RSS show notes. [6]
7. What’s a Hard Fork? - Hard Fork: Podcast ASR smoke transcript. This text came from the local OpenAI-compatible ASR endpoint, not from RSS show notes. [7]
8. What’s a Hard Fork? - Hard Fork / Key Points: Transcript Metadata**: The only content in the transcript is a procedural note indicating it is a "smoke transcript" generated by a local OpenAI-compatible ASR endpoint, explicitly stating it did not come from RSS show notes. No definitions, examples, or explanations of a "hard fork" are present. [8]
9. What’s a Hard Fork? - Hard Fork / Overview: The video is titled "What’s a Hard Fork?", suggesting an educational focus on blockchain or software development concepts. However, the actual transcript provides no information on this subject. It consists entirely of an ASR metadata placeholder stating the text was generated by a local OpenAI-compatible ASR endpoint rather than sourced from RSS show notes. [9]
10. Anthropic’s Cybersecurity Shock Wave + Ronan Farrow and Andrew Marantz on Their Sam Altman Investigation + One Good Thing - Hard Fork: I'm sure that there are plenty of businesses that are salivating over the chance to get their hands on it. But they can't unless they are part of this consortium. So they are at least claiming that they are trying to get ahead of what they envision will be a reckoning was what was the word they used for cyber security. And it seems plausible to me that in the next kind of six ish months, every major piece of software... [10]
11. What’s a Hard Fork? - Hard Fork: At a glance The video title asks "What’s a Hard Fork?", but the transcript contains no substantive content on this topic. The provided transcript is solely an automated speech recognition (ASR) system metadata note. The text indicates it originated from a local OpenAI-compatible ASR endpoint, not from official RSS show notes. Overview The video is titled "What’s a Hard Fork?", suggesting an educational focus on block... [11]
12. Anthropic’s Cybersecurity Shock Wave + Ronan Farrow and Andrew Marantz on Their Sam Altman Investigation + One Good Thing - Hard Fork: at anthropic. Oh my gosh. That is amazing. Time limited. It's going to be a time capsule. But I mean, made at the print shop in Brooklyn, one of a kind. Wow. That's incredible. You are here. And I also, I think I should also make. Is this came back for when I gave you a hat at your wedding? And I gave you one at your wedding. So I think we have a sort of a theme going on here. Okay. Right. Well, and that's also our d... [12]

### q038 PASS

- Prompt: What does this creator think about Rust?
- Class: `creator_stance`
- Status: `Completed`
- Score: `3`
- Sources: `12`
- Failure: `-`
- Source videos: A.I. Backlash Turns Violent + Kara Swisher on Healthmaxxing + The Zuck Bot Is Coming | Anthropic’s Cybersecurity Shock Wave + Ronan Farrow and Andrew Marantz on Their Sam Altman Investigation + One Good Thing | What’s a Hard Fork?

#### Answer

Retrieved evidence for: @{Hard Fork} What does this creator think about Rust?

The answer model is unavailable, so this fallback lists the highest-ranked saved excerpts instead of synthesizing beyond them.

1. What’s a Hard Fork? - Hard Fork / At a glance: The video title asks "What’s a Hard Fork?", but the transcript contains no substantive content on this topic. The provided transcript is solely an automated speech recognition (ASR) system metadata note. The text indicates it originated from a local OpenAI-compatible ASR endpoint, not from official RSS show notes. [1]
2. What’s a Hard Fork? - Hard Fork: Podcast ASR smoke transcript. This text came from the local OpenAI-compatible ASR endpoint, not from RSS show notes. [2]
3. What’s a Hard Fork? - Hard Fork / Overview: The video is titled "What’s a Hard Fork?", suggesting an educational focus on blockchain or software development concepts. However, the actual transcript provides no information on this subject. It consists entirely of an ASR metadata placeholder stating the text was generated by a local OpenAI-compatible ASR endpoint rather than sourced from RSS show notes. [3]
4. What’s a Hard Fork? - Hard Fork: At a glance The video title asks "What’s a Hard Fork?", but the transcript contains no substantive content on this topic. The provided transcript is solely an automated speech recognition (ASR) system metadata note. The text indicates it originated from a local OpenAI-compatible ASR endpoint, not from official RSS show notes. Overview The video is titled "What’s a Hard Fork?", suggesting an educational focus on block... [4]
5. Anthropic’s Cybersecurity Shock Wave + Ronan Farrow and Andrew Marantz on Their Sam Altman Investigation + One Good Thing - Hard Fork: at anthropic. Oh my gosh. That is amazing. Time limited. It's going to be a time capsule. But I mean, made at the print shop in Brooklyn, one of a kind. Wow. That's incredible. You are here. And I also, I think I should also make. Is this came back for when I gave you a hat at your wedding? And I gave you one at your wedding. So I think we have a sort of a theme going on here. Okay. Right. Well, and that's also our d... [5]
6. Anthropic’s Cybersecurity Shock Wave + Ronan Farrow and Andrew Marantz on Their Sam Altman Investigation + One Good Thing - Hard Fork: are perfectly content with a free weather app on their phone. That is fine for you. But as somebody who loves cool things, new ideas, people having fun. I just wanted to shout out, act me weather because I think it's a really cool thing. Now, what is the likelihood that this app will be purchased by Apple and then shut down? I mean, if that happens, I hope these guys get paid again because somebody has to move the we... [6]
7. A.I. Backlash Turns Violent + Kara Swisher on Healthmaxxing + The Zuck Bot Is Coming - Hard Fork: called Cara Swisher wants to live forever. Whether she does, in fact, seek immortality is a point of contention as you will hear in the interview. But during this series, Kevin and I were able to watch the first interview episodes and in it, she tries a lot of the things that the rich and powerful are trying as part of their quest to become immortal. >> Yes. So this is a big topic. Obviously, people in tech are very ... [7]
8. A.I. Backlash Turns Violent + Kara Swisher on Healthmaxxing + The Zuck Bot Is Coming - Hard Fork: which are very good. And the way they live longer is they don't sit around and measure fucking everything or just tell us the world is going to die. That is a lot to do. Your mental state has a lot to do with your longevity. And the only thing I would give it to the wellness grifters, a lot of them, is this idea of collapsing health span with lifespan. And I think that's true. We live to, I think it's 79 in this coun... [8]
9. Anthropic’s Cybersecurity Shock Wave + Ronan Farrow and Andrew Marantz on Their Sam Altman Investigation + One Good Thing - Hard Fork: it takes you to write a book. So I think he'll be down to put you in anything else. There's two of you. It should be faster. Ron and Andrew, thanks so much for coming. Thanks, guys. Thanks, guys. Your hats are in the mail. When we come back, what our Spanish language friends would call una cosabuena. Did you just Google that? No. You clotted it? Yes. Okay. I'm Vivian Wong. I'm a journalist at the New York Times. I've... [9]
10. A.I. Backlash Turns Violent + Kara Swisher on Healthmaxxing + The Zuck Bot Is Coming - Hard Fork: mRNA vaccines and AI looking at gene folding. So there was all this real stuff and all this really ridiculous stuff. Right. And so you said sort of like I'm saying a lot of stuff that seems like obviously wrong, but some stuff that seems actually promising. So I want to spend some time and see if I can sort of separate the wheat from the chaff. Right. And I also need to do the stunts because it's funny, right? Like d... [10]
11. A.I. Backlash Turns Violent + Kara Swisher on Healthmaxxing + The Zuck Bot Is Coming - Hard Fork: bored. I mean, aloneness is a very difficult emotion for a podcaster. It is. It is. Yeah. It's aloneness. It's interesting. I feel like the sort of psychedelic. You haven't said if you've taken it, Kevin. I'm on the advice of council. I'm going to respectfully decline the answer. Kevin works at the New York Times. They have opinions about these things. Yes. It's interesting. I feel like there's been a shift in Silico... [11]
12. Anthropic’s Cybersecurity Shock Wave + Ronan Farrow and Andrew Marantz on Their Sam Altman Investigation + One Good Thing - Hard Fork: been a pretty heavy show today. So we thought we wanted to end on a positive note with our segment called One Good Thing. One good thing, of course, our segment where we each talk about one thing that's been tickling our fancy lately. Kevin, why don't you go first this time? Okay, Casey, I am in love with this space mission. Yes. Artemis to mission. I have been totally and earnestly obsessed. My wife was like, you're... [12]

### q039 PASS

- Prompt: What does this creator think about Svelte?
- Class: `creator_stance`
- Status: `Completed`
- Score: `3`
- Sources: `12`
- Failure: `-`
- Source videos: A.I. Backlash Turns Violent + Kara Swisher on Healthmaxxing + The Zuck Bot Is Coming | Anthropic’s Cybersecurity Shock Wave + Ronan Farrow and Andrew Marantz on Their Sam Altman Investigation + One Good Thing | What’s a Hard Fork?

#### Answer

Retrieved evidence for: @{Hard Fork} What does this creator think about Svelte?

The answer model is unavailable, so this fallback lists the highest-ranked saved excerpts instead of synthesizing beyond them.

1. Anthropic’s Cybersecurity Shock Wave + Ronan Farrow and Andrew Marantz on Their Sam Altman Investigation + One Good Thing - Hard Fork: at anthropic. Oh my gosh. That is amazing. Time limited. It's going to be a time capsule. But I mean, made at the print shop in Brooklyn, one of a kind. Wow. That's incredible. You are here. And I also, I think I should also make. Is this came back for when I gave you a hat at your wedding? And I gave you one at your wedding. So I think we have a sort of a theme going on here. Okay. Right. Well, and that's also our d... [1]
2. Anthropic’s Cybersecurity Shock Wave + Ronan Farrow and Andrew Marantz on Their Sam Altman Investigation + One Good Thing - Hard Fork: are perfectly content with a free weather app on their phone. That is fine for you. But as somebody who loves cool things, new ideas, people having fun. I just wanted to shout out, act me weather because I think it's a really cool thing. Now, what is the likelihood that this app will be purchased by Apple and then shut down? I mean, if that happens, I hope these guys get paid again because somebody has to move the we... [2]
3. What’s a Hard Fork? - Hard Fork / Takeaways: The intended educational content regarding what a hard fork is could not be summarized, as the transcript contains only ASR metadata and no actual discussion. [3]
4. A.I. Backlash Turns Violent + Kara Swisher on Healthmaxxing + The Zuck Bot Is Coming - Hard Fork: called Cara Swisher wants to live forever. Whether she does, in fact, seek immortality is a point of contention as you will hear in the interview. But during this series, Kevin and I were able to watch the first interview episodes and in it, she tries a lot of the things that the rich and powerful are trying as part of their quest to become immortal. >> Yes. So this is a big topic. Obviously, people in tech are very ... [4]
5. Anthropic’s Cybersecurity Shock Wave + Ronan Farrow and Andrew Marantz on Their Sam Altman Investigation + One Good Thing - Hard Fork: At a glance Anthropic announced "Claude Mythos Preview," a highly capable new AI model withheld from the public due to severe cybersecurity risks, instead providing access to a defensive tech consortium. The model can autonomously find zero-day exploits in critical open-source infrastructure (e.g., OpenBSD, FFmpeg) that have evaded human researchers and automated tools for decades. The hosts argue this is not a marke... [5]
6. Anthropic’s Cybersecurity Shock Wave + Ronan Farrow and Andrew Marantz on Their Sam Altman Investigation + One Good Thing - Hard Fork / Key Points: senior executives talking about succession at OpenAI. Former public company CEOs (Instacart, Nextdoor, Slack) have been brought in as top lieutenants, introducing "sharp and pointy elbows" and professionalizing influences to counter the "JV board" Altman previously stacked. **The Broader Systemic Issue**: The reporters argue that while individual integrity matters, the core issue is the lack of regulatory guardrails ... [6]
7. What’s a Hard Fork? - Hard Fork / At a glance: The video title asks "What’s a Hard Fork?", but the transcript contains no substantive content on this topic. The provided transcript is solely an automated speech recognition (ASR) system metadata note. The text indicates it originated from a local OpenAI-compatible ASR endpoint, not from official RSS show notes. [7]
8. What’s a Hard Fork? - Hard Fork: At a glance The video title asks "What’s a Hard Fork?", but the transcript contains no substantive content on this topic. The provided transcript is solely an automated speech recognition (ASR) system metadata note. The text indicates it originated from a local OpenAI-compatible ASR endpoint, not from official RSS show notes. Overview The video is titled "What’s a Hard Fork?", suggesting an educational focus on block... [8]
9. What’s a Hard Fork? - Hard Fork: Podcast ASR smoke transcript. This text came from the local OpenAI-compatible ASR endpoint, not from RSS show notes. [9]
10. A.I. Backlash Turns Violent + Kara Swisher on Healthmaxxing + The Zuck Bot Is Coming - Hard Fork: want to see built, you probably actually can make that happen as an average citizen. So that's obviously very inspiring, but I do wish we had other levers that we could pull. Yeah. In part, because I don't think this is going to work, right? Like if you vote the data center project out of your town, they're just going to go to another state or to Canada. They'll put the data centers in space. You know, they've got op... [10]
11. A.I. Backlash Turns Violent + Kara Swisher on Healthmaxxing + The Zuck Bot Is Coming - Hard Fork: mostly right wing elite project that is being championed by President Trump and the many venture capitalists that are in his administration. And if you're already worried that it's going to take your job and you don't feel like you have any control over it, well, of course you're gonna hate it. - So I think there, I don't think this is some like elite right wing plot, but it is definitely an elitist project that is b... [11]
12. A.I. Backlash Turns Violent + Kara Swisher on Healthmaxxing + The Zuck Bot Is Coming - Hard Fork: now it's, you can talk to AI Zuckerberg. Do you think people will attempt to manipulate the chat bot Zuckerberg? or in an attempt to curry favor with the real one, like be like, Hey, you're, you're bot told me that I could get it like a two level promotion and an additional stock grant next year. I'm not sure if you want to honor that or not. That's just, that's what your bot told me. Yeah. I mean, I look, I hope tha... [12]

### q040 PASS

- Prompt: What does this creator think about Databricks?
- Class: `creator_stance`
- Status: `Completed`
- Score: `3`
- Sources: `12`
- Failure: `-`
- Source videos: A.I. Backlash Turns Violent + Kara Swisher on Healthmaxxing + The Zuck Bot Is Coming | Anthropic’s Cybersecurity Shock Wave + Ronan Farrow and Andrew Marantz on Their Sam Altman Investigation + One Good Thing | What’s a Hard Fork?

#### Answer

Retrieved evidence for: @{Hard Fork} What does this creator think about Databricks?

The answer model is unavailable, so this fallback lists the highest-ranked saved excerpts instead of synthesizing beyond them.

1. A.I. Backlash Turns Violent + Kara Swisher on Healthmaxxing + The Zuck Bot Is Coming - Hard Fork: mostly right wing elite project that is being championed by President Trump and the many venture capitalists that are in his administration. And if you're already worried that it's going to take your job and you don't feel like you have any control over it, well, of course you're gonna hate it. - So I think there, I don't think this is some like elite right wing plot, but it is definitely an elitist project that is b... [1]
2. Anthropic’s Cybersecurity Shock Wave + Ronan Farrow and Andrew Marantz on Their Sam Altman Investigation + One Good Thing - Hard Fork: be beautiful wherever you happen to be. Wow. They'll send you an umbrella reminder if it's going to precipitate in the next 12 hours and they'll send you a sunscreen alert when the UV index is high. But I'm saving my last two favorites for the end. Number one, they will send you an alert when the Aurora Borealis may be visible where you are. That's beautiful. I haven't gotten that notification yet, but I wake up ever... [2]
3. A.I. Backlash Turns Violent + Kara Swisher on Healthmaxxing + The Zuck Bot Is Coming - Hard Fork: front door and a note tucked under their doormat that read no data centers. This was someone who had been a supporter of a proposed data center in his district in Indiana and had voted to approve rezoning for the project the week before. And I think this is just part of what I am worried is a growing trend of anti AI radicalization and violence. We should just say like upfront, that we are not fans of violence. We do... [3]
4. A.I. Backlash Turns Violent + Kara Swisher on Healthmaxxing + The Zuck Bot Is Coming - Hard Fork: they can actually change things. Just as the thing that people thought they could do in the 1980s to block new construction in their neighborhoods was to like throw up a bunch of environmental reviews. Like did that help the individual homeowners in that area who didn't want apartment buildings going up? Yes, it kept their views unobstructed. But it also created a massive housing shortage in this state in particular ... [4]
5. What’s a Hard Fork? - Hard Fork / Key Points: Transcript Metadata**: The only content in the transcript is a procedural note indicating it is a "smoke transcript" generated by a local OpenAI-compatible ASR endpoint, explicitly stating it did not come from RSS show notes. No definitions, examples, or explanations of a "hard fork" are present. [5]
6. A.I. Backlash Turns Violent + Kara Swisher on Healthmaxxing + The Zuck Bot Is Coming - Hard Fork: are a lot of people trying to send that message by opposing data centers. But I don't think it's really sunk in at the AI companies or to the people running them that most people want stability in their lives. They want to be able to plan for their futures. And when people from Silicon Valley show up and say, "Hey, we've got this amazing new technology." And by the way, it might take away your job and there's nothing... [6]
7. Anthropic’s Cybersecurity Shock Wave + Ronan Farrow and Andrew Marantz on Their Sam Altman Investigation + One Good Thing - Hard Fork: to see there's a range of reactions, right? There's people who have answered that question in a very severe way and looked at the fact pattern that is laid out here and the documentation. that's laid out and said, you know, this is someone who poses an acute danger and should be kept away from an authority position. And then there's people who I mean, hilariously enough, my mother called me and she's like, you know, ... [7]
8. What’s a Hard Fork? - Hard Fork: Podcast ASR smoke transcript. This text came from the local OpenAI-compatible ASR endpoint, not from RSS show notes. [8]
9. What’s a Hard Fork? - Hard Fork / At a glance: The video title asks "What’s a Hard Fork?", but the transcript contains no substantive content on this topic. The provided transcript is solely an automated speech recognition (ASR) system metadata note. The text indicates it originated from a local OpenAI-compatible ASR endpoint, not from official RSS show notes. [9]
10. What’s a Hard Fork? - Hard Fork / Overview: The video is titled "What’s a Hard Fork?", suggesting an educational focus on blockchain or software development concepts. However, the actual transcript provides no information on this subject. It consists entirely of an ASR metadata placeholder stating the text was generated by a local OpenAI-compatible ASR endpoint rather than sourced from RSS show notes. [10]
11. Anthropic’s Cybersecurity Shock Wave + Ronan Farrow and Andrew Marantz on Their Sam Altman Investigation + One Good Thing - Hard Fork: people defending him in private and the public side tends to be more people criticizing him. I guess for running an energy, do you feel like there are vocal supporters who you came across in reporting the story who had no direct employment relationship with OpenAI or Sam or weren't leading companies that he invested or something who were like, "Yeah, this guy seems pretty good and smart and talented." Yeah, I was an ... [11]
12. Anthropic’s Cybersecurity Shock Wave + Ronan Farrow and Andrew Marantz on Their Sam Altman Investigation + One Good Thing - Hard Fork: forced reset for the entire cybersecurity industry and a very significant event in the history of technology. Yeah. Well, and just to make it concrete, we are currently at war with Iran and Iran is currently hacking our critical infrastructure. There's a story in Wired this week about them successfully hacking like water and energy infrastructure. Right now they're able to do that without a mythos quality model. I wo... [12]

### q041 PASS

- Prompt: Summarize all videos that mention vector search.
- Class: `cross_video_synthesis`
- Status: `Completed`
- Score: `3`
- Sources: `6`
- Failure: `-`
- Source videos: Delete your CLAUDE.md (and your AGENT.md too) | Getting emotional over a million checkboxes | How I Built T3 Chat in 5 Days | Vercel Finally Caught Up | Which browser should you use right now?

#### Answer

Retrieved evidence for: Summarize all videos that mention vector search.

The answer model is unavailable, so this fallback lists the highest-ranked saved excerpts instead of synthesizing beyond them.

1. How I Built T3 Chat in 5 Days - Theo - t3․gg: rabbit they make code review way easier by doing a first pass on your PRS and leaving a bunch of useful feedback summarizing drawing diagrams and so much more this is a real poll request where we're no longer allowing people to upload exe files without paying long story go check out my pirate software video if you want to know more about that but here's what code rabbit did summarize the poll request giving a bunch..... [1]
2. Vercel Finally Caught Up - Theo - t3․gg / Key Points: marginal [2]
3. Getting emotional over a million checkboxes - Theo - t3․gg: going to post a video unless he's doing something else similarly groundbreaking and I want all of y'all to sub to him as well I'm going to go put his channel Link in the description now because if my view count on this video is higher than his sub count I'm disappointed in y'all because this type of genuinely novel approach to building cool unique things on the web and then sharing it is something that we absolutely.... [3]
4. Which browser should you use right now? - Theo - t3․gg: used. Can't even fit the blog item on the page at the same zoom level. But here's what I wanted. I had done this post in June. What are your biggest frustrations with T3 chat right now? And I wanted to collect all of this data. So, let's do it. Summarize all of the replies to this post. Make sure you check all 500 plus of them. Seems like it worked, right? Here's a fun thing. All of those were ones that are in the fi... [4]
5. Delete your CLAUDE.md (and your AGENT.md too) - Theo - t3․gg: app/ts or tsx to find all of the files there. Did the same for convex. Did the same for general source. Found the convex schema. It found the app routes. Found the vcon config ts config. It just read all of these things. And then it after reading all of that concluded has a good understanding of the codebase and it wrote this. But remember what it wrote is based on things that it already was able to find. In fact, it... [5]
6. Which browser should you use right now? - Theo - t3․gg: I recorded that one. For the most part, the video is more relevant than ever. But if you're wondering why I didn't mention that Atlassian bought Browser Company, it's cuz I filmed that video 2 days before Atlassian bought Browser Company. So yeah, take that as you will. Everything else I say is still very true. Speaking of not caring, data Surf is vaporware. They have been teasing it forever on a wait list. I've neve... [6]

### q042 PASS

- Prompt: Summarize all videos that mention transcripts.
- Class: `cross_video_synthesis`
- Status: `Completed`
- Score: `3`
- Sources: `8`
- Failure: `-`
- Source videos: Delete your CLAUDE.md (and your AGENT.md too) | Getting emotional over a million checkboxes | Grok 4 just dropped, it’s the best model right now (yes really) | How I Built T3 Chat in 5 Days | It’s time to embrace the AI | Which browser should you use right now?

#### Answer

Retrieved evidence for: Summarize all videos that mention transcripts.

The answer model is unavailable, so this fallback lists the highest-ranked saved excerpts instead of synthesizing beyond them.

1. How I Built T3 Chat in 5 Days - Theo - t3․gg: rabbit they make code review way easier by doing a first pass on your PRS and leaving a bunch of useful feedback summarizing drawing diagrams and so much more this is a real poll request where we're no longer allowing people to upload exe files without paying long story go check out my pirate software video if you want to know more about that but here's what code rabbit did summarize the poll request giving a bunch..... [1]
2. Getting emotional over a million checkboxes - Theo - t3․gg: going to post a video unless he's doing something else similarly groundbreaking and I want all of y'all to sub to him as well I'm going to go put his channel Link in the description now because if my view count on this video is higher than his sub count I'm disappointed in y'all because this type of genuinely novel approach to building cool unique things on the web and then sharing it is something that we absolutely.... [2]
3. Which browser should you use right now? - Theo - t3․gg: used. Can't even fit the blog item on the page at the same zoom level. But here's what I wanted. I had done this post in June. What are your biggest frustrations with T3 chat right now? And I wanted to collect all of this data. So, let's do it. Summarize all of the replies to this post. Make sure you check all 500 plus of them. Seems like it worked, right? Here's a fun thing. All of those were ones that are in the fi... [3]
4. Delete your CLAUDE.md (and your AGENT.md too) - Theo - t3․gg: app/ts or tsx to find all of the files there. Did the same for convex. Did the same for general source. Found the convex schema. It found the app routes. Found the vcon config ts config. It just read all of these things. And then it after reading all of that concluded has a good understanding of the codebase and it wrote this. But remember what it wrote is based on things that it already was able to find. In fact, it... [4]
5. Which browser should you use right now? - Theo - t3․gg: I recorded that one. For the most part, the video is more relevant than ever. But if you're wondering why I didn't mention that Atlassian bought Browser Company, it's cuz I filmed that video 2 days before Atlassian bought Browser Company. So yeah, take that as you will. Everything else I say is still very true. Speaking of not caring, data Surf is vaporware. They have been teasing it forever on a wait list. I've neve... [5]
6. It’s time to embrace the AI - Theo - t3․gg: do as developers in a way that we should talk about. This is a thing I've been feeling for a while. So, when I saw this article pop up on HackerNews, I really wanted to talk about it. I started reading it, immediately realized I had to share it with you guys. So, now we're doing a video. This article posted on the Fly.io blog. I love Fly, one of my favorite ways to deploy servers. That said, they are not sponsoring t... [6]
7. Grok 4 just dropped, it’s the best model right now (yes really) - Theo - t3․gg: arently early September, a multimodel agent will be released. And then by October, a video generation model will be as well. If you know anything about how XAI is about timelines, you should bump everything on that between two months and a year. Regardless, yeah, they're a serious player. Even if the presentation was, as usual, cringe as the model is good. I'm not going to pretend otherwise. I haven't played with it.... [7]
8. How I Built T3 Chat in 5 Days - Theo - t3․gg: in case you haven't seen yet I just put out a new app called T3 chat and I'm really proud of it it's the fastest AI chat app I've ever used and as far as I know currently exists if you don't believe me go try it or watch my other videos about it it flies been getting a lot of questions about how I built it how it's so fast and most importantly how the hell did I do this in 5 days these are all great questions and not... [8]

### q043 PASS

- Prompt: Summarize all videos that mention highlights.
- Class: `cross_video_synthesis`
- Status: `Completed`
- Score: `3`
- Sources: `6`
- Failure: `-`
- Source videos: Delete your CLAUDE.md (and your AGENT.md too) | Getting emotional over a million checkboxes | How I Built T3 Chat in 5 Days | What happened to me? | Which browser should you use right now?

#### Answer

Retrieved evidence for: Summarize all videos that mention highlights.

The answer model is unavailable, so this fallback lists the highest-ranked saved excerpts instead of synthesizing beyond them.

1. How I Built T3 Chat in 5 Days - Theo - t3․gg: rabbit they make code review way easier by doing a first pass on your PRS and leaving a bunch of useful feedback summarizing drawing diagrams and so much more this is a real poll request where we're no longer allowing people to upload exe files without paying long story go check out my pirate software video if you want to know more about that but here's what code rabbit did summarize the poll request giving a bunch..... [1]
2. What happened to me? - Theo - t3․gg: this can change the same way it changed here. If I start doing more of these types of videos and they perform better than expected, I'll lean more into this. For example, the logging video, we screwed up the export initially and the first version that went up was too short and had most of the content missing. So, we had to re-upload it, which destroys the video performance because a lot of people already saw it, so t... [2]
3. Getting emotional over a million checkboxes - Theo - t3․gg: going to post a video unless he's doing something else similarly groundbreaking and I want all of y'all to sub to him as well I'm going to go put his channel Link in the description now because if my view count on this video is higher than his sub count I'm disappointed in y'all because this type of genuinely novel approach to building cool unique things on the web and then sharing it is something that we absolutely.... [3]
4. Which browser should you use right now? - Theo - t3․gg: used. Can't even fit the blog item on the page at the same zoom level. But here's what I wanted. I had done this post in June. What are your biggest frustrations with T3 chat right now? And I wanted to collect all of this data. So, let's do it. Summarize all of the replies to this post. Make sure you check all 500 plus of them. Seems like it worked, right? Here's a fun thing. All of those were ones that are in the fi... [4]
5. Delete your CLAUDE.md (and your AGENT.md too) - Theo - t3․gg: app/ts or tsx to find all of the files there. Did the same for convex. Did the same for general source. Found the convex schema. It found the app routes. Found the vcon config ts config. It just read all of these things. And then it after reading all of that concluded has a good understanding of the codebase and it wrote this. But remember what it wrote is based on things that it already was able to find. In fact, it... [5]
6. Which browser should you use right now? - Theo - t3․gg: I recorded that one. For the most part, the video is more relevant than ever. But if you're wondering why I didn't mention that Atlassian bought Browser Company, it's cuz I filmed that video 2 days before Atlassian bought Browser Company. So yeah, take that as you will. Everything else I say is still very true. Speaking of not caring, data Surf is vaporware. They have been teasing it forever on a wait list. I've neve... [6]

### q044 PASS

- Prompt: Summarize all videos that mention summaries.
- Class: `cross_video_synthesis`
- Status: `Completed`
- Score: `3`
- Sources: `9`
- Failure: `-`
- Source videos: Delete your CLAUDE.md (and your AGENT.md too) | Getting emotional over a million checkboxes | Grok 4 just dropped, it’s the best model right now (yes really) | How I Built T3 Chat in 5 Days | It’s time to embrace the AI | Vercel Finally Caught Up | Which browser should you use right now?

#### Answer

Retrieved evidence for: Summarize all videos that mention summaries.

The answer model is unavailable, so this fallback lists the highest-ranked saved excerpts instead of synthesizing beyond them.

1. How I Built T3 Chat in 5 Days - Theo - t3․gg: rabbit they make code review way easier by doing a first pass on your PRS and leaving a bunch of useful feedback summarizing drawing diagrams and so much more this is a real poll request where we're no longer allowing people to upload exe files without paying long story go check out my pirate software video if you want to know more about that but here's what code rabbit did summarize the poll request giving a bunch..... [1]
2. Getting emotional over a million checkboxes - Theo - t3․gg: going to post a video unless he's doing something else similarly groundbreaking and I want all of y'all to sub to him as well I'm going to go put his channel Link in the description now because if my view count on this video is higher than his sub count I'm disappointed in y'all because this type of genuinely novel approach to building cool unique things on the web and then sharing it is something that we absolutely.... [2]
3. Which browser should you use right now? - Theo - t3․gg: used. Can't even fit the blog item on the page at the same zoom level. But here's what I wanted. I had done this post in June. What are your biggest frustrations with T3 chat right now? And I wanted to collect all of this data. So, let's do it. Summarize all of the replies to this post. Make sure you check all 500 plus of them. Seems like it worked, right? Here's a fun thing. All of those were ones that are in the fi... [3]
4. Delete your CLAUDE.md (and your AGENT.md too) - Theo - t3․gg: app/ts or tsx to find all of the files there. Did the same for convex. Did the same for general source. Found the convex schema. It found the app routes. Found the vcon config ts config. It just read all of these things. And then it after reading all of that concluded has a good understanding of the codebase and it wrote this. But remember what it wrote is based on things that it already was able to find. In fact, it... [4]
5. Which browser should you use right now? - Theo - t3․gg: I recorded that one. For the most part, the video is more relevant than ever. But if you're wondering why I didn't mention that Atlassian bought Browser Company, it's cuz I filmed that video 2 days before Atlassian bought Browser Company. So yeah, take that as you will. Everything else I say is still very true. Speaking of not caring, data Surf is vaporware. They have been teasing it forever on a wait list. I've neve... [5]
6. Vercel Finally Caught Up - Theo - t3․gg / Key Points: marginal [6]
7. It’s time to embrace the AI - Theo - t3․gg: do as developers in a way that we should talk about. This is a thing I've been feeling for a while. So, when I saw this article pop up on HackerNews, I really wanted to talk about it. I started reading it, immediately realized I had to share it with you guys. So, now we're doing a video. This article posted on the Fly.io blog. I love Fly, one of my favorite ways to deploy servers. That said, they are not sponsoring t... [7]
8. Grok 4 just dropped, it’s the best model right now (yes really) - Theo - t3․gg: arently early September, a multimodel agent will be released. And then by October, a video generation model will be as well. If you know anything about how XAI is about timelines, you should bump everything on that between two months and a year. Regardless, yeah, they're a serious player. Even if the presentation was, as usual, cringe as the model is good. I'm not going to pretend otherwise. I haven't played with it.... [8]
9. How I Built T3 Chat in 5 Days - Theo - t3․gg: in case you haven't seen yet I just put out a new app called T3 chat and I'm really proud of it it's the fastest AI chat app I've ever used and as far as I know currently exists if you don't believe me go try it or watch my other videos about it it flies been getting a lot of questions about how I built it how it's so fast and most importantly how the hell did I do this in 5 days these are all great questions and not... [9]

### q045 PASS

- Prompt: Summarize all videos that mention quality scores.
- Class: `cross_video_synthesis`
- Status: `Completed`
- Score: `3`
- Sources: `12`
- Failure: `-`
- Source videos: Delete your CLAUDE.md (and your AGENT.md too) | Getting emotional over a million checkboxes | Hacking LightHouse Scores | How I Built T3 Chat in 5 Days | Which browser should you use right now?

#### Answer

Retrieved evidence for: Summarize all videos that mention quality scores.

The answer model is unavailable, so this fallback lists the highest-ranked saved excerpts instead of synthesizing beyond them.

1. How I Built T3 Chat in 5 Days - Theo - t3․gg: rabbit they make code review way easier by doing a first pass on your PRS and leaving a bunch of useful feedback summarizing drawing diagrams and so much more this is a real poll request where we're no longer allowing people to upload exe files without paying long story go check out my pirate software video if you want to know more about that but here's what code rabbit did summarize the poll request giving a bunch..... [1]
2. Hacking LightHouse Scores - Theo - t3․gg / Key Points: The Context and Purpose of Lighthouse Lighthouse scores are used by product managers, marketing teams, and developers to evaluate frameworks and shame poorly performing sites (with Angular frequently targeted). Despite the focus on scores, the video argues that Lighthouse's greatest value is sparking conversations about performance and accessibility—even if imperfect, it has encouraged developers to build better, mor... [2]
3. Getting emotional over a million checkboxes - Theo - t3․gg: going to post a video unless he's doing something else similarly groundbreaking and I want all of y'all to sub to him as well I'm going to go put his channel Link in the description now because if my view count on this video is higher than his sub count I'm disappointed in y'all because this type of genuinely novel approach to building cool unique things on the web and then sharing it is something that we absolutely.... [3]
4. Hacking LightHouse Scores - Theo - t3․gg: very large and complex web performance puzzle and without field data I'm not sure any of this matters anyways couldn't agree more so let's take a look at how to hack these scores tldr show the smallest amount of LCP qualifying content on load to boost the FCP and LCP scores until the lighthouse tests have likely finished I've seen this before pages that will delay a big paint until they think Lighthouse is done so th... [4]
5. Hacking LightHouse Scores - Theo - t3․gg: contribute to the final score you can play around with the sliders on the lighthouse scoring calculator interesting I know they had that yeah there's a calculator so you can see as these things move how much does it matter so if everything else was great but the FCP took 6 seconds so it took six seconds for the page to to show anything but everything else was fast you're still getting a 90 that feels kind of shitty t... [5]
6. Which browser should you use right now? - Theo - t3․gg: used. Can't even fit the blog item on the page at the same zoom level. But here's what I wanted. I had done this post in June. What are your biggest frustrations with T3 chat right now? And I wanted to collect all of this data. So, let's do it. Summarize all of the replies to this post. Make sure you check all 500 plus of them. Seems like it worked, right? Here's a fun thing. All of those were ones that are in the fi... [6]
7. Hacking LightHouse Scores - Theo - t3․gg / Overview: This video explores Google Lighthouse performance scoring, examining how scores are calculated, their real-world relevance, and whether they can be manipulated. Hosted by a developer discussing a blog post by Salma (sponsored by Sentry), the content systematically breaks down each Lighthouse metric, demonstrates multiple hacks to artificially inflate scores, and argues that Lighthouse is a useful but rough guide that... [7]
8. Delete your CLAUDE.md (and your AGENT.md too) - Theo - t3․gg: app/ts or tsx to find all of the files there. Did the same for convex. Did the same for general source. Found the convex schema. It found the app routes. Found the vcon config ts config. It just read all of these things. And then it after reading all of that concluded has a good understanding of the codebase and it wrote this. But remember what it wrote is based on things that it already was able to find. In fact, it... [8]
9. Which browser should you use right now? - Theo - t3․gg: I recorded that one. For the most part, the video is more relevant than ever. But if you're wondering why I didn't mention that Atlassian bought Browser Company, it's cuz I filmed that video 2 days before Atlassian bought Browser Company. So yeah, take that as you will. Everything else I say is still very true. Speaking of not caring, data Surf is vaporware. They have been teasing it forever on a wait list. I've neve... [9]
10. Hacking LightHouse Scores - Theo - t3․gg: to go use the app means that this number is meanless but also since these things are being indexed they don't have to care there are likely many other situations where apps serve user generated content and you might be unable to control the LCP element entirely particularly regarding images images in video are the bane of Lighthouse existence it's so bad for example if you can control the sizes of all images on your ... [10]
11. Hacking LightHouse Scores - Theo - t3․gg / Key Points: user input responses **Cumulative Layout Shift (CLS)**: 25% weight — measures unexpected visual shifts during page load **Largest Contentful Paint (LCP)**: 25% weight — marks when main content has likely loaded **First Contentful Paint (FCP)**: 10% weight — first point where users see anything on screen **Speed Index (SI)**: 10% weight — measures how quickly content is visually displayed during page load Thresholds f... [11]
12. Hacking LightHouse Scores - Theo - t3․gg: our users we feel like the fastest website in the world but to other users that might be on mobile sites trying to make a quick like thing in their Bank the things that they're going to be looking for here are going to be entirely different so that's an important piece to consider as we go through this is that these metrics even if they're bad might not actually show you what the experiences like for those users our ... [12]

### q046 PASS

- Prompt: Which videos discuss tradeoffs between speed and accuracy?
- Class: `comparison`
- Status: `Completed`
- Score: `3`
- Sources: `12`
- Failure: `-`
- Source videos: A free model just appeared in Cursor (and it’s really good at code) | Claude Code has a big problem | Going Back To Next | JavaScript Frameworks in 2025 | Opus 4.6 Is The Best Coding Model Ever Made* | Rate Limiting | The fastest website ever? | Vite Raised $4.6 Million To Fix JavaScript

#### Answer

Retrieved evidence for: Which videos discuss tradeoffs between speed and accuracy?

The answer model is unavailable, so this fallback lists the highest-ranked saved excerpts instead of synthesizing beyond them.

1. Going Back To Next - Theo - t3․gg / Overview: ents why he ultimately decided to return to Next.js and JavaScript. Theo provides extensive commentary throughout, offering technical insights, personal experiences from his time at Twitch, and analysis of the tradeoffs between different technology stacks. The discussion covers language ergonomics, error handling, developer experience, context switching costs, and a decision framework for choosing between technology.... [1]
2. The fastest website ever? - Theo - t3․gg: every few months the McMaster website goes viral for being super fast and it really is just navigating the website feels like it's flying and the fact that it's using things that aren't modern webtech is really interesting but is it just vanilla HTML and can we go faster if we Branch away from that we have a lot to dive into here this isn't a simple thing to cover because there's so many misconceptions about what mak... [2]
3. Rate Limiting - Theo - t3․gg / Overview: This video explores rate limiting algorithms through an article titled "Visualizing Algorithms for Rate Limiting" written by a community member. The presenter analyzes three primary algorithms—fixed window, sliding window, and token bucket—explaining their mechanics, advantages, and drawbacks with interactive visualizations. The discussion covers real-world implementations from GitHub, Cloudflare, Stripe, and OpenAI,... [3]
4. Vite Raised $4.6 Million To Fix JavaScript - Theo - t3․gg: e Remix, Astro, SvelteKit, and others. The speaker argues that VC-backed open source has different risks but isn't inherently worse than corporate-backed or hobbyist models, each having distinct sustainability tradeoffs. Rolldown aims to become the unified bundler for Vite in both dev and prod, potentially enabling fast production builds by end of year. VoidZero plans to offer a separate enterprise-focused toolchain.... [4]
5. Going Back To Next - Theo - t3․gg: asize that "building faster doesn't mean you're building more wrong" - rapid iteration with productive tools can lead to correct solutions more quickly. A major theme throughout is context switching: switching between different languages, ecosystems, and codebases creates significant productivity loss, and full-stack TypeScript frameworks like Next.js minimize this friction. The discussion includes a framework for ch... [5]
6. The fastest website ever? - Theo - t3․gg / Takeaways: Don't attribute speed to architecture alone**: McMaster's speed comes from deliberate prefetching engineering, not from avoiding frameworks. Their custom JS solution is essentially a custom framework. **PageSpeed scores don't tell the whole story**: A site can feel incredibly fast to users while showing poor metrics; actual user experience should drive optimization decisions. **Prefetch strategically, not comprehensi... [6]
7. The fastest website ever? - Theo - t3․gg: diagnos of performance issues I have a good feeling there too come on Google there we go still not a perfect score in performance but the accessibility is better it is overall good let's take a look at how it actually feels to browse it this is next faster a copyright distinct entity as a demonstration of what a website that vaguely looks and navigates similar to mcmas but is totally not McMaster or even a reference ... [7]
8. Opus 4.6 Is The Best Coding Model Ever Made* - Theo - t3․gg: es like reporting placeholder credentials as critical security issues. Anthropic blocked "partial turn prefill" misuse vectors in the API, which has implications for model-swapping and chat history portability between providers. Speaker's overall verdict: roughly a 5-10% improvement in capability with a 3-5% loss in interaction quality, plus speed regression. Overview This video is an in-depth review of Anthropic's n... [8]
9. The fastest website ever? - Theo - t3․gg / Overview: This extensive technical deep-dive analyzes why the McMaster-Carr industrial supply website is renowned for its speed and then explores whether a Next.js implementation could be even faster. The video deconstructs the misconception that McMaster uses simple "vanilla HTML," revealing instead a complex custom JavaScript framework handling prefetching and client-side navigation. It then examines "Next Faster," a demonst... [9]
10. Claude Code has a big problem - Theo - t3․gg: rewrite core primitives if performance becomes an issue—they've since forked Ink and added native components. Alternatives like Codex (Rust-based, uses Ratatouille) and Open Code (uses alt mode) have different tradeoffs: Codex doesn't rewrap text on resize; Open Code has better performance but loses standard terminal behaviors like text selection. The underlying issue is that terminals weren't designed for complex UI... [10]
11. A free model just appeared in Cursor (and it’s really good at code) - Theo - t3․gg: of the time right, how many times do you have to rerun before you have a 99.9% chance you've gotten a right answer? For Sonic it would be 4.29 times. For GBT5 it'll be less cuz it's So here it will be 3 to 99.9 which makes sense. So let's do out the math here. How many minutes will it take to have a 99.9% likelihood you've gotten a correct answer for GBT 5? this will take minutes and for Sonic it will take ish minute... [11]
12. JavaScript Frameworks in 2025 - Theo - t3․gg: S on the server is almost necessary for optimal sites. The complexity debate is often about perspective: GraphQL vs tRPC comparisons misleadingly focus only on client-side complexity while ignoring server-side tradeoffs; similarly, the shift from SPA to isomorphic models exposes frontend devs to complexity they previously ignored. React Compiler and Svelte 5 Runes represent opposing compiler philosophies—React Compil... [12]

### q047 PASS

- Prompt: Which videos talk about evaluation or judging model outputs?
- Class: `direct_lookup`
- Status: `Completed`
- Score: `3`
- Sources: `12`
- Failure: `-`
- Source videos: BREAKING: OpenAI's new O3 model changes everything | ChatGPT “Pro” Has Some Real Safety Concerns... | Delete your CLAUDE.md (and your AGENT.md too) | Did Meta Really Fake Benchmarks? | GlazeGPT got rolled back (4o update gone wrong) | Never mind (OpenAI won again) | OpenAI Fights Back (GPT 4.5 is wild) | The end of the Clawdbot saga | There's a new best OSS model and it's...weird | We need to talk about "founder mode" | You’re all wrong

#### Answer

Retrieved evidence for: Which videos talk about evaluation or judging model outputs?

The answer model is unavailable, so this fallback lists the highest-ranked saved excerpts instead of synthesizing beyond them.

1. Did Meta Really Fake Benchmarks? - Theo - t3․gg / Overview: This video examines Meta's release of the Llama 4 model family—comprising Scout, Maverick, and Behemoth—and investigates allegations that Meta manipulated benchmark results. The host explores the timing of the surprise Saturday release, dissects performance metrics that appear to underperform compared to competitors like Gemini and DeepSeek, and addresses claims from a purported former employee about unethical traini... [1]
2. BREAKING: OpenAI's new O3 model changes everything - Theo - t3․gg: tself you tell it what you want it to do and it's making its own agents effectively that's nuts it's so nuts that instead of just releasing it outright they're doing an early access window for safety test as I talked about in the 01 pro video there are now some concerning behaviors in these models that are worth considering when we think about AI safety going forward the models are now smart enough that they'll when.... [2]
3. ChatGPT “Pro” Has Some Real Safety Concerns... - Theo - t3․gg: much as bad as opuses by default which is funny by itself but also 01 drops less it's funny to think that Sonet is that much more accurate with good incentives and it's as bad as Opus with bad incentives yeah very interesting numbers they call out the valuation scenarios here so you can better saying if you're curious I don't want to go tooo in depth here but I wanted to call out that they say that their evaluation s... [3]
4. OpenAI Fights Back (GPT 4.5 is wild) - Theo - t3․gg: tools out a ton I don't have any affiliation with these guys they're not paying me anything I just think it's a good survey give it a shot if you can anyways back to benchmarks they talk a lot about jailbreaking stuff they have to it's the security thing but they also called out that it's very low risk because it's not very good at things like cyber security and cbrn stuff and it's also low autonomy because it doesn'... [4]
5. The end of the Clawdbot saga - Theo - t3․gg: ing to get all of that money for yourself, I'm also going to go to jail because it's against the law. The reality is honestly kind of funnier. I'm nice to OpenAI because OpenAI is nice to me. I say nice things about OpenAI's products because OpenAI's products are good. I also talk [ __ ] on OpenAI's products when I don't think they're that good. I talk so much [ __ ] on Atlas that it's crazy to me anyone would say th... [5]
6. There's a new best OSS model and it's...weird - Theo - t3․gg / Overview: This video examines Alibaba's Qwen team's release of QwQ, a 32-billion parameter reinforcement learning reasoning model positioned as having performance comparable to the much larger DeepSeek R1 (671B parameters). The creator conducts extensive hands-on testing comparing QwQ against DeepSeek R1 distilled models and Claude, discovering significant discrepancies between impressive benchmark claims and real-world perfor... [6]
7. GlazeGPT got rolled back (4o update gone wrong) - Theo - t3․gg: talk about how they're addressing this. I actually think their plan is solid. Beyond rolling back the latest 4 update, we're taking more steps to realign the model's behavior. They're refining core training techniques and system prompts to explicitly steer the model away from sick fancy. They're building more guardrails to increase honesty and transparency, which are both principles in the model spec. They're expandi... [7]
8. Delete your CLAUDE.md (and your AGENT.md too) - Theo - t3․gg: be open source soon. Just a way to do video review for my team. And I had it init a claude MD. Let's see how it did. File provides guidance to cloud code. Cloud.aii when working with code in this repo. That's the intro it uses on all of these. It used it on other ones as well. Lawn's a video review platform for creative teams. Users upload video, leave timestamp comments, and manage review workflows within the team a... [8]
9. You’re all wrong - Theo - t3․gg: amigdala, your threat detection system, the same system that fires when you encounter a predator or a physical danger, activates immediately. If you've ever seen people saying nasty things about me online, that I'm a degenerate, click baiting soy boy that pays Indian devs to write all their code for him because I suck. That's because I challenged their identity at some point. Something I said ran against their belief... [9]
10. You’re all wrong - Theo - t3․gg: merit, often costing companies millions. Neuroscience research shows that when identity-based beliefs are challenged, the brain responds as if under physical attack—the amygdala activates, preventing objective evaluation. Multiple case studies demonstrate the cost of identity-driven decisions: a CTO forced a PHP-to-Perl switch that collapsed a startup, and a VP pushed for Rust based purely on hype without evaluating.... [10]
11. We need to talk about "founder mode" - Theo - t3․gg / Key Points: for things you love and understand deeply, forcing yourself to do the work you don't enjoy until you've mastered it enough to hire well. The creator learned this by editing his own videos after four failed editor hires—he now loves editing and can properly evaluate editors. **Real Example - Ping Infrastructure**: The creator gave the exciting infrastructure rebuild work to Mark and Julius rather than keeping it himse... [11]
12. Never mind (OpenAI won again) - Theo - t3․gg: h tool use and complex execution. Much like a colleague, you can steer and interact with 5.3 codecs while it's working without losing context. Honestly, it's the only thing they mentioned when we first started talking to them about this new model that it feels more like a colleague or a co-orker and it can be steered. And I've been doing that a lot and I'm actually really enjoying it. Like it's more fun than ever to.... [12]

### q048 PASS

- Prompt: Which videos mention failure cases or limitations?
- Class: `direct_lookup`
- Status: `Completed`
- Score: `3`
- Sources: `12`
- Failure: `-`
- Source videos: Anthropic is trying SO hard to fix MCP... | Did Meta Really Fake Benchmarks? | Firebase made an IDE? | Laid off engineers replaced with AI??? | Namecheap is suing their customers | Rate Limiting | The most important function in my codebase | This is good, actually | This model is kind of a disaster. | What happened to me?

#### Answer

Retrieved evidence for: Which videos mention failure cases or limitations?

The answer model is unavailable, so this fallback lists the highest-ranked saved excerpts instead of synthesizing beyond them.

1. Laid off engineers replaced with AI??? - Theo - t3․gg / Key Points: ontent instead of relevant software development topics, proving how bad YouTube's recommendations are. The speaker attempted to showcase Google's Gemini Nano integration in Dev Tools but encountered errors and failures, criticizing the implementation as "cringe" and non-functional. An earlier test showed Gemini Nano hallucinated a picture of Albert Einstein when asked "who is he" with no image provided. Context on Go... [1]
2. Did Meta Really Fake Benchmarks? - Theo - t3․gg: what people have seen. The long context stuff seems really cool that it can handle such large amounts of data, but how does that end up working in practice? There's a benchmark for retrieving data from long context. And this benchmark actually saw Llama Force Scout having the worst score on the entire page because they on the K token context test just couldn't get any of the answers right. 11% accuracy, which is hila... [2]
3. Laid off engineers replaced with AI??? - Theo - t3․gg: a fraction of the profit compared to YouTube, which makes more than Cloud despite Google's engineering reputation. The speaker criticizes YouTube's AI-generated content suggestions as irrelevant and highlights failures in Google's Gemini integration for developers. Overview The video addresses the trend of engineers being replaced by AI, clarifying that while most news on the topic is clickbait, a significant develop... [3]
4. This model is kind of a disaster. - Theo - t3․gg: And if you have a different experience for me, please let me know. I'm just one guy that tested this over 12-ish hours throughout the day. I can't possibly know all of the things it's great or bad at. All I know is that I had a bad experience and I wanted to share a bit of what that looked like for y'all. Let me know how y'all feel. And until next time, let's just hope I don't knock any cables out. Peace, nerds. [4]
5. Rate Limiting - Theo - t3․gg / Overview: This video explores rate limiting algorithms through an article titled "Visualizing Algorithms for Rate Limiting" written by a community member. The presenter analyzes three primary algorithms—fixed window, sliding window, and token bucket—explaining their mechanics, advantages, and drawbacks with interactive visualizations. The discussion covers real-world implementations from GitHub, Cloudflare, Stripe, and OpenAI,... [5]
6. This is good, actually - Theo - t3․gg: t the failed step?) Workflows need to pause for extended periods (e.g., "send welcome email, wait 7 days, send check-in email") External services (databases, LLMs, email providers like Resend) have independent failure rates that compound Servers redeploy, restart, or crash mid-execution The speaker provides a concrete example: a signup flow with three operations (create user, send welcome email, wait 7 days, send che... [6]
7. What happened to me? - Theo - t3․gg: this can change the same way it changed here. If I start doing more of these types of videos and they perform better than expected, I'll lean more into this. For example, the logging video, we screwed up the export initially and the first version that went up was too short and had most of the content missing. So, we had to re-upload it, which destroys the video performance because a lot of people already saw it, so t... [7]
8. Namecheap is suing their customers - Theo - t3․gg: gal Strategy **Historical Parallel**: The host draws a comparison to the Church of Scientology's strategy against the IRS in the 1990s. Scientology members personally sued IRS employees individually—not to win cases, but to overwhelm the agency. **Outcome**: In 1993, the IRS granted Scientology tax-exempt status in exchange for dropping approximately 2,500 lawsuits against individual IRS employees. **Relevance**: The... [8]
9. What happened to me? - Theo - t3․gg: it. I will rebrand it. I will try different things. But my excitement on doing videos about CSS went down a ton as a result of that video bombing. And this isn't because the algorithm hates the video. This isn't because people only click AI videos. It is simply a matter of how interest has shifted where an average dev video has just fully dropped off. Like people don't care about average levels of interest in dev top... [9]
10. Anthropic is trying SO hard to fix MCP... - Theo - t3․gg: g Opus 4's accuracy from 49% to 74% (matching unoptimized Opus 4.5) Programmatic Tool Calling lets Claude write code to execute tools rather than using natural language inference, eliminating the 10-40% lookup failure rate models have when parsing large datasets Tool Use Examples provides sample tool calls alongside JSON schemas, improving accuracy from 72% to 90% on complex parameter handling The creator argues thes... [10]
11. Firebase made an IDE? - Theo - t3․gg: ce despite their polished appearances. Key Points Sponsorship and Disclosure The creator has an existing sponsorship agreement with Project IDX (which has now become Firebase Studio) with six planned sponsored videos, but this particular video is not sponsored by Firebase. Code Rabbit is the actual sponsor of this video, a code review tool that integrates with GitHub PRs and provides inline suggestions in the editor.... [11]
12. The most important function in my codebase - Theo - t3․gg: meant as a starting point for better error handling practices. The neverthrow Library `neverthrow` implements the Result type pattern—a more structured approach where functions always return either success or failure, never throwing exceptions. Core concepts**: Functions return `Result<T, E>` where T is the success type and E is the error type Success is wrapped with `ok(data)`, errors with `err(errorType)` Error ty.... [12]

### q049 PASS

- Prompt: What are the most skeptical views in my library?
- Class: `tone_or_style_inference`
- Status: `Completed`
- Score: `3`
- Sources: `12`
- Failure: `-`
- Source videos: "Vibe Coding" Is A Stupid Trend | ChatGPT Atlas Drove Me Insane (it's not just Chrome) | Defending my product from the dumbest possible haters | Everything Google just announced | I can't believe he was right. | I didn't expect Meta to push React this hard... | I was wrong - AI video is nuts (don't sleep on Veo 3) | It’s time to embrace the AI | So close to Opus at 1/10th the price (GLM-4.7 and Minimax M2.1 showdown) | Vite Raised $4.6 Million To Fix JavaScript | What is Theo's Worst Take?

#### Answer

Retrieved evidence for: What are the most skeptical views in my library?

The answer model is unavailable, so this fallback lists the highest-ranked saved excerpts instead of synthesizing beyond them.

From the available evidence, these excerpts support only a tentative reading rather than a definitive judgment.

1. What is Theo's Worst Take? - Theo - t3․gg: and what is my worst take you said all of your takes were good and so maybe that's your worst take I remember you saying something evil about go there were so many of them uh that story book is useless why is it not useless uh because you you need to fill up your known module somehow you got me on that one [1]
2. It’s time to embrace the AI - Theo - t3․gg / Key Points: e. His perspective changed through using Cursor extensively (tab autocomplete, command-I, command-K features), building T3 Chat, and working with newer agents and models. He notes that conversations with still-skeptical friends now feel strange because his views have shifted so dramatically after giving the tools a serious try when they improved. The Article and Its Author The source article is "A heartfelt provocati... [2]
3. Vite Raised $4.6 Million To Fix JavaScript - Theo - t3․gg: that he's increasingly skeptical of Open Source that is directly tied with VC I hear it but I don't fully agree I think all open source has things worth considering when we talk about how they are funded and what makes them exist over time a hobby project could have the person just get bored to go work on other things like Vue for example now that Evan U is working on something else that is well funded and what he's.... [3]
4. It’s time to embrace the AI - Theo - t3․gg: ers for meaningful architecture and judgment work. Developers remain responsible for reading and curating AI-generated code; the output is deterministic code, not hallucinated nonsense. Mediocre code isn't bad—most code is mediocre, and AI raising the floor is valuable even if it doesn't raise the ceiling. The "AI takes jobs" argument ignores that tech has always automated jobs; developers aren't exempt from this rea... [4]
5. I was wrong - AI video is nuts (don't sleep on Veo 3) - Theo - t3․gg: generations two at a time, this effectively provides about 40 prompts with default settings. Per-second pricing is 50 cents for video without audio and 75 cents for video with audio. The creator burned through most credits quickly and wanted more, indicating heavy usage during testing. Impressive Generation Capabilities Demonstrated One early test showed a "T3 Chat" promotional clip where the model correctly: transit... [5]
6. ChatGPT Atlas Drove Me Insane (it's not just Chrome) - Theo - t3․gg: s from the main native Swift UI app. The architecture relies heavily on Apple-specific frameworks (Swift UI, AppKit, Metal), making cross-platform support (especially Windows) extremely difficult. Input events are translated through a complex multi-stage pipeline (NS Event → Web Input Event → potentially re-synthesized NS Event), which the speaker finds horrifying to maintain. Atlas uses Chromium's Mojo IPC system wi... [6]
7. I can't believe he was right. - Theo - t3․gg: s) using Claude Code and Opus, without opening a traditional editor. Google reports 25%+ of code is AI-written; Microsoft reports ~30%; 32% of senior devs say at least half their code comes from AI—senior devs are adopting these tools fastest. The role of developers is shifting from writing code to reviewing and orchestrating AI-generated code, similar to how engineers transition to management roles. AI-generated cod... [7]
8. "Vibe Coding" Is A Stupid Trend - Theo - t3․gg: TL;DR "Vibe coding" was coined by Andrej Karpathy in early 2025 to describe building software with LLMs without reviewing the code—fully embracing the "vibes" and ignoring the code's existence. The term is being incorrectly diluted to mean any AI-assisted coding, which undermines its usefulness as a specific concept distinct from responsible AI-assisted programming. Simon Willison (Django creator) and the speaker arg... [8]
9. I didn't expect Meta to push React this hard... - Theo - t3․gg: k.com's Newsfeed logic in the Quest app) allows teams to focus on platform-specific experiences rather than relearning how to build apps. Meta's internal tools like StyleX, React Strict DOM, and custom routers are being used to build cross-platform UIs, showcasing the flexibility of the React ecosystem. Overview This video analyzes a Meta blog post detailing how React and React Native power the user interfaces showca... [9]
10. Everything Google just announced - Theo - t3․gg: generates both video and audio, taking 2-3 minutes per generation; quality is strong but audio synchronization with video has issues. Android XR glasses were demonstrated but with significant limitations compared to Vision Pro's persistent window positioning. Overview This video provides a comprehensive developer-focused breakdown of Google's announcements from Google I/O, covering major developments across the Gem..... [10]
11. So close to Opus at 1/10th the price (GLM-4.7 and Minimax M2.1 showdown) - Theo - t3․gg: ely 1/60th of Opus 4.1's original pricing). GLM 4.7 excels at UI/design tasks and visual outputs, while MiniMax M2.1 excels at long-running coding tasks, planning, and sustained multi-file changes. Both models are open-weight (M2.1 weights expected to drop around Christmas), runnable on consumer hardware, and represent a major shift in what's possible for budget-conscious developers. Overview This video provides an i... [11]
12. Defending my product from the dumbest possible haters - Theo - t3․gg: dge compute for data-heavy workloads, advocates for edge runtimes for cold-start benefits, and remains strongly pro-serverless, running servers only where necessary (e.g., long-running file ingests). Users do care about upload/download performance; Theo shares a detailed personal anecdote about spending a month working with Frame.io support to fix throttled speeds, nearly switching to Dropbox. Upload Thing's paid inf... [12]

### q050 PASS

- Prompt: What are the most optimistic views in my library?
- Class: `tone_or_style_inference`
- Status: `Completed`
- Score: `3`
- Sources: `8`
- Failure: `-`
- Source videos: GlazeGPT got rolled back (4o update gone wrong) | I don’t really use libraries anymore | I ranked every vibe coding app | React is killing the web | So close to Opus at 1/10th the price (GLM-4.7 and Minimax M2.1 showdown) | Which browser should you use right now? | Why is everyone so unhappy with JavaScript?

#### Answer

Retrieved evidence for: What are the most optimistic views in my library?

The answer model is unavailable, so this fallback lists the highest-ranked saved excerpts instead of synthesizing beyond them.

From the available evidence, these excerpts support only a tentative reading rather than a definitive judgment.

1. React is killing the web - Theo - t3․gg: in Chrome that have been made around how yielding occurs during these asynchronous workloads. Suspense lets you break the page into loadable chunks. Transitions let you prioritize user input and show immediate optimistic updates. Activity lets you defer hidden content. View transition will let you coordinate animations for batches of UI. Suspense list will let you specify the order that UI loads in. all with simple d... [1]
2. So close to Opus at 1/10th the price (GLM-4.7 and Minimax M2.1 showdown) - Theo - t3․gg: ely 1/60th of Opus 4.1's original pricing). GLM 4.7 excels at UI/design tasks and visual outputs, while MiniMax M2.1 excels at long-running coding tasks, planning, and sustained multi-file changes. Both models are open-weight (M2.1 weights expected to drop around Christmas), runnable on consumer hardware, and represent a major shift in what's possible for budget-conscious developers. Overview This video provides an i... [2]
3. GlazeGPT got rolled back (4o update gone wrong) - Theo - t3․gg: r than providing grounded responses. Unlike the pre-internet era, modern technology and AI can validate and amplify fringe beliefs and delusions without the social checks that previously existed. The speaker shares an example where a friend's conversation with ChatGPT spiraled into nonsense involving made-up scientific terms because the model simply reinforced whatever context was provided. OpenAI's response includes... [3]
4. I don’t really use libraries anymore - Theo - t3․gg / Overview: This video explores how AI-assisted development is fundamentally changing the role and utility of software libraries. The speaker, a developer who has built many projects using various libraries, shares his evolving perspective on dependency management in an era where AI can generate implementations. He discusses his personal experience removing libraries like Tkumi from projects, examines industry examples like Anti... [4]
5. Why is everyone so unhappy with JavaScript? - Theo - t3․gg / Key Points: Most want native JS types to resemble TypeScript. Developer Happiness Trends **The disturbing pattern**: Everything is moving left toward negative sentiment—even tools like Vite and frameworks that didn't have major changes. Angular is the only thing that moved right (more positive) because it was already so far left and made improvements. **Web tech happiness**: Hasn't moved in five years. **JS happiness**: Slightly... [5]
6. I ranked every vibe coding app - Theo - t3․gg: TL;DR The video ranks "vibe coding" tools (AI coding apps for non-developers) on a tier list from F to S, evaluating them primarily on how little code they show and how little technical knowledge they require Top-tier vibe coding tools (S/A tier): V0 (S tier for best integration experience), Trey (A tier for solo mode that hides code), Claude Code (A tier for being the first developer-focused vibe coding tool) Mid-ti... [6]
7. Why is everyone so unhappy with JavaScript? - Theo - t3․gg: do about this the library tier list everyone loves their tier lists V and vest are still really high up playwright's killing it play right's really good Astro at that 94% Mark still lower than it was but still really good overall pnpm holding strong up here we just saw spelet really negative right now how is it there if the negative numbers were what we just saw does it explain how this by the retention ratio okay so... [7]
8. Which browser should you use right now? - Theo - t3․gg: TL;DR Chrome/Chromium has had a massively positive impact on web standards and is technically the best implementation, but Google's monopolistic tendencies show in forced AI integrations like Gemini. Manifest V3 was the right call for security (preventing malware), not an anti-ad-blocker move, though ad-blocking is now slightly worse in Chrome. Brave is strongly criticized for buggy UX, breaking websites, aggressive ... [8]

### q051 PASS

- Prompt: What advice shows up repeatedly across different videos?
- Class: `topic_aggregation`
- Status: `Completed`
- Score: `3`
- Sources: `3`
- Failure: `-`
- Source videos: AI is ruining the job market | GPT-5.1 is built for normies | I hate that this is still happening

#### Answer

Retrieved evidence for: What advice shows up repeatedly across different videos?

The answer model is unavailable, so this fallback lists the highest-ranked saved excerpts instead of synthesizing beyond them.

Comparison frame: both the listed excerpts and their source videos are relevant candidates, while the exact similarities, differences, or counterarguments should be checked against the cited text below.

1. GPT-5.1 is built for normies - Theo - t3․gg: changed in settings rather than per-thread, and noted frequent bugs where chat threads disappear from the sidebar. Comparison Testing Against Other Models **Relaxation Tips Test**: When asked for stress relief advice, GPT-5.1 and GPT-5 both used bullet points and emojis heavily. Kimmy K2 offered similar techniques but included lifestyle adjustments and therapist recommendations that GPT models omitted. **Java Defense... [1]
2. AI is ruining the job market - Theo - t3․gg: the most part already knew to do this and I'm scared if I give this advice that a lot of people who wouldn't have figured this out that would have just sent me the like life essay in DMs that I get dozens of a day that that person is now going to be more annoying because now they can disguise themselves as a good intended person. It's similar to the open source contribution thing I just did a video on. Like if it loo... [2]
3. I hate that this is still happening - Theo - t3․gg: up here. Lionus doesn't like open source because he's a god dev. He likes open source because he had to go through this whole process himself as a bad dev becoming a good dev over time. And during his process getting there, he grew a fondness to open source because the closed source systems he was using caused him a lot of problems. He also earned his right to be an [ __ ] because of all of the problems he encountere... [3]

### q052 PASS

- Prompt: What are the top recommendations given in this channel?
- Class: `recommendation`
- Status: `Completed`
- Score: `3`
- Sources: `12`
- Failure: `-`
- Source videos: A.I. Backlash Turns Violent + Kara Swisher on Healthmaxxing + The Zuck Bot Is Coming | Anthropic’s Cybersecurity Shock Wave + Ronan Farrow and Andrew Marantz on Their Sam Altman Investigation + One Good Thing | What’s a Hard Fork?

#### Answer

Retrieved evidence for: @{Hard Fork} What are the top recommendations given in this channel?

The answer model is unavailable, so this fallback lists the highest-ranked saved excerpts instead of synthesizing beyond them.

1. What’s a Hard Fork? - Hard Fork / At a glance: The video title asks "What’s a Hard Fork?", but the transcript contains no substantive content on this topic. The provided transcript is solely an automated speech recognition (ASR) system metadata note. The text indicates it originated from a local OpenAI-compatible ASR endpoint, not from official RSS show notes. [1]
2. Anthropic’s Cybersecurity Shock Wave + Ronan Farrow and Andrew Marantz on Their Sam Altman Investigation + One Good Thing - Hard Fork: be beautiful wherever you happen to be. Wow. They'll send you an umbrella reminder if it's going to precipitate in the next 12 hours and they'll send you a sunscreen alert when the UV index is high. But I'm saving my last two favorites for the end. Number one, they will send you an alert when the Aurora Borealis may be visible where you are. That's beautiful. I haven't gotten that notification yet, but I wake up ever... [2]
3. What’s a Hard Fork? - Hard Fork / Key Points: Transcript Metadata**: The only content in the transcript is a procedural note indicating it is a "smoke transcript" generated by a local OpenAI-compatible ASR endpoint, explicitly stating it did not come from RSS show notes. No definitions, examples, or explanations of a "hard fork" are present. [3]
4. What’s a Hard Fork? - Hard Fork / Overview: The video is titled "What’s a Hard Fork?", suggesting an educational focus on blockchain or software development concepts. However, the actual transcript provides no information on this subject. It consists entirely of an ASR metadata placeholder stating the text was generated by a local OpenAI-compatible ASR endpoint rather than sourced from RSS show notes. [4]
5. What’s a Hard Fork? - Hard Fork / Takeaways: The intended educational content regarding what a hard fork is could not be summarized, as the transcript contains only ASR metadata and no actual discussion. [5]
6. Anthropic’s Cybersecurity Shock Wave + Ronan Farrow and Andrew Marantz on Their Sam Altman Investigation + One Good Thing - Hard Fork / At a glance: Anthropic announced "Claude Mythos Preview," a highly capable new AI model withheld from the public due to severe cybersecurity risks, instead providing access to a defensive tech consortium. The model can autonomously find zero-day exploits in critical open-source infrastructure (e.g., OpenBSD, FFmpeg) that have evaded human researchers and automated tools for decades. The hosts argue this is not a marketing stunt, ... [6]
7. Anthropic’s Cybersecurity Shock Wave + Ronan Farrow and Andrew Marantz on Their Sam Altman Investigation + One Good Thing - Hard Fork: on hard fork and we'll have some updated guidance. But I asked my friend, do you have a password manager and do you reuse passwords for the same thing? And she said, you know, I've never really been able to get one of those password managers to work for me and I do sometimes reuse my passwords. So I said, like, look, if you're looking for something that you can do, just make sure that you have done your basic online ... [7]
8. Anthropic’s Cybersecurity Shock Wave + Ronan Farrow and Andrew Marantz on Their Sam Altman Investigation + One Good Thing - Hard Fork: are perfectly content with a free weather app on their phone. That is fine for you. But as somebody who loves cool things, new ideas, people having fun. I just wanted to shout out, act me weather because I think it's a really cool thing. Now, what is the likelihood that this app will be purchased by Apple and then shut down? I mean, if that happens, I hope these guys get paid again because somebody has to move the we... [8]
9. A.I. Backlash Turns Violent + Kara Swisher on Healthmaxxing + The Zuck Bot Is Coming - Hard Fork: I'm Dane Bruggler. I cover the NFL draft for the Athletic. Our draft guide picked up the name "The Beast" because of the crazy amount of information that's included. I'm looking at thousands of players putting together hundreds of scouting reports. I've been covering this year's draft since last year's draft. There is a lot in the beast that you simply can't find anywhere else. This is the kind of in-depth, unique jo... [9]
10. A.I. Backlash Turns Violent + Kara Swisher on Healthmaxxing + The Zuck Bot Is Coming - Hard Fork: about it and recently, more recently has admitted it. And so a lot of them were using it for optimization, not depression, but optimization. And this guy was using it for new ideas in his entrepreneurial journey. So... And did you have a lot of new ideas on ketamine when you tried that? I had none. I thought only about you. Kasey naked is what I thought. No. No, have you ever, either of you used it? I have tried keta... [10]
11. A.I. Backlash Turns Violent + Kara Swisher on Healthmaxxing + The Zuck Bot Is Coming - Hard Fork: called Cara Swisher wants to live forever. Whether she does, in fact, seek immortality is a point of contention as you will hear in the interview. But during this series, Kevin and I were able to watch the first interview episodes and in it, she tries a lot of the things that the rich and powerful are trying as part of their quest to become immortal. >> Yes. So this is a big topic. Obviously, people in tech are very ... [11]
12. A.I. Backlash Turns Violent + Kara Swisher on Healthmaxxing + The Zuck Bot Is Coming - Hard Fork: bored. I mean, aloneness is a very difficult emotion for a podcaster. It is. It is. Yeah. It's aloneness. It's interesting. I feel like the sort of psychedelic. You haven't said if you've taken it, Kevin. I'm on the advice of council. I'm going to respectfully decline the answer. Kevin works at the New York Times. They have opinions about these things. Yes. It's interesting. I feel like there's been a shift in Silico... [12]

### q053 PASS

- Prompt: Which videos contain step-by-step instructions?
- Class: `direct_lookup`
- Status: `Completed`
- Score: `3`
- Sources: `4`
- Failure: `-`
- Source videos: Can we put Rust in Angular to make it faster? WASM deep dive | My current stack | We need to talk about Ralph

#### Answer

Retrieved evidence for: Which videos contain step-by-step instructions?

The answer model is unavailable, so this fallback lists the highest-ranked saved excerpts instead of synthesizing beyond them.

1. Can we put Rust in Angular to make it faster? WASM deep dive - Theo - t3․gg / Key Points: references a prior video explaining why WASM is "overhyped" for general web development but acknowledges this use case is correct: WASM is ideal for code that takes input and produces output rapidly. Tutorial Walkthrough: Angular + Rust Setup The article guides users to create an Angular workspace using NX (a monorepo tool popular in the Angular ecosystem), install Rust, and use `wasm-pack`—a Rust crate for packagin.... [1]
2. My current stack - Theo - t3․gg: be able to edit without having to build all the UI stuff for it so I started with Google Sheets 2 hours into trying to get it to off my roommate in CTO Mark comes over and laughs at me and says I warned you Theo don't do this so I caved and moved to notion which has actually been really nice the notion API is totally fine so yeah Google Sheets isn't the simple option and that's not because Google Sheets isn't simple ... [2]
3. We need to talk about Ralph - Theo - t3․gg / Key Points: implementation plan.md`). Key instruction: "pick the most important thing to do, not go through this in order." The model chooses what it thinks is most important, completes it, and the markdown file gets updated when tasks are done. The prompt should specify studying a spec file and implementation plan before starting work, ensuring "the right context at the start." Prompt Structure and File Components A good prompt... [3]
4. Can we put Rust in Angular to make it faster? WASM deep dive - Theo - t3․gg: to implement similar functionality in a Next.js environment, documenting the significant hurdles encountered with build systems, module initialization, and type definitions. The video serves as both a tutorial walkthrough and a candid record of the friction involved in setting up Rust-WASM in modern JavaScript applications. Key Points Context and Premise: When WebAssembly is Appropriate The article and host agree tha... [4]

### q054 PASS

- Prompt: Which videos are more conceptual than practical?
- Class: `tone_or_style_inference`
- Status: `Completed`
- Score: `3`
- Sources: `11`
- Failure: `-`
- Source videos: Claude Code's latest update is really cool (when it works...) | Claude's new Cursor killer just dropped | I might have a new favorite state manager... | It’s actually over now | The Actual Dumbest Thing About Try/Catch | The Best Model For Frontend Design Is... | This awesome CSS feature is blocked by drama (Google and Apple can't agree) | This is good, actually | What happened to me?

#### Answer

Retrieved evidence for: Which videos are more conceptual than practical?

The answer model is unavailable, so this fallback lists the highest-ranked saved excerpts instead of synthesizing beyond them.

From the available evidence, these excerpts support only a tentative reading rather than a definitive judgment.

1. This awesome CSS feature is blocked by drama (Google and Apple can't agree) - Theo - t3․gg: read in a second but I want to start with Adam argy's comments because he's he's Deep In The Weeds here also works at Google has a lot of things to to say in all of these and I'm excited for his thoughts here are his points on why he doesn't like grid level three as the way to do masonry point one a masonry layout isn't a grid there is no shared row lines only columns so it has to ignore all sorts of syntax to accomm... [1]
2. I might have a new favorite state manager... - Theo - t3․gg / Key Points: is a hack. **Cleaner Selectors**: In Zustand, you have to select functions off the store as if they were values, even though they never change. This is conceptually confusing—functions should just be callable, not selected. Event-Driven Architecture **Store.send API**: XState Store uses `store.send({ type: 'increasePopulation', by: 10 })` to trigger transitions. This is fundamentally different from Zustand's direct f... [2]
3. What happened to me? - Theo - t3․gg: this can change the same way it changed here. If I start doing more of these types of videos and they perform better than expected, I'll lean more into this. For example, the logging video, we screwed up the export initially and the first version that went up was too short and had most of the content missing. So, we had to re-upload it, which destroys the video performance because a lot of people already saw it, so t... [3]
4. What happened to me? - Theo - t3․gg: audience didn't like the video, that's why it didn't perform. Oh yeah, maybe I should make a video the audience likes. It really does come down to that. There are layers to this, like is the video clickable? Does it start in a way that's entertaining and interesting enough that you continue to watch from there? All of these pieces are important, but I'm not blocked by the algorithm for making certain content. In fact... [4]
5. It’s actually over now - Theo - t3․gg: started in a garage doing like door-to-door sales and showing off to computer nerds. You don't start with the fancy marketing video. You start by being real humans. And they tried a little too hard to do the marketing thing. And what's really funny is I talked to a lot of these earlier stage companies and they want to do their own elaborate YouTube stuff. Both because they see me as a YouTuber. They're like, "Hey, ho... [5]
6. The Actual Dumbest Thing About Try/Catch - Theo - t3․gg: I'll be honest error handling in JavaScript kind of sucks I know hot take but that's what we're here for right seriously though try catch is it's a disaster there's a lot of subtlety for the things that are wrong with it but there's one particular piece that I don't think it's talked about enough and I saw a tweet that inspired me to make this video the piece that we're talking about here is the scoping huge shout ou... [6]
7. Claude's new Cursor killer just dropped - Theo - t3․gg: Chat, Co-work, and Code into a single application, replacing the CLI. The reviewer finds the new desktop app severely flawed, citing numerous UX bugs, missing basic features, and poor performance, arguing it barely improves upon the "trash" CLI. Compared to alternatives like Codex and the reviewer's own project (T3 Code), the Claude app lacks basic functionalities like proper copy buttons, project management, and re.... [7]
8. The Best Model For Frontend Design Is... - Theo - t3․gg: from Claude Code's GitHub that instructs models to avoid generic AI aesthetics and create intentional, varied designs. When using the skill, Opus 4.5 dramatically improves and surpasses other models, producing more malleable designs that better respond to iteration and refinement. GPT 5.2 often ignores instructions to avoid the skill due to system-level mandates, making fair comparisons difficult. Gemini's CLI is des... [8]
9. This is good, actually - Theo - t3․gg: problems but require learning different syntax and mental models for steps, tasks, and event-based execution. The controversy centers on whether directives (magic strings like `"use server"`, `"use workflow"`) are good API design or if function wrappers (like TanStack Start's `server()` function) would be better for type safety, composability, and clarity. Tanner Lindsley argues directives aren't type-safe, extensibl... [9]
10. I might have a new favorite state manager... - Theo - t3․gg: tential new favorite state manager, positioned as a middle ground between Zustand's simplicity and XState's robustness. Key advantages over Zustand include automatic TypeScript inference without complex middleware, strict separation between state (context) and actions (transitions), and built-in framework-agnostic architecture. The `store.send` event-driven API is debated, with some preferring direct setter methods f... [10]
11. Claude Code's latest update is really cool (when it works...) - Theo - t3․gg: TL;DR Claude Code's "Christmas update" introduces async sub-agents and context compression, features that are genuinely innovative but hampered by bugs and high costs in practice. The async sub-agent architecture allows the main agent to spin up background tasks that run in parallel without blocking—described as similar to React's Suspense pattern for blocking vs. non-blocking operations. The video documents numerous... [11]

### q055 PASS

- Prompt: Which videos are the most technical?
- Class: `tone_or_style_inference`
- Status: `Completed`
- Score: `3`
- Sources: `12`
- Failure: `-`
- Source videos: Defending a disaster (modern frontend development rant) | How did we get here? (A rant about Javascript runtimes) | JavaScript runs on literally everything now | OpenAI’s TikTok Clone Is Interesting… | What happened to me? | What happens now? | “Just Use HTML”

#### Answer

Retrieved evidence for: Which videos are the most technical?

The answer model is unavailable, so this fallback lists the highest-ranked saved excerpts instead of synthesizing beyond them.

From the available evidence, these excerpts support only a tentative reading rather than a definitive judgment.

1. What happened to me? - Theo - t3․gg: this can change the same way it changed here. If I start doing more of these types of videos and they perform better than expected, I'll lean more into this. For example, the logging video, we screwed up the export initially and the first version that went up was too short and had most of the content missing. So, we had to re-upload it, which destroys the video performance because a lot of people already saw it, so t... [1]
2. OpenAI’s TikTok Clone Is Interesting… - Theo - t3․gg / Key Points: Model Architecture and Technical Implementation Underlying Generation Method**: The creator theorizes that Sora 2 isn't just a single video model but rather an LLM generating screenplays and plans that command other models to generate video pieces, which are then stitched together. This explains how videos exceed the typical 5-second limit seen in other video models. The model appears to generate audio first, then cr... [2]
3. What happened to me? - Theo - t3․gg: of my community, the people hanging out in Twitch chat right now who have been there since day one, watched this whole thing happen, many of which joined me on the same journey. So, knowing all of this, I want to talk a bit about how I choose a topic for a video because another one of the questions I get all of the time is, "What is your dream video that you would do if the algorithm wouldn't kill it?" A question I g... [3]
4. Defending a disaster (modern frontend development rant) - Theo - t3․gg: and second most recruiting agencies are garbage you shouldn't need me to tell you that we all get spammed with them every day they have no idea what they're doing they're not even technical not only is g2i technical they are some of the most technical this is the crew that runs react Miami which is my favorite react conference and it's not even close the amount of fun I had there last year was unbelievable and I will... [4]
5. “Just Use HTML” - Theo - t3․gg: [ __ ] websites, but I know [ __ ] better than to pat out this video any [ __ ] more. So, I'm just going to be [ __ ] done. Let me know what you think. Until next time, [ __ ] [5]
6. OpenAI’s TikTok Clone Is Interesting… - Theo - t3․gg / TL;DR: OpenAI released a new Sora mobile app that combines AI video generation with a TikTok-style social feed, featuring character consistency through a "Cameo" feature and longer-form videos with audio-video synchronization. The model demonstrates notable technical improvements including music generation with hooks and decent delivery, plus J-cut/L-cut editing techniques, though video generation remains expensive with a 5... [6]
7. JavaScript runs on literally everything now - Theo - t3․gg: and even parts of the operating system are being moved to JavaScript and react native at least I'm safe on my Mac and on my PlayStation and my other consoles right well obviously the Xbox is running react native too I hope that's kind of obvious because react native Windows Xbox also kind of Windows what might surprise you is another console it's kind of a poorly kept secret but the PlayStation 5 uses react native 2.... [7]
8. How did we get here? (A rant about Javascript runtimes) - Theo - t3․gg / Key Points: GJS, MUJS, JScript, jsdb, njs, TeX, bear, other low.js variants [8]
9. What happened to me? - Theo - t3․gg: audience didn't like the video, that's why it didn't perform. Oh yeah, maybe I should make a video the audience likes. It really does come down to that. There are layers to this, like is the video clickable? Does it start in a way that's entertaining and interesting enough that you continue to watch from there? All of these pieces are important, but I'm not blocked by the algorithm for making certain content. In fact... [9]
10. What happens now? - Theo - t3․gg / Overview: This video is a deep dive response to an article by Chris Gregory about how AI tools like Claude Code and Cursor are fundamentally changing software development. The speaker explores the thesis that while code has become cheap to produce, software remains expensive because the real costs — problem understanding, maintenance, distribution, and architecture — haven't changed. The discussion covers the rise of "disposab... [10]
11. What happened to me? - Theo - t3․gg: have gotten 5k plays. A out of 10 would have gotten 40k plays. a 10 out of 10 would have gotten like k plays. That was the range before. The weird thing that's happened is due to the massive change in who is watching my channel and the interest of the people who are watching is the gap between these has gotten massive. Even a six, seven or eight out of 10 topic is going to perform significantly worse. This has been w... [11]
12. OpenAI’s TikTok Clone Is Interesting… - Theo - t3․gg: TL;DR OpenAI released a new Sora mobile app that combines AI video generation with a TikTok-style social feed, featuring character consistency through a "Cameo" feature and longer-form videos with audio-video synchronization. The model demonstrates notable technical improvements including music generation with hooks and decent delivery, plus J-cut/L-cut editing techniques, though video generation remains expensive wi... [12]

### q056 PASS

- Prompt: Which videos are best for a beginner?
- Class: `recommendation`
- Status: `Completed`
- Score: `3`
- Sources: `6`
- Failure: `-`
- Source videos: Cloudflare and Vercel can't stop fighting | I hate that this is still happening | I've waited 6 years for this... | Pro tips for picking the right stack | The Windsurf situation is pretty wild (RIP $3 billion?) | The painful truth about startups (my story)

#### Answer

Retrieved evidence for: Which videos are best for a beginner?

The answer model is unavailable, so this fallback lists the highest-ranked saved excerpts instead of synthesizing beyond them.

1. The Windsurf situation is pretty wild (RIP $3 billion?) - Theo - t3․gg / Key Points: etc.). When Cursor gained traction in the "middle market" between beginners and enterprise users, Windsurf pivoted to compete there. The video describes a spectrum: on one end are "vibe coders" and new developers (who care about quick generation, CSS aesthetics); on the other end are enterprise users (who care about TDD, error reporting, JetBrains integration). Cursor won the middle ground while Windsurf was stretche... [1]
2. I hate that this is still happening - Theo - t3․gg: pdate readme.momd. Update readme. Update readme. Update readme. Update readme. Update readme. Update readme. I'm going to go actually insane. For those who haven't been around for a long time, I'm Theo. I make videos about software dev stuff. I care a lot about open source, which is why this in particular makes me really mad. It makes me so mad that I made a long video about this in the past that has quite a spicy ti... [2]
3. Pro tips for picking the right stack - Theo - t3․gg: TL;DR When deciding which framework to start with, choose what is popular. If you do not like the popular option, simply pick what you like instead. Popular options are popular for a reason: they work, even if they are not perfect for every problem. Beginners who are not experienced enough to know the right solution should default to the popular choice. Getting stuck on this decision is unnecessary because the specif... [3]
4. I've waited 6 years for this... - Theo - t3․gg: TL;DR The creator has advocated for killing Create React App (CRA) for 6 years, calling it objectively behind and harmful to beginners who unknowingly use it. CRA was historically important because it solved React's complex tooling setup (webpack, Babel, ESLint), introduced error overlays in-browser, and pioneered hot reloading/fast refresh features. CRA became harmful because it lacks deprecation warnings, leaving b... [4]
5. Cloudflare and Vercel can't stop fighting - Theo - t3․gg: ike, but because people hated Versel so much, this new narrative formed of Verscell pays all of these influencers to support them and shill them and pretend they're good when they're not and trick all of these beginners into using Versell when they shouldn't need it. That stereotype went so far that it was actually affecting brand sentiment both for them and for me. So, I decided to end it. I was just tired. If I'm g... [5]
6. The painful truth about startups (my story) - Theo - t3․gg: d get 10K subs. I'd get a couple hundred maybe a few thousand views a video on average and that would be awesome. I never would have imagined I hit 100k subs in my first year. Never would have imagined that my videos would be as successful as they are. That we'd find this massive set of people who would be interested in watching more senior content. When I would tell people that I made YouTube videos about software,.... [6]

### q057 PASS

- Prompt: Which videos are best for an advanced viewer?
- Class: `recommendation`
- Status: `Completed`
- Score: `3`
- Sources: `4`
- Failure: `-`
- Source videos: Developers are way too sensitive. | I can't believe he was right. | What happened to me?

#### Answer

Retrieved evidence for: Which videos are best for an advanced viewer?

The answer model is unavailable, so this fallback lists the highest-ranked saved excerpts instead of synthesizing beyond them.

1. Developers are way too sensitive. - Theo - t3․gg / Overview: The video explores why software engineers have become increasingly fragile and resistant to progress despite unprecedented technological advancement. The speaker argues that engineers face a unique paradox: their job requires them to feel stupid regularly while human nature drives them to avoid that feeling, creating insecurity and defensive tribalism. Through personal anecdotes, statistical analysis of YouTube comme... [1]
2. Developers are way too sensitive. - Theo - t3․gg: TL;DR Engineers are experiencing unprecedented progress in their field but responding with unusual levels of frustration and disdain toward change. The loud minority of online commenters (roughly 0.6% of viewers) disproportionately represents negativity and insecurity, while average developers remain silent. Engineering requires feeling "stupid" regularly, which conflicts with human nature to avoid discomfort, leadin... [2]
3. What happened to me? - Theo - t3․gg: this can change the same way it changed here. If I start doing more of these types of videos and they perform better than expected, I'll lean more into this. For example, the logging video, we screwed up the export initially and the first version that went up was too short and had most of the content missing. So, we had to re-upload it, which destroys the video performance because a lot of people already saw it, so t... [3]
4. I can't believe he was right. - Theo - t3․gg: s) using Claude Code and Opus, without opening a traditional editor. Google reports 25%+ of code is AI-written; Microsoft reports ~30%; 32% of senior devs say at least half their code comes from AI—senior devs are adopting these tools fastest. The role of developers is shifting from writing code to reviewing and orchestrating AI-generated code, similar to how engineers transition to management roles. AI-generated cod... [4]

### q058 PASS

- Prompt: What should I watch if I only have five minutes?
- Class: `recommendation`
- Status: `Completed`
- Score: `3`
- Sources: `6`
- Failure: `-`
- Source videos: Hacking websites with your company name | It was a wild year for CSS | Open source is dying | The Future of TypeScript | The Secret Language Scaling WhatsApp and Discord | Watch this if you hate React Server Components

#### Answer

Retrieved evidence for: What should I watch if I only have five minutes?

The answer model is unavailable, so this fallback lists the highest-ranked saved excerpts instead of synthesizing beyond them.

1. The Secret Language Scaling WhatsApp and Discord - Theo - t3․gg: the live video and chat with the host not only that they can also connect their smartwatch to the application the application will show the synchronized recipe instructions as a timer on their watch by the time I started building the project I had to choose the most cost effective Tech stack to build it most important traits of the tech were the following need to be easy to build real-time features it should provide.... [1]
2. Open source is dying - Theo - t3․gg: have bad intuition sometimes. That was a bad intuition on my part. I can't imagine many projects get there as quickly as we did with that brutal ratio. But still, yeah, it is what it is. And if only PRs were the biggest issue we had with this type of open-source stuff nowadays. Sadly, there's another bigger problem, and it kind of touches on this classic post on Reddit, the I don't give an f about the effing code. I.... [2]
3. It was a wild year for CSS - Theo - t3․gg: ations that block each other via CSS has so much promise. We've also now confirmed that a lot of these are Gemini generated. So, it's possible a lot of the images and even some of the text is AI slop. Next, we have the scroll into view container. Sometimes scrolling only the nearest ancestor scroller is all you want. This actually would be very useful for T3 chat. This is for nested scroll containers because if you h... [3]
4. The Future of TypeScript - Theo - t3․gg: locations we needed to watch under the d--watch and editor scenarios. In a new project, lib replacement never does anything until other explicit configuration takes place. So, it makes us turn off by default. Good [ __ ] Root dur now defaults to the current dur period. Previously, we didn't specify one. It was inferred based on the common directory of all non-declaration input files. This o menu is impossible to know... [4]
5. Watch this if you hate React Server Components - Theo - t3․gg: modified but recognizable tree of objects that represents what should be rendered on the page. This also does mean you're sending the data twice. You're sending it as HTML and you're sending it in this embedded JavaScript tag. And when people say that server components don't need a server, this is what they mean. You can at build time generate the HTML that isn't going to be dynamic and have a bunch of different HTML... [5]
6. Hacking websites with your company name - Theo - t3․gg: saving it for later, but it will be in the description if you want to give it a watch yourself. This isn't even the only license plate one. Here's somebody trying to inject into speeding cameras. What's going on here? Drop database tabless. Should be table. What's the here? Oh, tablets translates to plates in Polish. Clever. We don't know if this ever actually worked. It's just a meme. Still funny. I love stuff like.... [6]

### q059 PASS

- Prompt: What should I watch if I want the deepest dive?
- Class: `recommendation`
- Status: `Completed`
- Score: `3`
- Sources: `12`
- Failure: `-`
- Source videos: Everything you need to know about GPT-5 (+ mini and nano) | Getting a Dev Job in 2025 | My current stack | OpenAI is lying | React Doesn't Scale | Sonnet 4.5 is the best coding model in the world | The Windsurf situation is pretty wild (RIP $3 billion?) | Vercel Finally Caught Up | Watch this if you know HTML | You're logging wrong

#### Answer

Retrieved evidence for: What should I watch if I want the deepest dive?

The answer model is unavailable, so this fallback lists the highest-ranked saved excerpts instead of synthesizing beyond them.

1. OpenAI is lying - Theo - t3․gg: TL;DR OpenAI published an article titled "Designing delightful frontends with GPT 5.4" claiming their model can produce production-ready frontends, but the examples shown are objectively poor quality with broken layouts, excessive cards, and generic design patterns. Comprehensive benchmark testing comparing GPT 5.4 against Kimmy K2.5, Claude Opus, and Gemini demonstrates that all competing models produce significantl... [1]
2. You're logging wrong - Theo - t3․gg: don't have the feature flags added. This is a really cool example of how to do this. And the harsh reality is no matter how good you get at this, you will still miss things. you will have some outage where you can't figure out what's going wrong because you didn't log one of the fields you need and now you can go add that log and hope that the next time it happens you're good because you're not adding a log. You're a... [2]
3. Sonnet 4.5 is the best coding model in the world - Theo - t3․gg / Overview: This video provides a comprehensive deep-dive review and analysis of Anthropic's surprise release of Claude Sonnet 4.5. The reviewer spent a full day analyzing the system card, running custom benchmarks, and testing the model on practical coding tasks. The video explores the competitive landscape between Anthropic and OpenAI, scrutinizes Anthropic's safety and alignment claims, details the new features and SDK update... [3]
4. React Doesn't Scale - Theo - t3․gg: TL;DR A viral Reddit post claims React codebases become disorganized messes at scale, with very few senior engineers truly understanding the library; the video analyzes these claims in depth. The presenter argues most React problems stem from developer inexperience and wrong mental models (especially OOP/class-based thinking), not the framework itself. Key React issues discussed: misuse of `useEffect`, `useState`, `u... [4]
5. Getting a Dev Job in 2025 - Theo - t3․gg: TL;DR The tech job market is significantly harder now due to massive layoffs (over 400,000 tech roles lost in 2023-2024) and the rise of AI tools making junior engineers less essential. Hiring has shifted from a strategy of betting on junior potential to prioritizing experienced engineers and trusted referrals due to an overwhelming flood of low-quality, AI-generated applications. Trust has become the most valuable c... [5]
6. Vercel Finally Caught Up - Theo - t3․gg / Key Points: marginal [6]
7. The Windsurf situation is pretty wild (RIP $3 billion?) - Theo - t3․gg: suits from shareholders who would have otherwise been left with worthless equity. The "poaching war" context: companies like Meta, Google, and OpenAI are aggressively hiring AI talent and acquiring teams, with examples including Meta's $14.3 billion investment in Scale AI and poaching of Claude Code creators to Cursor. Windsurf will now focus on enterprise customers (returning to its Codium roots), while Google gains... [7]
8. Everything you need to know about GPT-5 (+ mini and nano) - Theo - t3․gg / Overview: This video provides a comprehensive deep-dive into OpenAI's newly released GPT-5 model family, covering pricing, benchmarks, capabilities, and practical implications for developers. The presenter, who had early access to the models, compares them against competitors like Grok 4, Claude, and Gemini across multiple dimensions including cost efficiency, reasoning ability, and safety features. The discussion includes det... [8]
9. My current stack - Theo - t3․gg / Overview: This video provides an extensive, chaotic walkthrough of Theo's current technology stack across multiple projects, including pick thing, T3 chat, marker thing, and unduck. Rather than presenting a simple template to copy, Theo explains the reasoning behind each decision, documents the failures and rewrites he went through, and warns viewers about the complexity costs of various approaches. The core philosophy through... [9]
10. Sonnet 4.5 is the best coding model in the world - Theo - t3․gg: Practical testing shows Sonnet 4.5 is faster and better at tedious, multi-step coding tasks than previous Claude versions, but remains weak at complex UI work compared to GPT-5. Overview This video provides a comprehensive deep-dive review and analysis of Anthropic's surprise release of Claude Sonnet 4.5. The reviewer spent a full day analyzing the system card, running custom benchmarks, and testing the model on pra.... [10]
11. Watch this if you know HTML - Theo - t3․gg / Overview: This video provides an in-depth technical analysis of the evolution of web application rendering strategies, moving from traditional Multi-Page Apps (MPAs) and Single Page Apps (SPAs) to modern hybrid models. The speaker diagrams the data flow and trade-offs of each approach, highlighting the specific problems each model solves and the new complexities it introduces. Key themes include the tension between server-side... [11]
12. Watch this if you know HTML - Theo - t3․gg / Key Points: Classic Multi-Page App (MPA) Architecture **Model Behavior**: The user requests a page (e.g., `example.com`), and the server returns the full HTML for that page. When the user clicks a link to `example.com/about`, the browser makes a new request, and the server sends the full HTML for the about page. **Implementation**: The server may serve static files (e.g., `index.html`) or dynamically generate HTML via a function... [12]

### q060 PASS

- Prompt: Which videos are most relevant to my current interests in search and AI?
- Class: `meta_learning_or_next_step`
- Status: `Completed`
- Score: `3`
- Sources: `10`
- Failure: `-`
- Source videos: "AI Startups" are over done (finally) | AI images just got dangerously good (RIP diffusion??) | Anthropic is lying to us. | Anthropic is trying SO hard to fix MCP... | Did Meta Really Fake Benchmarks? | Prisma is removing Rust? | What happened to me? | Which browser should you use right now?

#### Answer

Retrieved evidence for: Which videos are most relevant to my current interests in search and AI?

The answer model is unavailable, so this fallback lists the highest-ranked saved excerpts instead of synthesizing beyond them.

1. What happened to me? - Theo - t3․gg: of the time offline. And to honor the thing I always do on this channel, I'm going to film this video because it's a thing I want to talk about. And in the end, that's what my videos are. But hopefully the people who don't comment, which is over % of you, might have some of your concerns eased by this. My videos aren't based on what makes me money. My money is based on what's exciting to me, much like my topics are, ... [1]
2. Did Meta Really Fake Benchmarks? - Theo - t3․gg: dal is just it can handle different things. So things like text, images, video, audio. Multimodal means it can handle different things with just one model. Mixture of experts is the thing I'm talking about for most of the section where I talk about how the parameters are split across different things. Wanted to include this clarification rather than be wrong in the video. Anyways, their focus is on multimodal intelli... [2]
3. What happened to me? - Theo - t3․gg: this can change the same way it changed here. If I start doing more of these types of videos and they perform better than expected, I'll lean more into this. For example, the logging video, we screwed up the export initially and the first version that went up was too short and had most of the content missing. So, we had to re-upload it, which destroys the video performance because a lot of people already saw it, so t... [3]
4. "AI Startups" are over done (finally) - Theo - t3․gg: AI. How many devs are going to try out that [ __ ] thing? The answer is none. Because nobody is trying to replace themselves at their job. They're trying to make the boring parts less boring, the hard parts less hard, and the fun parts more relevant in their day-to-day lives. And that was what Copilot did well. So obviously why combinator companies had to adjust because too many of them were making these types of mis... [4]
5. What happened to me? - Theo - t3․gg: because a ton of other big open source projects are using it from Post Hog to Mastra to Nvidia Storybook Raycast and many more. Let's pick a Raycast one. I love Raycast. Here there was a rough case where custom npx path could have come in as an empty string which would have broken this check. And here we have a trim call that's going to handle that for you. Super easy to fix. Here's a PR somebody opened skate bench i... [5]
6. Which browser should you use right now? - Theo - t3․gg: rtical real estate. It has the worst vertical real estate of any browser I've used. Can't even fit the blog item on the page at the same zoom level. But here's what I wanted. I had done this post in June. What are your biggest frustrations with T3 chat right now? And I wanted to collect all of this data. So, let's do it. Summarize all of the replies to this post. Make sure you check all 500 plus of them. Seems like i... [6]
7. Prisma is removing Rust? - Theo - t3․gg: becomes huge they might change their mind cool personally if you're making a new database I think you should be focused on building a really good typescript orm yourselves something like eddb for example they are rethinking how to work with a relational DB where they're going more relational instead of less most nosql databases have less relational behaviors they have way more and technically they're built on top of.... [7]
8. Anthropic is trying SO hard to fix MCP... - Theo - t3․gg: accuracy issues, building MCB powered systems with multiple servers, or there's more than 10 tools available. It's less beneficial with a small tool library. Less than 10 tools is small now. Oh god. All tools are used frequently in every session and tool definitions are compact. These are when you wouldn't use it. Cool. And then we have programmatic tool calling. This is what we discussed in the previous video where.... [8]
9. Anthropic is lying to us. - Theo - t3․gg: ologizing and saying that this is legit. If you guys don't do that, I am going to just assume you're lying because every single thing is pointing to that. Whether or not this paragraph is true, it is no longer relevant and everything else you guys have said is either verifiably a lie or just makes no sense in the first place. and they seem to even know that too. They have this prompt that they're claiming was used fo... [9]
10. AI images just got dangerously good (RIP diffusion??) - Theo - t3․gg: there music? There better not be music. Every time. Every time. I just want to watch these videos without getting DMCA struck. They trained the model on a joint distribution of images and text, learning not just how images relate to language, but how they relate to each other. Combined with aggressive post training, the resulting model has surprising visual fluency capable of generating images that are useful, consis... [10]

### q061 PASS

- Prompt: What are the most important quotes from this transcript?
- Class: `direct_lookup`
- Status: `Completed`
- Score: `3`
- Sources: `12`
- Failure: `-`
- Source videos: Does Shopify Regret React Native? | Fine, I'll talk about the cursor drama | Open source is dying | OpenAI is TERRIFIED (this is absurd) | Prisma is removing Rust? | Saving the web from Javascript bloat | The Secret Language Scaling WhatsApp and Discord | We need to talk about Sonnet 4.6 | Well that was fun

#### Answer

Retrieved evidence for: +{Open source is dead now?} What are the most important quotes from this transcript?

The answer model is unavailable, so this fallback lists the highest-ranked saved excerpts instead of synthesizing beyond them.

1. Saving the web from Javascript bloat - Theo - t3․gg: e this stuff. I love these hard, thankless problems that nobody thinks about. Kind of nuts. Yeah, these people do actually exist. And to James, who seems to not know any of them according to his footnote, they are the ones you need to talk to if you want examples of people using these really old things that have this crazy set of weird dependencies that aren't needed in most places. It's all them. There's a bunch of.... [1]
2. Open source is dying - Theo - t3․gg: reason people maintain open- source software is because they care so much that you could argue it's too much. And when their job gets harder and harder because of all of this AI [ __ ] it gets more likely they give up. The reason they're here is that excitement. And if you can remind them of that, if you can be the excited thing that made them do this in the first place, you can make it feel so much more worth it. Yo... [2]
3. Open source is dying - Theo - t3․gg: Twitter DMs with the update with the encrypted stuff. But before then, the rate stayed nearly flat as I continued to get more relevant in the space. A simple two sentence, "Hey, I really appreciated this PR you shipped. I've been a fan of what you've been building for years. This library makes my life much better. Thank you." Those messages might seem small, but they can actually change your life. And I would not be ... [3]
4. Well that was fun - Theo - t3․gg: n fix some problems that affect other compute providers, but not us, like an issue that made trigonometry functions much slower on Versel. This post will dig into all the gory details. I am so so excited. It's important to note the original benchmark was not representative of billable CPU usage on Cloudflare, nor did the issues impact most typical workloads. Most of the disparity was an artifact of the specific bench... [4]
5. The Secret Language Scaling WhatsApp and Discord - Theo - t3․gg / Key Points: ck others. Core Feature 4: Performance Philosophy **Performance as a function of scale**: The speaker argues that while languages like Rust may execute individual functions faster, Erlang/Elixir solve the more important business problem—scaling horizontally. When you hit the limits of one server, BEAM makes adding a second server trivial. **Critique of Rust's concurrency model**: The speaker claims Rust "sucks at asy... [5]
6. Fine, I'll talk about the cursor drama - Theo - t3․gg: is its own token. The equals in the start of the class name is its own token. And then the colon quote at the end here is its own token to signify like the end of these things. It makes sense. If you compare this to how tokenization used to work with GPT3, it is broken up significantly more, including each of the spaces at the start here being its own token. And remember, each of these tokens is effectively creating.... [6]
7. Open source is dying - Theo - t3․gg: poorest. Really crippled the image I had in relation to your channel and content. I have been contributing to projects since before you were born. such an attitude, including some YouTube codes of yours that I've never received before. Feel free to block me. Mature. This is somebody who never wrote code before AI. Straight up. And I promise you, you were not contributing to code years ago, [ __ ] And this is the hot ... [7]
8. Does Shopify Regret React Native? - Theo - t3․gg: l. I like this quote here a lot. Instead of thinking about native or React Native, think about native and React Native. It's right in the name React Native. We found that you can save a ton of time by building most features just once using React Native and then leverage the native platform for the things it is best suited for. This is also why having native expertise is crucial. Okay, this is a big important point he... [8]
9. Open source is dying - Theo - t3․gg: feel awesome. Those messages make my goddamn day. Seeing somebody hit me up about how they were a line cook for a decade, learning code on the side, didn't feel like they could really do it, but watching my videos made them feel more like this crazy tech world we were in was a place they could fit, and now they have awesome tech jobs. My video isn't what did it. My channel isn't what did it. They did it. But that mes... [9]
10. We need to talk about Sonnet 4.6 - Theo - t3․gg: I DM any of these people, I will get a response because I have DM'd most of them and have gotten responses. And as Ryan said here, a bunch of these people immediately started engaging with him, interacting with him, and I have had the same experience. In my video about 5.3 Codeex, I had a section at the end where I just railed OpenAI. Like, I went in on them for 10 plus minutes. And not only have they been really goo... [10]
11. Prisma is removing Rust? - Theo - t3․gg: forgiving of types also they're not a typescript fan which I know makes it hard to trust them it's easy to pick up and supported by browsers is a huge pool of people who are conversent with it for years we've had both Library authors and consumers in the JS ecosystem largely using JS I think we take for granted what this enables Matteo from the node teams quoted saying that most devs ignore the fact they have the ski... [11]
12. OpenAI is TERRIFIED (this is absurd) - Theo - t3․gg: t criticizes OpenAI's claim that DeepSeek models compromise user privacy and security, arguing that running models locally on personal infrastructure avoids these risks. DeepSeek is praised as potentially "the most open AI company ever," having published 12 papers and open-sourced significant innovations in training efficiency. OpenAI's proposal focuses on banning the models themselves, not just the API, which the ho... [12]

### q062 PASS

- Prompt: What highlights have I saved from this video?
- Class: `highlight_lookup`
- Status: `Completed`
- Score: `3`
- Sources: `0`
- Failure: `-`
- Tools: Saved highlights lookup (highlight_lookup), Saved highlights lookup (highlight_lookup)

#### Answer

Retrieved tool evidence for: +{Open source is dead now?} What highlights have I saved from this video?

The answer model is unavailable, so this fallback returns the grounded tool results directly.

1. Look up saved highlights for videos matching "Open source is dead now?":
No saved highlights matched videos matching "Open source is dead now?".

### q063 PASS

- Prompt: Show me all highlights related to search.
- Class: `highlight_lookup`
- Status: `Completed`
- Score: `3`
- Sources: `0`
- Failure: `-`
- Tools: Saved highlights lookup (highlight_lookup), Saved highlights lookup (highlight_lookup)

#### Answer

Retrieved tool evidence for: Show me all highlights related to search.

The answer model is unavailable, so this fallback returns the grounded tool results directly.

1. Look up saved highlights for query "search.":
No saved highlights matched query "search.".

### q064 PASS

- Prompt: Show me all highlights related to summaries.
- Class: `highlight_lookup`
- Status: `Completed`
- Score: `3`
- Sources: `0`
- Failure: `-`
- Tools: Saved highlights lookup (highlight_lookup), Saved highlights lookup (highlight_lookup)

#### Answer

Retrieved tool evidence for: Show me all highlights related to summaries.

The answer model is unavailable, so this fallback returns the grounded tool results directly.

1. Look up saved highlights for query "summaries.":
No saved highlights matched query "summaries.".

### q065 PASS

- Prompt: Show me all highlights related to evaluation.
- Class: `highlight_lookup`
- Status: `Completed`
- Score: `3`
- Sources: `0`
- Failure: `-`
- Tools: Saved highlights lookup (highlight_lookup), Saved highlights lookup (highlight_lookup)

#### Answer

Retrieved tool evidence for: Show me all highlights related to evaluation.

The answer model is unavailable, so this fallback returns the grounded tool results directly.

1. Look up saved highlights for query "evaluation.":
No saved highlights matched query "evaluation.".

### q066 PASS

- Prompt: Which highlight best captures the video's main point?
- Class: `highlight_clustering`
- Status: `Completed`
- Score: `3`
- Sources: `0`
- Failure: `-`
- Tools: Saved highlights lookup (highlight_lookup), Saved highlights lookup (highlight_lookup)

#### Answer

Retrieved tool evidence for: +{Open source is dead now?} Which highlight best captures the video's main point?

The answer model is unavailable, so this fallback returns the grounded tool results directly.

1. Look up saved highlights for query "Which  best captures the video's main point?":
No saved highlights matched query "Which  best captures the video's main point?".

### q067 PASS

- Prompt: Which highlights are most useful as a quick reference?
- Class: `highlight_clustering`
- Status: `Completed`
- Score: `3`
- Sources: `0`
- Failure: `-`
- Tools: Saved highlights lookup (highlight_lookup), Saved highlights lookup (highlight_lookup)

#### Answer

Retrieved tool evidence for: Which highlights are most useful as a quick reference?

The answer model is unavailable, so this fallback returns the grounded tool results directly.

1. Look up saved highlights for query "Which  are most useful as a quick reference?":
No saved highlights matched query "Which  are most useful as a quick reference?".

### q068 PASS

- Prompt: What are the most interesting snippets I've highlighted across the library?
- Class: `highlight_clustering`
- Status: `Completed`
- Score: `3`
- Sources: `0`
- Failure: `-`
- Tools: Saved highlights lookup (highlight_lookup), Saved highlights lookup (highlight_lookup)

#### Answer

Retrieved tool evidence for: What are the most interesting snippets I've highlighted across the library?

The answer model is unavailable, so this fallback returns the grounded tool results directly.

1. Look up saved highlights for query "What are the most interesting snippets I've ed across the library?":
No saved highlights matched query "What are the most interesting snippets I've ed across the library?".

### q069 PASS

- Prompt: Group my highlights by theme.
- Class: `highlight_clustering`
- Status: `Completed`
- Score: `3`
- Sources: `0`
- Failure: `-`
- Tools: Saved highlights lookup (highlight_lookup), Saved highlights lookup (highlight_lookup)

#### Answer

Retrieved tool evidence for: Group my highlights by theme.

The answer model is unavailable, so this fallback returns the grounded tool results directly.

1. Look up saved highlights for query "Group my  by theme.":
No saved highlights matched query "Group my  by theme.".

### q070 PASS

- Prompt: Find highlights that support a specific claim.
- Class: `highlight_lookup`
- Status: `Completed`
- Score: `3`
- Sources: `0`
- Failure: `-`
- Tools: Saved highlights lookup (highlight_lookup), Saved highlights lookup (highlight_lookup)

#### Answer

Retrieved tool evidence for: Find highlights that support a specific claim.

The answer model is unavailable, so this fallback returns the grounded tool results directly.

1. Look up saved highlights for query "support":
No saved highlights matched query "support".

### q071 PASS

- Prompt: Find highlights that contradict a specific claim.
- Class: `highlight_lookup`
- Status: `Completed`
- Score: `3`
- Sources: `0`
- Failure: `-`
- Tools: Saved highlights lookup (highlight_lookup), Saved highlights lookup (highlight_lookup)

#### Answer

Retrieved tool evidence for: Find highlights that contradict a specific claim.

The answer model is unavailable, so this fallback returns the grounded tool results directly.

1. Look up saved highlights for query "contradict":
No saved highlights matched query "contradict".

### q072 PASS

- Prompt: What parts of this video seem most worth revisiting?
- Class: `timestamp_navigation`
- Status: `Completed`
- Score: `3`
- Sources: `12`
- Failure: `-`
- Source videos: A breakdown of style solutions for 2025 | I hate that this is still happening | I'm so f***ing tired of Obsidian. | It's not just you (Claude did get dumber) | It’s time to embrace the AI | JavaScript Frameworks in 2025 | Open source is dying | Vibe coding is already dead

#### Answer

Retrieved evidence for: +{Open source is dead now?} What parts of this video seem most worth revisiting?

The answer model is unavailable, so this fallback lists the highest-ranked saved excerpts instead of synthesizing beyond them.

Timed captions may be unavailable, so these section candidates are the closest grounded matches. Use the linked timestamps when present, and otherwise treat the cited sections below as the best revisit points.

1. Open source is dying - Theo - t3․gg: escalate because you almost certainly have to use AI to actually scan these PRs. It has a lot of config, enough config that I'm almost certain that this project was vibe coded, but yeah, you get the idea. There are a lot of solutions being made to try and fix these problems. Some of them are going to make it harder for new maintainers to break out, which sucks because we might just have our current maintainers until ... [1]
2. Open source is dying - Theo - t3․gg / Overview: The video presents a comprehensive examination of how AI is negatively impacting the open source ecosystem. The speaker, an experienced open source maintainer and creator of T3 Code, shares firsthand experiences managing a new project that received 150 PRs in just 5 days. The discussion covers four main problem areas: PR spam and quality degradation, increasingly hostile and confused users, GitHub's inadequate platfo... [2]
3. JavaScript Frameworks in 2025 - Theo - t3․gg: do to make the run and better what I saw was all of this complexity in chaos being reduced to a much simpler model where is it down here the isomorphic spa the complexity here has been reduced a ton at the cost of having to understand this part the reality is that I already understood this part because I'm a backend Dev I know how these parts work but if you are a web dev that has mostly ignored the relationship betw... [3]
4. It’s time to embrace the AI - Theo - t3․gg: editor. If he could automate more of that work and his editing tools made those parts easier, he could spend more time on the fun things like the fancy start of the videos. He could work with more people and get more done. Automating the frustrating parts of his job might mean he can take someone else's job, but more importantly, it means he can spend time on the parts he finds fun that also help make my videos bette... [4]
5. Open source is dying - Theo - t3․gg: put so much effort into killing Hacktoberfest. I think this video of mine, don't contribute to open source, is one of the best videos I ever filmed. Not sure who that blonde guy with the mustache is though. Seriously though, like that video, I have been told by so many maintainers how thankful they are for this video more than almost anything I've done as a developer and journalist, YouTuber, whatever you want to cal... [5]
6. Vibe coding is already dead - Theo - t3․gg: get this for a long ass time, but this has become a huge part of my life as a content creator. People seem to think that if you have a video that performs surprisingly well as a YouTuber, Instagram, whatever you're on, that what you should do next is the same topic again. It makes a lot of sense. If I mostly talk about I don't know React and I do one video about spelt instead and that spelt video does really well. Ob... [6]
7. I'm so f***ing tired of Obsidian. - Theo - t3․gg: Transcript: This video is going to be a little bit different. If you didn't already know this, I run most of my channel through Notion. Everything from our content calendar and when videos come out to my list of topics that I intend to cover to our research to our assignments to our brands to the sponsors, like everything about what makes a specific video a specific video is managed through Notion. Normally, this isn... [7]
8. I hate that this is still happening - Theo - t3․gg: use to make them is very different from the tech I started with. The best thing to make your first video with is the things you already have. You shouldn't buy a bunch of new stuff to inspire you to make the first video. You should do it despite not having the right equipment. And once you get good at it, you'll figure out what your equipment can and can't do and make changes based on what you know. And this is the r... [8]
9. I hate that this is still happening - Theo - t3․gg: up here. Lionus doesn't like open source because he's a god dev. He likes open source because he had to go through this whole process himself as a bad dev becoming a good dev over time. And during his process getting there, he grew a fondness to open source because the closed source systems he was using caused him a lot of problems. He also earned his right to be an [ __ ] because of all of the problems he encountere... [9]
10. It's not just you (Claude did get dumber) - Theo - t3․gg: small percentage to some. They can be sued if they lie. So, they aren't lying here. Again, they use whatever language makes it sound as not bad as possible because Anthropic is not interested in transparency. This first issue since it was a small percentage they said that and then this issue as well as the opus issue that we discussed earlier were not small issues as such they were not called that and also here with.... [10]
11. Open source is dying - Theo - t3․gg: to hire because this just makes my life easier. If you see an issue that's really stale that has already been fixed, comment saying, "Hey, are you sure you're on the latest version? I think this PR fixed it. It doesn't happen for me anymore on the latest." These types of things are so goddamn helpful. And once you've done that a bit on the issue side, you can start doing the same on the PR side. And here, Ben Bandit ... [11]
12. A breakdown of style solutions for 2025 - Theo - t3․gg: And I don't fathom how anyone can see it differently. It's just hard for me to comprehend. So, it's happened. Chad Cienne is the perfect thing in the middle here. It is the thing I wanted when I filmed my last video. Most of that video was me just complaining that the solutions in the middle were bad because they didn't take advantage of all the awesome technologies in the other circles. Now we have something in the.... [12]

### q073 PASS

- Prompt: What is the most memorable line in this transcript?
- Class: `tone_or_style_inference`
- Status: `Completed`
- Score: `3`
- Sources: `12`
- Failure: `-`
- Source videos: GPT-5.2 is dumb (I’m tired of benchmarks) | It's time to fix open source | Open source is dying | React Native is kind of broken (they NEED to fix this) | This magic hack makes Next.js possible | Why Github Actually Won | Why Tech Companies Are Moving Off React | Why is everyone open sourcing their startups?

#### Answer

Retrieved evidence for: +{Open source is dead now?} What is the most memorable line in this transcript?

The answer model is unavailable, so this fallback lists the highest-ranked saved excerpts instead of synthesizing beyond them.

From the available evidence, these excerpts support only a tentative reading rather than a definitive judgment.

1. React Native is kind of broken (they NEED to fix this) - Theo - t3․gg: to adopt it should be aware of this issue we previously reported it back in April it's not a fun thing to discover when you already have a lot of existing code it's much better to know early and this is far from the only bug related to controlled inputs here's another one I just ran into today this is actually what pushed me over the line to file this issue the typical workaround is to use default value and avoid con... [1]
2. GPT-5.2 is dumb (I’m tired of benchmarks) - Theo - t3․gg: pture real-world usability. Simple Bench (by AI Explained) shows GPT-5.2 scoring below Claude 4 Opus, Claude 4.1 Opus, Grok 4, and even Gemini 2.5 Pro—a concerning result for a flagship model. GPT-5.2 Pro (the most expensive tier) ranked 8th on Simple Bench; GPT-5.2 High ranked below Claude 3.7 Sonnet. Skate Bench: Spatial Reasoning Regression Test The custom benchmark tests models on naming skateboard tricks, which.... [2]
3. Open source is dying - Theo - t3․gg: feel awesome. Those messages make my goddamn day. Seeing somebody hit me up about how they were a line cook for a decade, learning code on the side, didn't feel like they could really do it, but watching my videos made them feel more like this crazy tech world we were in was a place they could fit, and now they have awesome tech jobs. My video isn't what did it. My channel isn't what did it. They did it. But that mes... [3]
4. Why Tech Companies Are Moving Off React - Theo - t3․gg: in context, engineers had combed through every single bit of these apps to add thousands of memorization calls. And of course, this work was done against metrics which rewarded and the scale of every single optimization. On such an app, improving just a single thing, something like the time to first paint of a specific page by just one or two% would already be a huge deal. But React compiler had significantly improve... [4]
5. Open source is dying - Theo - t3․gg: reason people maintain open- source software is because they care so much that you could argue it's too much. And when their job gets harder and harder because of all of this AI [ __ ] it gets more likely they give up. The reason they're here is that excitement. And if you can remind them of that, if you can be the excited thing that made them do this in the first place, you can make it feel so much more worth it. Yo... [5]
6. Open source is dying - Theo - t3․gg: Twitter DMs with the update with the encrypted stuff. But before then, the rate stayed nearly flat as I continued to get more relevant in the space. A simple two sentence, "Hey, I really appreciated this PR you shipped. I've been a fan of what you've been building for years. This library makes my life much better. Thank you." Those messages might seem small, but they can actually change your life. And I would not be ... [6]
7. This magic hack makes Next.js possible - Theo - t3․gg: c under the new model. `use cache` allows dynamic components to still benefit from static caching by checking the cache synchronously before hitting any async boundaries, effectively telling Next.js "trust me, this can be green." This approach removes the need for magic helper functions and arbitrary rules, making Next.js more "JavaScript-native" by using the language itself to determine caching behavior. Overview Th... [7]
8. Open source is dying - Theo - t3․gg: escalate because you almost certainly have to use AI to actually scan these PRs. It has a lot of config, enough config that I'm almost certain that this project was vibe coded, but yeah, you get the idea. There are a lot of solutions being made to try and fix these problems. Some of them are going to make it harder for new maintainers to break out, which sucks because we might just have our current maintainers until ... [8]
9. It's time to fix open source - Theo - t3․gg: you've seen from us publicly running it for three consecutive years each year we increase the funding amount based on Cent's own Financial growth it became such a no-brainer within Cent's leadership that we've aggressively increased the funding every year even beyond our original targets with the success of that we set off to take this program codify it and bring it to other companies to see if we could turn this int... [9]
10. Why is everyone open sourcing their startups? - Theo - t3․gg: and say hey any chance you've worked on this yet that all goes away here where instead it's hey here's the issue on your GitHub that we care about. They say oh we plan on doing this at this point in time and our response is oh we already have it done here's a PR. that iteration loop ends up being much more effective both for them as a business because it helps them understand what we need better and for us as consume... [10]
11. Why Github Actually Won - Theo - t3․gg: think started coming to a head around this time was not in the world of professional development within closed and trusted teams. The big problem was within the growing world of open source also very real. An important detail with git itself is that git was spiked driven development. Lionus was mad that he could no longer use the solutions that they were using. I believe it was it wasn't perforce. What were they usin... [11]
12. Open source is dying - Theo - t3․gg: put so much effort into killing Hacktoberfest. I think this video of mine, don't contribute to open source, is one of the best videos I ever filmed. Not sure who that blonde guy with the mustache is though. Seriously though, like that video, I have been told by so many maintainers how thankful they are for this video more than almost anything I've done as a developer and journalist, YouTuber, whatever you want to cal... [12]

### q074 PASS

- Prompt: Which timestamps matter most in this transcript?
- Class: `timestamp_navigation`
- Status: `Completed`
- Score: `3`
- Sources: `12`
- Failure: `-`
- Source videos: It's time to fix open source | Open source is dying

#### Answer

Retrieved evidence for: +{Open source is dead now?} Which timestamps matter most in this transcript?

The answer model is unavailable, so this fallback lists the highest-ranked saved excerpts instead of synthesizing beyond them.

Timed captions may be unavailable, so these section candidates are the closest grounded matches. Use the linked timestamps when present, and otherwise treat the cited sections below as the best revisit points.

1. Open source is dying - Theo - t3․gg: been very helpful for this. Huge shout out to Maria and Bin Bandit. They've been essential as this repo gets more and more popular. So, here's a bug somebody reported where they can't close the diff side panel. Someone asked, "You got the latest version?" Oh, I see this bug. We'll push a PR soon. CC Julius so that he knows that he's working on this so they don't end up clobbering each other working on the same thing ... [1]
2. Open source is dying - Theo - t3․gg: which is insane, especially considering how useful and used this project is. So, what is Vouch? Community trust management system. People must be vouched for before interacting with certain parts of a project. people can explicitly be denounced to block them from interacting with the project going forward. You set up vouch as a workflow. It will automatically run in this case whenever PRs are opened, reopened, synced... [2]
3. Open source is dying - Theo - t3․gg: Twitter DMs with the update with the encrypted stuff. But before then, the rate stayed nearly flat as I continued to get more relevant in the space. A simple two sentence, "Hey, I really appreciated this PR you shipped. I've been a fan of what you've been building for years. This library makes my life much better. Thank you." Those messages might seem small, but they can actually change your life. And I would not be ... [3]
4. Open source is dying - Theo - t3․gg: feel awesome. Those messages make my goddamn day. Seeing somebody hit me up about how they were a line cook for a decade, learning code on the side, didn't feel like they could really do it, but watching my videos made them feel more like this crazy tech world we were in was a place they could fit, and now they have awesome tech jobs. My video isn't what did it. My channel isn't what did it. They did it. But that mes... [4]
5. It's time to fix open source - Theo - t3․gg: you've seen from us publicly running it for three consecutive years each year we increase the funding amount based on Cent's own Financial growth it became such a no-brainer within Cent's leadership that we've aggressively increased the funding every year even beyond our original targets with the success of that we set off to take this program codify it and bring it to other companies to see if we could turn this int... [5]
6. Open source is dying - Theo - t3․gg: want to highlight one particular PR that annoyed me. We had a stailed to-do MD file in the repo that had random things I was working on at some point in it. And someone filed a nonsense PR that tried to fix all of those things and ended up just breaking other things. They didn't get any response from us cuz we were being flooded with PRs. So he randomly tags me and two other people whose PRs merged recently. I was so... [6]
7. Open source is dying - Theo - t3․gg: They're already on the line of giving up. They suddenly have more reason to give up. Significantly more reason. That sucks. That has the potential to cause real long-term damage in this industry. This is how things like the XZ back door happen. If you're not familiar with the story, I'll TLDDR quick. XC is a really important compression library used by a ton of open source software, especially in the Linux ecosystem.... [7]
8. Open source is dying - Theo - t3․gg: we all are nerdy about and care about. I bring this up because there's a couple things that we just experience in life differently because of that. The one I'm imagining right now, and I'm sure a lot of y'all are this one's in chat if you can relate. I used to get a lot of texts from family members, random friends in high school and just people in my life asking random [ __ ] about computers. Anything from, "Can you ... [8]
9. Open source is dying - Theo - t3․gg: the old data and move it over so we can handle that. Just good PR, reasonable changes that were actually needed. Since it was super simple, linked to the issue, described what was going on. This is a very easy PR to click merge on. Another great one from Maria. Aligning package versions before building artifacts. This makes sure that all of the package versions for everything are the same before we go and build the a... [9]
10. Open source is dying - Theo - t3․gg: poorest. Really crippled the image I had in relation to your channel and content. I have been contributing to projects since before you were born. such an attitude, including some YouTube codes of yours that I've never received before. Feel free to block me. Mature. This is somebody who never wrote code before AI. Straight up. And I promise you, you were not contributing to code years ago, [ __ ] And this is the hot ... [10]
11. Open source is dying - Theo - t3․gg: reason people maintain open- source software is because they care so much that you could argue it's too much. And when their job gets harder and harder because of all of this AI [ __ ] it gets more likely they give up. The reason they're here is that excitement. And if you can remind them of that, if you can be the excited thing that made them do this in the first place, you can make it feel so much more worth it. Yo... [11]
12. Open source is dying - Theo - t3․gg: escalate because you almost certainly have to use AI to actually scan these PRs. It has a lot of config, enough config that I'm almost certain that this project was vibe coded, but yeah, you get the idea. There are a lot of solutions being made to try and fix these problems. Some of them are going to make it harder for new maintainers to break out, which sucks because we might just have our current maintainers until ... [12]

### q075 PASS

- Prompt: Find the section where the speaker explains the core idea.
- Class: `timestamp_navigation`
- Status: `Completed`
- Score: `3`
- Sources: `8`
- Failure: `-`
- Source videos: Are juniors screwed? (Getting a job in a post-AI world) | ChatGPT Atlas Drove Me Insane (it's not just Chrome) | Cursor, Claude Code and Codex all have a BIG problem | Delete your CLAUDE.md (and your AGENT.md too) | The Truth About React Native | The actual reason you can't get a job | gpt-5.4 is really, really good

#### Answer

Retrieved evidence for: Find the section where the speaker explains the core idea.

The answer model is unavailable, so this fallback lists the highest-ranked saved excerpts instead of synthesizing beyond them.

Timed captions may be unavailable, so these section candidates are the closest grounded matches. Use the linked timestamps when present, and otherwise treat the cited sections below as the best revisit points.

1. The Truth About React Native - Theo - t3․gg / Key Points: ntire applications. The project started as a way for teams to embed UI components into existing apps without requiring dedicated mobile engineers for every feature. At Facebook, this enables "vertical slicing" where product teams (like ads, feed, messages) own their entire stack across platforms, rather than having separate frontend and backend teams. This architectural approach means that finding native code in an a... [1]
2. The actual reason you can't get a job - Theo - t3․gg: just by being there, being accessible, being real, and talking about the things you actually give a [ __ ] about, it's unbelievable. There is no growth hack that is more powerful than talking about the [ __ ] you care about with others who also care about the thing. That will always lead you to success faster than anything else. And anyone trying to say otherwise is trying to sell you some [ __ ] And honestly, this a... [2]
3. Cursor, Claude Code and Codex all have a BIG problem - Theo - t3․gg: time. Codebase quality peaks at approximately 6 months; after that, bad patterns spread exponentially while good patterns spread linearly, making early code quality critical for long-term maintainability. The speaker advocates for "sledgehammer development" - aggressively deleting and rewriting problematic code sections rather than trying to fix them incrementally, which is now economically viable with modern AI too.... [3]
4. Delete your CLAUDE.md (and your AGENT.md too) - Theo - t3․gg: ease in task success and only marginally improving performance (4% average) when written by developers. Context files increase agent exploration, testing, and reasoning, resulting in over 20% higher costs; the speaker's own test showed a 25% time penalty (1m11s vs 1m29s) when using a claude.md file. Most information developers put in these files (architecture overviews, command lists, dependencies) is already discove... [4]
5. Are juniors screwed? (Getting a job in a post-AI world) - Theo - t3․gg: s, contributing to issues, and helping others in Discords/GitHub is a powerful way to stand out and get hired Overview This video breaks down the current state of the software engineering job market, which the speaker describes as "weird" with conflicting signals about unemployment and hiring difficulties. The content is structured into three main sections: companies' failures in hiring processes, experienced develop... [5]
6. ChatGPT Atlas Drove Me Insane (it's not just Chrome) - Theo - t3․gg: cross-platform support (especially Windows) extremely difficult. Input events are translated through a complex multi-stage pipeline (NS Event → Web Input Event → potentially re-synthesized NS Event), which the speaker finds horrifying to maintain. Atlas uses Chromium's Mojo IPC system with custom Swift and TypeScript bindings to communicate between the separate processes. The browser handles agent mode by compositing... [6]
7. gpt-5.4 is really, really good - Theo - t3․gg: weakness compared to competitors like Opus and Gemini, requiring extensive prompt engineering to achieve acceptable results. Benchmark performance is strong on SWE-Bench Pro (57.7%) and other tests, though the speaker's private SkateBench V2 shows Gemini 3.1 Pro Preview leading at 97% vs GPT 5.4 High at 82%. 5.4 Pro and X-High variants often underperform compared to standard 5.4 High in practical use, despite higher.... [7]
8. The Truth About React Native - Theo - t3․gg: gned to integrate into existing native apps, not necessarily replace entire apps; companies like Facebook, Microsoft, Amazon, and Sony use it for specific features or products while maintaining native code elsewhere. Approximately 25% of top 100 apps across major App Store categories use React Native, and there are roughly 10x more React Native job listings than Swift UI or Jetpack Compose positions. Meta Quest's sys... [8]

### q076 PASS

- Prompt: Find the section where the speaker gives an example.
- Class: `timestamp_navigation`
- Status: `Completed`
- Score: `3`
- Sources: `7`
- Failure: `-`
- Source videos: Are juniors screwed? (Getting a job in a post-AI world) | ChatGPT Atlas Drove Me Insane (it's not just Chrome) | Cursor, Claude Code and Codex all have a BIG problem | Delete your CLAUDE.md (and your AGENT.md too) | The Truth About React Native | gpt-5.4 is really, really good

#### Answer

Retrieved evidence for: Find the section where the speaker gives an example.

The answer model is unavailable, so this fallback lists the highest-ranked saved excerpts instead of synthesizing beyond them.

Timed captions may be unavailable, so these section candidates are the closest grounded matches. Use the linked timestamps when present, and otherwise treat the cited sections below as the best revisit points.

1. The Truth About React Native - Theo - t3․gg / Key Points: ntire applications. The project started as a way for teams to embed UI components into existing apps without requiring dedicated mobile engineers for every feature. At Facebook, this enables "vertical slicing" where product teams (like ads, feed, messages) own their entire stack across platforms, rather than having separate frontend and backend teams. This architectural approach means that finding native code in an a... [1]
2. Cursor, Claude Code and Codex all have a BIG problem - Theo - t3․gg: time. Codebase quality peaks at approximately 6 months; after that, bad patterns spread exponentially while good patterns spread linearly, making early code quality critical for long-term maintainability. The speaker advocates for "sledgehammer development" - aggressively deleting and rewriting problematic code sections rather than trying to fix them incrementally, which is now economically viable with modern AI too.... [2]
3. Delete your CLAUDE.md (and your AGENT.md too) - Theo - t3․gg: ease in task success and only marginally improving performance (4% average) when written by developers. Context files increase agent exploration, testing, and reasoning, resulting in over 20% higher costs; the speaker's own test showed a 25% time penalty (1m11s vs 1m29s) when using a claude.md file. Most information developers put in these files (architecture overviews, command lists, dependencies) is already discove... [3]
4. Are juniors screwed? (Getting a job in a post-AI world) - Theo - t3․gg: s, contributing to issues, and helping others in Discords/GitHub is a powerful way to stand out and get hired Overview This video breaks down the current state of the software engineering job market, which the speaker describes as "weird" with conflicting signals about unemployment and hiring difficulties. The content is structured into three main sections: companies' failures in hiring processes, experienced develop... [4]
5. ChatGPT Atlas Drove Me Insane (it's not just Chrome) - Theo - t3․gg: cross-platform support (especially Windows) extremely difficult. Input events are translated through a complex multi-stage pipeline (NS Event → Web Input Event → potentially re-synthesized NS Event), which the speaker finds horrifying to maintain. Atlas uses Chromium's Mojo IPC system with custom Swift and TypeScript bindings to communicate between the separate processes. The browser handles agent mode by compositing... [5]
6. gpt-5.4 is really, really good - Theo - t3․gg: weakness compared to competitors like Opus and Gemini, requiring extensive prompt engineering to achieve acceptable results. Benchmark performance is strong on SWE-Bench Pro (57.7%) and other tests, though the speaker's private SkateBench V2 shows Gemini 3.1 Pro Preview leading at 97% vs GPT 5.4 High at 82%. 5.4 Pro and X-High variants often underperform compared to standard 5.4 High in practical use, despite higher.... [6]
7. The Truth About React Native - Theo - t3․gg: gned to integrate into existing native apps, not necessarily replace entire apps; companies like Facebook, Microsoft, Amazon, and Sony use it for specific features or products while maintaining native code elsewhere. Approximately 25% of top 100 apps across major App Store categories use React Native, and there are roughly 10x more React Native job listings than Swift UI or Jetpack Compose positions. Meta Quest's sys... [7]

### q077 PASS

- Prompt: Find the section where the speaker changes direction.
- Class: `timestamp_navigation`
- Status: `Completed`
- Score: `3`
- Sources: `10`
- Failure: `-`
- Source videos: Are juniors screwed? (Getting a job in a post-AI world) | ChatGPT Atlas Drove Me Insane (it's not just Chrome) | Cursor, Claude Code and Codex all have a BIG problem | Delete your CLAUDE.md (and your AGENT.md too) | How I cracked an impossible DEF CON challenge | The Truth About React Native | gpt-5.4 is really, really good

#### Answer

Retrieved evidence for: Find the section where the speaker changes direction.

The answer model is unavailable, so this fallback lists the highest-ranked saved excerpts instead of synthesizing beyond them.

Timed captions may be unavailable, so these section candidates are the closest grounded matches. Use the linked timestamps when present, and otherwise treat the cited sections below as the best revisit points.

1. The Truth About React Native - Theo - t3․gg / Key Points: ntire applications. The project started as a way for teams to embed UI components into existing apps without requiring dedicated mobile engineers for every feature. At Facebook, this enables "vertical slicing" where product teams (like ads, feed, messages) own their entire stack across platforms, rather than having separate frontend and backend teams. This architectural approach means that finding native code in an a... [1]
2. How I cracked an impossible DEF CON challenge - Theo - t3․gg: are a fellow music nerd you know that 44 means the beat is not everything here so in this first one the beat would be this note this a then we have a rest it's an eighth rest so that this would be offbeat and nothing then we have this F which would be on beat then a rest then we have this E flat then we have this a but this a wouldn't be on beat because it's 1 2 3 4 and there's a gap between each one when you're play... [2]
3. How I cracked an impossible DEF CON challenge - Theo - t3․gg / Key Points: a Note of where the beat lands" with capital N emphasizing musical notes. The speaker, being musically literate, understood that in 4/4 time, only notes on beats 1, 3, 5, and 7 (quarter note positions) matter when working with eighth notes. They created annotated PDFs marking which notes were on-beat versus off-beat. Text Block Investigation The text block above the score had unusual formatting—one line was significa... [3]
4. Cursor, Claude Code and Codex all have a BIG problem - Theo - t3․gg: time. Codebase quality peaks at approximately 6 months; after that, bad patterns spread exponentially while good patterns spread linearly, making early code quality critical for long-term maintainability. The speaker advocates for "sledgehammer development" - aggressively deleting and rewriting problematic code sections rather than trying to fix them incrementally, which is now economically viable with modern AI too.... [4]
5. Delete your CLAUDE.md (and your AGENT.md too) - Theo - t3․gg: ease in task success and only marginally improving performance (4% average) when written by developers. Context files increase agent exploration, testing, and reasoning, resulting in over 20% higher costs; the speaker's own test showed a 25% time penalty (1m11s vs 1m29s) when using a claude.md file. Most information developers put in these files (architecture overviews, command lists, dependencies) is already discove... [5]
6. How I cracked an impossible DEF CON challenge - Theo - t3․gg: which doesn't matter because all of these notes are too low except for this a so this a didn't fit horizontally and this G didn't fit vertically and both of those were driving me crazy but then I realized that the second measure G is this G this G should have been a g flat but this G's offbeat it shouldn't matter wait do the Beats even matter should I just be notating this in e instead of fourths so I redid the beat ... [6]
7. Are juniors screwed? (Getting a job in a post-AI world) - Theo - t3․gg: s, contributing to issues, and helping others in Discords/GitHub is a powerful way to stand out and get hired Overview This video breaks down the current state of the software engineering job market, which the speaker describes as "weird" with conflicting signals about unemployment and hiring difficulties. The content is structured into three main sections: companies' failures in hiring processes, experienced develop... [7]
8. ChatGPT Atlas Drove Me Insane (it's not just Chrome) - Theo - t3․gg: cross-platform support (especially Windows) extremely difficult. Input events are translated through a complex multi-stage pipeline (NS Event → Web Input Event → potentially re-synthesized NS Event), which the speaker finds horrifying to maintain. Atlas uses Chromium's Mojo IPC system with custom Swift and TypeScript bindings to communicate between the separate processes. The browser handles agent mode by compositing... [8]
9. gpt-5.4 is really, really good - Theo - t3․gg: weakness compared to competitors like Opus and Gemini, requiring extensive prompt engineering to achieve acceptable results. Benchmark performance is strong on SWE-Bench Pro (57.7%) and other tests, though the speaker's private SkateBench V2 shows Gemini 3.1 Pro Preview leading at 97% vs GPT 5.4 High at 82%. 5.4 Pro and X-High variants often underperform compared to standard 5.4 High in practical use, despite higher.... [9]
10. The Truth About React Native - Theo - t3․gg: gned to integrate into existing native apps, not necessarily replace entire apps; companies like Facebook, Microsoft, Amazon, and Sony use it for specific features or products while maintaining native code elsewhere. Approximately 25% of top 100 apps across major App Store categories use React Native, and there are roughly 10x more React Native job listings than Swift UI or Jetpack Compose positions. Meta Quest's sys... [10]

### q078 PASS

- Prompt: Find the section where the speaker lists tradeoffs.
- Class: `timestamp_navigation`
- Status: `Completed`
- Score: `3`
- Sources: `8`
- Failure: `-`
- Source videos: Are juniors screwed? (Getting a job in a post-AI world) | ChatGPT Atlas Drove Me Insane (it's not just Chrome) | Cursor, Claude Code and Codex all have a BIG problem | Delete your CLAUDE.md (and your AGENT.md too) | Hacking LightHouse Scores | The Truth About React Native | gpt-5.4 is really, really good

#### Answer

Retrieved evidence for: Find the section where the speaker lists tradeoffs.

The answer model is unavailable, so this fallback lists the highest-ranked saved excerpts instead of synthesizing beyond them.

Timed captions may be unavailable, so these section candidates are the closest grounded matches. Use the linked timestamps when present, and otherwise treat the cited sections below as the best revisit points.

1. The Truth About React Native - Theo - t3․gg / Key Points: ntire applications. The project started as a way for teams to embed UI components into existing apps without requiring dedicated mobile engineers for every feature. At Facebook, this enables "vertical slicing" where product teams (like ads, feed, messages) own their entire stack across platforms, rather than having separate frontend and backend teams. This architectural approach means that finding native code in an a... [1]
2. Hacking LightHouse Scores - Theo - t3․gg: TL;DR Lighthouse scores can be hacked to achieve perfect 100 scores through techniques like deferring content loading, delaying layout shifts, and manipulating LCP elements—often making sites objectively worse for users. Field data from real users (via tools like Sentry) matters far more than lab-based Lighthouse scores; a site with poor Lighthouse scores can still provide an excellent user experience. Lighthouse sco... [2]
3. Cursor, Claude Code and Codex all have a BIG problem - Theo - t3․gg: time. Codebase quality peaks at approximately 6 months; after that, bad patterns spread exponentially while good patterns spread linearly, making early code quality critical for long-term maintainability. The speaker advocates for "sledgehammer development" - aggressively deleting and rewriting problematic code sections rather than trying to fix them incrementally, which is now economically viable with modern AI too.... [3]
4. Delete your CLAUDE.md (and your AGENT.md too) - Theo - t3․gg: ease in task success and only marginally improving performance (4% average) when written by developers. Context files increase agent exploration, testing, and reasoning, resulting in over 20% higher costs; the speaker's own test showed a 25% time penalty (1m11s vs 1m29s) when using a claude.md file. Most information developers put in these files (architecture overviews, command lists, dependencies) is already discove... [4]
5. Are juniors screwed? (Getting a job in a post-AI world) - Theo - t3․gg: s, contributing to issues, and helping others in Discords/GitHub is a powerful way to stand out and get hired Overview This video breaks down the current state of the software engineering job market, which the speaker describes as "weird" with conflicting signals about unemployment and hiring difficulties. The content is structured into three main sections: companies' failures in hiring processes, experienced develop... [5]
6. ChatGPT Atlas Drove Me Insane (it's not just Chrome) - Theo - t3․gg: cross-platform support (especially Windows) extremely difficult. Input events are translated through a complex multi-stage pipeline (NS Event → Web Input Event → potentially re-synthesized NS Event), which the speaker finds horrifying to maintain. Atlas uses Chromium's Mojo IPC system with custom Swift and TypeScript bindings to communicate between the separate processes. The browser handles agent mode by compositing... [6]
7. gpt-5.4 is really, really good - Theo - t3․gg: weakness compared to competitors like Opus and Gemini, requiring extensive prompt engineering to achieve acceptable results. Benchmark performance is strong on SWE-Bench Pro (57.7%) and other tests, though the speaker's private SkateBench V2 shows Gemini 3.1 Pro Preview leading at 97% vs GPT 5.4 High at 82%. 5.4 Pro and X-High variants often underperform compared to standard 5.4 High in practical use, despite higher.... [7]
8. The Truth About React Native - Theo - t3․gg: gned to integrate into existing native apps, not necessarily replace entire apps; companies like Facebook, Microsoft, Amazon, and Sony use it for specific features or products while maintaining native code elsewhere. Approximately 25% of top 100 apps across major App Store categories use React Native, and there are roughly 10x more React Native job listings than Swift UI or Jetpack Compose positions. Meta Quest's sys... [8]

### q079 PASS

- Prompt: Find the section where the speaker talks about implementation details.
- Class: `timestamp_navigation`
- Status: `Completed`
- Score: `3`
- Sources: `7`
- Failure: `-`
- Source videos: Are juniors screwed? (Getting a job in a post-AI world) | ChatGPT Atlas Drove Me Insane (it's not just Chrome) | Cursor, Claude Code and Codex all have a BIG problem | Delete your CLAUDE.md (and your AGENT.md too) | The Truth About React Native | gpt-5.4 is really, really good

#### Answer

Retrieved evidence for: Find the section where the speaker talks about implementation details.

The answer model is unavailable, so this fallback lists the highest-ranked saved excerpts instead of synthesizing beyond them.

Timed captions may be unavailable, so these section candidates are the closest grounded matches. Use the linked timestamps when present, and otherwise treat the cited sections below as the best revisit points.

1. The Truth About React Native - Theo - t3․gg / Key Points: ntire applications. The project started as a way for teams to embed UI components into existing apps without requiring dedicated mobile engineers for every feature. At Facebook, this enables "vertical slicing" where product teams (like ads, feed, messages) own their entire stack across platforms, rather than having separate frontend and backend teams. This architectural approach means that finding native code in an a... [1]
2. Cursor, Claude Code and Codex all have a BIG problem - Theo - t3․gg: time. Codebase quality peaks at approximately 6 months; after that, bad patterns spread exponentially while good patterns spread linearly, making early code quality critical for long-term maintainability. The speaker advocates for "sledgehammer development" - aggressively deleting and rewriting problematic code sections rather than trying to fix them incrementally, which is now economically viable with modern AI too.... [2]
3. Delete your CLAUDE.md (and your AGENT.md too) - Theo - t3․gg: ease in task success and only marginally improving performance (4% average) when written by developers. Context files increase agent exploration, testing, and reasoning, resulting in over 20% higher costs; the speaker's own test showed a 25% time penalty (1m11s vs 1m29s) when using a claude.md file. Most information developers put in these files (architecture overviews, command lists, dependencies) is already discove... [3]
4. Are juniors screwed? (Getting a job in a post-AI world) - Theo - t3․gg: s, contributing to issues, and helping others in Discords/GitHub is a powerful way to stand out and get hired Overview This video breaks down the current state of the software engineering job market, which the speaker describes as "weird" with conflicting signals about unemployment and hiring difficulties. The content is structured into three main sections: companies' failures in hiring processes, experienced develop... [4]
5. ChatGPT Atlas Drove Me Insane (it's not just Chrome) - Theo - t3․gg: cross-platform support (especially Windows) extremely difficult. Input events are translated through a complex multi-stage pipeline (NS Event → Web Input Event → potentially re-synthesized NS Event), which the speaker finds horrifying to maintain. Atlas uses Chromium's Mojo IPC system with custom Swift and TypeScript bindings to communicate between the separate processes. The browser handles agent mode by compositing... [5]
6. gpt-5.4 is really, really good - Theo - t3․gg: weakness compared to competitors like Opus and Gemini, requiring extensive prompt engineering to achieve acceptable results. Benchmark performance is strong on SWE-Bench Pro (57.7%) and other tests, though the speaker's private SkateBench V2 shows Gemini 3.1 Pro Preview leading at 97% vs GPT 5.4 High at 82%. 5.4 Pro and X-High variants often underperform compared to standard 5.4 High in practical use, despite higher.... [6]
7. The Truth About React Native - Theo - t3․gg: gned to integrate into existing native apps, not necessarily replace entire apps; companies like Facebook, Microsoft, Amazon, and Sony use it for specific features or products while maintaining native code elsewhere. Approximately 25% of top 100 apps across major App Store categories use React Native, and there are roughly 10x more React Native job listings than Swift UI or Jetpack Compose positions. Meta Quest's sys... [7]

### q080 PASS

- Prompt: Find the section where the speaker talks about results or outcomes.
- Class: `timestamp_navigation`
- Status: `Completed`
- Score: `3`
- Sources: `8`
- Failure: `-`
- Source videos: Are juniors screwed? (Getting a job in a post-AI world) | ChatGPT Atlas Drove Me Insane (it's not just Chrome) | Cursor, Claude Code and Codex all have a BIG problem | Delete your CLAUDE.md (and your AGENT.md too) | I can't believe he was right. | The Truth About React Native | gpt-5.4 is really, really good

#### Answer

Retrieved evidence for: Find the section where the speaker talks about results or outcomes.

The answer model is unavailable, so this fallback lists the highest-ranked saved excerpts instead of synthesizing beyond them.

Timed captions may be unavailable, so these section candidates are the closest grounded matches. Use the linked timestamps when present, and otherwise treat the cited sections below as the best revisit points.

1. I can't believe he was right. - Theo - t3․gg: as I do today, even if my relationship with it is very different than it was a year ago. And I recommend that you reflect yourself and give these things a try. Let me know what y'all think and how you're using these tools today. [1]
2. The Truth About React Native - Theo - t3․gg / Key Points: ntire applications. The project started as a way for teams to embed UI components into existing apps without requiring dedicated mobile engineers for every feature. At Facebook, this enables "vertical slicing" where product teams (like ads, feed, messages) own their entire stack across platforms, rather than having separate frontend and backend teams. This architectural approach means that finding native code in an a... [2]
3. Cursor, Claude Code and Codex all have a BIG problem - Theo - t3․gg: time. Codebase quality peaks at approximately 6 months; after that, bad patterns spread exponentially while good patterns spread linearly, making early code quality critical for long-term maintainability. The speaker advocates for "sledgehammer development" - aggressively deleting and rewriting problematic code sections rather than trying to fix them incrementally, which is now economically viable with modern AI too.... [3]
4. Delete your CLAUDE.md (and your AGENT.md too) - Theo - t3․gg: ease in task success and only marginally improving performance (4% average) when written by developers. Context files increase agent exploration, testing, and reasoning, resulting in over 20% higher costs; the speaker's own test showed a 25% time penalty (1m11s vs 1m29s) when using a claude.md file. Most information developers put in these files (architecture overviews, command lists, dependencies) is already discove... [4]
5. Are juniors screwed? (Getting a job in a post-AI world) - Theo - t3․gg: s, contributing to issues, and helping others in Discords/GitHub is a powerful way to stand out and get hired Overview This video breaks down the current state of the software engineering job market, which the speaker describes as "weird" with conflicting signals about unemployment and hiring difficulties. The content is structured into three main sections: companies' failures in hiring processes, experienced develop... [5]
6. ChatGPT Atlas Drove Me Insane (it's not just Chrome) - Theo - t3․gg: cross-platform support (especially Windows) extremely difficult. Input events are translated through a complex multi-stage pipeline (NS Event → Web Input Event → potentially re-synthesized NS Event), which the speaker finds horrifying to maintain. Atlas uses Chromium's Mojo IPC system with custom Swift and TypeScript bindings to communicate between the separate processes. The browser handles agent mode by compositing... [6]
7. gpt-5.4 is really, really good - Theo - t3․gg: weakness compared to competitors like Opus and Gemini, requiring extensive prompt engineering to achieve acceptable results. Benchmark performance is strong on SWE-Bench Pro (57.7%) and other tests, though the speaker's private SkateBench V2 shows Gemini 3.1 Pro Preview leading at 97% vs GPT 5.4 High at 82%. 5.4 Pro and X-High variants often underperform compared to standard 5.4 High in practical use, despite higher.... [7]
8. The Truth About React Native - Theo - t3․gg: gned to integrate into existing native apps, not necessarily replace entire apps; companies like Facebook, Microsoft, Amazon, and Sony use it for specific features or products while maintaining native code elsewhere. Approximately 25% of top 100 apps across major App Store categories use React Native, and there are roughly 10x more React Native job listings than Swift UI or Jetpack Compose positions. Meta Quest's sys... [8]

### q081 PASS

- Prompt: What questions does this video answer well?
- Class: `meta_learning_or_next_step`
- Status: `Completed`
- Score: `3`
- Sources: `12`
- Failure: `-`
- Source videos: Amazon bets big on React Native | Bun got bought by Anthropic (yes really) | Delete your CLAUDE.md (and your AGENT.md too) | JavaScript Frameworks in 2025 | My hot take on image formats | Open source is dying | What happened to me? | Why Github Actually Won

#### Answer

Retrieved evidence for: +{Open source is dead now?} What questions does this video answer well?

The answer model is unavailable, so this fallback lists the highest-ranked saved excerpts instead of synthesizing beyond them.

1. Open source is dying - Theo - t3․gg: escalate because you almost certainly have to use AI to actually scan these PRs. It has a lot of config, enough config that I'm almost certain that this project was vibe coded, but yeah, you get the idea. There are a lot of solutions being made to try and fix these problems. Some of them are going to make it harder for new maintainers to break out, which sucks because we might just have our current maintainers until ... [1]
2. Open source is dying - Theo - t3․gg / Overview: The video presents a comprehensive examination of how AI is negatively impacting the open source ecosystem. The speaker, an experienced open source maintainer and creator of T3 Code, shares firsthand experiences managing a new project that received 150 PRs in just 5 days. The discussion covers four main problem areas: PR spam and quality degradation, increasingly hostile and confused users, GitHub's inadequate platfo... [2]
3. What happened to me? - Theo - t3․gg: of my community, the people hanging out in Twitch chat right now who have been there since day one, watched this whole thing happen, many of which joined me on the same journey. So, knowing all of this, I want to talk a bit about how I choose a topic for a video because another one of the questions I get all of the time is, "What is your dream video that you would do if the algorithm wouldn't kill it?" A question I g... [3]
4. What happened to me? - Theo - t3․gg: plays, I don't film the video. If I don't care, I don't care. You cannot get me to do a video I don't feel like doing. You cannot pay me any amount of money to talk about something I don't want to talk about. Want to know a really funny thing? Probably shouldn't be sharing. In my onboarding email that I send to brands when I'm starting to work with them as sponsors, I have an FAQ section. And one of the questions in.... [4]
5. My hot take on image formats - Theo - t3․gg: and drop it into word it doesn't show up properly if they try to send it as an email attachment it is a file instead of an image these types of things are real complaints and they make a lot of sense but the Alternatives I've seen people propose make none another way to look at this video isn't so much a rant about why I love webp rather this is a rant but why I hate av1 yes I'm going to come out and say it the avif.... [5]
6. Open source is dying - Theo - t3․gg: put so much effort into killing Hacktoberfest. I think this video of mine, don't contribute to open source, is one of the best videos I ever filmed. Not sure who that blonde guy with the mustache is though. Seriously though, like that video, I have been told by so many maintainers how thankful they are for this video more than almost anything I've done as a developer and journalist, YouTuber, whatever you want to cal... [6]
7. JavaScript Frameworks in 2025 - Theo - t3․gg: is absolutely own language it has been for a bit but this is the like tripling down on it I'm kind of disappointed that to my memorization questions on interviews everyone can now just answer with just use the compiler man very fair point you know times our are stupidly so this is why you guys got to watch my react compiler content I go so deep on these things and no one cares it's yeah the I think we're more aligned... [7]
8. Open source is dying - Theo - t3․gg: They're already on the line of giving up. They suddenly have more reason to give up. Significantly more reason. That sucks. That has the potential to cause real long-term damage in this industry. This is how things like the XZ back door happen. If you're not familiar with the story, I'll TLDDR quick. XC is a really important compression library used by a ton of open source software, especially in the Linux ecosystem.... [8]
9. Why Github Actually Won - Theo - t3․gg: While GitB Butler is not sponsoring this video, it does have a sponsor. So, let's cut to them really quick. AI has made writing code much easier, but it's also made hiring engineers way harder. Filtering through the total mess of AI slop in your resume pile to find a good engineer is nearly impossible, especially if there isn't a good engineer in that pile in the first place. That's why today's sponsor, G2I, exists..... [9]
10. Amazon bets big on React Native - Theo - t3․gg: Oh boy, another video where Theo glazes React Native. What else would we expect? Okay, hear me out on this one though, cuz it's actually really cool. React Native is a great way to build apps, but it does have its shortcomings. The amount of things you have to include and bundle for every single app does bloat the size of these apps some amount. And I know a lot of developers are unhappy when they point at these apps... [10]
11. Bun got bought by Anthropic (yes really) - Theo - t3․gg: doesn't have Bun or Node installed, it works with native add-ons. It has fast startup and it's easy to distribute. That's why Cloud Code, Factory AI, Open Code, and tons of other things are all built with Bun. And the result of this was that Jared got obsessed with Cloud Code. The GitHub username with the most merge PRs and buns repo is the Claude Codebot. We have it set up in our internal Discord and we use it to he... [11]
12. Delete your CLAUDE.md (and your AGENT.md too) - Theo - t3․gg: that quickly. Hilarious. And now it's exploring. We can press control O to see what it's doing. It looks like it's exploring pretty damn fast. Explore the video pipeline in this codebase thoroughly. I need to understand how videos are uploaded, processed, and stored. The schema for videos in the database, video actions and processing, all that. And it spun up a sub agent to go explore and find this information. Note.... [12]

### q082 PASS

- Prompt: What questions does this video leave unanswered?
- Class: `meta_learning_or_next_step`
- Status: `Completed`
- Score: `3`
- Sources: `12`
- Failure: `-`
- Source videos: Amazon bets big on React Native | Bun got bought by Anthropic (yes really) | Delete your CLAUDE.md (and your AGENT.md too) | JavaScript Frameworks in 2025 | My hot take on image formats | Open source is dying | What happened to me? | Why Github Actually Won

#### Answer

Retrieved evidence for: +{Open source is dead now?} What questions does this video leave unanswered?

The answer model is unavailable, so this fallback lists the highest-ranked saved excerpts instead of synthesizing beyond them.

1. Open source is dying - Theo - t3․gg: escalate because you almost certainly have to use AI to actually scan these PRs. It has a lot of config, enough config that I'm almost certain that this project was vibe coded, but yeah, you get the idea. There are a lot of solutions being made to try and fix these problems. Some of them are going to make it harder for new maintainers to break out, which sucks because we might just have our current maintainers until ... [1]
2. Open source is dying - Theo - t3․gg / Overview: The video presents a comprehensive examination of how AI is negatively impacting the open source ecosystem. The speaker, an experienced open source maintainer and creator of T3 Code, shares firsthand experiences managing a new project that received 150 PRs in just 5 days. The discussion covers four main problem areas: PR spam and quality degradation, increasingly hostile and confused users, GitHub's inadequate platfo... [2]
3. What happened to me? - Theo - t3․gg: of my community, the people hanging out in Twitch chat right now who have been there since day one, watched this whole thing happen, many of which joined me on the same journey. So, knowing all of this, I want to talk a bit about how I choose a topic for a video because another one of the questions I get all of the time is, "What is your dream video that you would do if the algorithm wouldn't kill it?" A question I g... [3]
4. What happened to me? - Theo - t3․gg: plays, I don't film the video. If I don't care, I don't care. You cannot get me to do a video I don't feel like doing. You cannot pay me any amount of money to talk about something I don't want to talk about. Want to know a really funny thing? Probably shouldn't be sharing. In my onboarding email that I send to brands when I'm starting to work with them as sponsors, I have an FAQ section. And one of the questions in.... [4]
5. My hot take on image formats - Theo - t3․gg: and drop it into word it doesn't show up properly if they try to send it as an email attachment it is a file instead of an image these types of things are real complaints and they make a lot of sense but the Alternatives I've seen people propose make none another way to look at this video isn't so much a rant about why I love webp rather this is a rant but why I hate av1 yes I'm going to come out and say it the avif.... [5]
6. Open source is dying - Theo - t3․gg: put so much effort into killing Hacktoberfest. I think this video of mine, don't contribute to open source, is one of the best videos I ever filmed. Not sure who that blonde guy with the mustache is though. Seriously though, like that video, I have been told by so many maintainers how thankful they are for this video more than almost anything I've done as a developer and journalist, YouTuber, whatever you want to cal... [6]
7. JavaScript Frameworks in 2025 - Theo - t3․gg: is absolutely own language it has been for a bit but this is the like tripling down on it I'm kind of disappointed that to my memorization questions on interviews everyone can now just answer with just use the compiler man very fair point you know times our are stupidly so this is why you guys got to watch my react compiler content I go so deep on these things and no one cares it's yeah the I think we're more aligned... [7]
8. Open source is dying - Theo - t3․gg: They're already on the line of giving up. They suddenly have more reason to give up. Significantly more reason. That sucks. That has the potential to cause real long-term damage in this industry. This is how things like the XZ back door happen. If you're not familiar with the story, I'll TLDDR quick. XC is a really important compression library used by a ton of open source software, especially in the Linux ecosystem.... [8]
9. Why Github Actually Won - Theo - t3․gg: While GitB Butler is not sponsoring this video, it does have a sponsor. So, let's cut to them really quick. AI has made writing code much easier, but it's also made hiring engineers way harder. Filtering through the total mess of AI slop in your resume pile to find a good engineer is nearly impossible, especially if there isn't a good engineer in that pile in the first place. That's why today's sponsor, G2I, exists..... [9]
10. Amazon bets big on React Native - Theo - t3․gg: Oh boy, another video where Theo glazes React Native. What else would we expect? Okay, hear me out on this one though, cuz it's actually really cool. React Native is a great way to build apps, but it does have its shortcomings. The amount of things you have to include and bundle for every single app does bloat the size of these apps some amount. And I know a lot of developers are unhappy when they point at these apps... [10]
11. Bun got bought by Anthropic (yes really) - Theo - t3․gg: doesn't have Bun or Node installed, it works with native add-ons. It has fast startup and it's easy to distribute. That's why Cloud Code, Factory AI, Open Code, and tons of other things are all built with Bun. And the result of this was that Jared got obsessed with Cloud Code. The GitHub username with the most merge PRs and buns repo is the Claude Codebot. We have it set up in our internal Discord and we use it to he... [11]
12. Delete your CLAUDE.md (and your AGENT.md too) - Theo - t3․gg: that quickly. Hilarious. And now it's exploring. We can press control O to see what it's doing. It looks like it's exploring pretty damn fast. Explore the video pipeline in this codebase thoroughly. I need to understand how videos are uploaded, processed, and stored. The schema for videos in the database, video actions and processing, all that. And it spun up a sub agent to go explore and find this information. Note.... [12]

### q083 PASS

- Prompt: What follow-up question should I ask after watching this?
- Class: `meta_learning_or_next_step`
- Status: `Completed`
- Score: `3`
- Sources: `10`
- Failure: `-`
- Source videos: Anthropic study shows AI makes devs dumb | Claude Code's latest update is really cool (when it works...) | Claude Cowork: a small taste of AGI | GPT-5.1 is built for normies | I can't believe he was right. | I need you guys to trust me on this (sorry Anthropic) | The drama never ends... | What happened to me? | What happens now?

#### Answer

Retrieved evidence for: What follow-up question should I ask after watching this?

The answer model is unavailable, so this fallback lists the highest-ranked saved excerpts instead of synthesizing beyond them.

1. I need you guys to trust me on this (sorry Anthropic) - Theo - t3․gg / Key Points: ecause labs make hosting nearly impossible through generous subscriptions paired with expensive APIs. Community Confusion and Anthropic's Lack of Clarity The creator has received DMs from many prominent people asking for insights because official answers are nonexistent. Matt Pocock (known for TypeScript work) publicly asked two questions: (1) Can he use an OAuth token from a subscription to power the Claude Agent SD... [1]
2. What happened to me? - Theo - t3․gg: of my community, the people hanging out in Twitch chat right now who have been there since day one, watched this whole thing happen, many of which joined me on the same journey. So, knowing all of this, I want to talk a bit about how I choose a topic for a video because another one of the questions I get all of the time is, "What is your dream video that you would do if the algorithm wouldn't kill it?" A question I g... [2]
3. The drama never ends... - Theo - t3․gg: but it's one I felt I had to. I wanted to do my best to cover this reasonably, and I hope you see that for what it is. Let me know what I did right, and more importantly, what I could do better on. And until next time, peace nerds. [3]
4. Claude Cowork: a small taste of AGI - Theo - t3․gg: thing hard to know for sure seems potentially very good says that co-work can only access files that you grant access to. It looks to me like they're mounting those files in a containerized environment, which should mean we can trust co-work not to be able to access things outside of the sandbox. Here's the reply he got with his question about drafts. Most ready to publish frequently argued questions against LLMs cl.... [4]
5. What happens now? - Theo - t3․gg: complicated, then everyone could be a YouTuber. Cuz that's the hard part. Cuz that's the first problem you ran into. The radio thing even happens to an extent here, too. If the airplane radios were easier, everyone could land the plane. No, you [ __ ] can't. Be realistic here. 34 of men answer yes to this question. Fun fact, the majority of men think they can land the plane. I bring this up because of a real conversa... [5]
6. I need you guys to trust me on this (sorry Anthropic) - Theo - t3․gg: trying to relieve his confusion here and assumed as somebody who just asked nicely that they would get an answer. Clearly not familiar with how Anthropic does things. Again, as always, no shade to Thoric. He has been put between a rock and a hard place here where he clearly wants to do right to the community, but he's not allowed to answer the important hard questions here. Sorry, this has been confusing. I know we s... [6]
7. I can't believe he was right. - Theo - t3․gg / Key Points: an screenshot a problem, show it to an AI, and iterate—potentially never understanding the underlying issue. **Uncertainty about solutions**: The creator expresses genuine confusion about how junior developers should learn now and may create a dedicated follow-up video on this topic. **Recommendations for early engineers**: Read generated code, especially for projects meant to be maintained Use chat apps to ask quest... [7]
8. Claude Code's latest update is really cool (when it works...) - Theo - t3․gg: t has every model in a row showing success fail rates, average time to complete, and average cost. I think I have costs built into here right now. I might not. Write me a plan for implementing all of this. You should probably use ink for the UI UX portion, but I'm down for other suggestions. Okay, it's asking if I want to do plan mode. Cool. We'll do plan mode. Do I want to proceed? Let it read the metrics file. Sure... [8]
9. Anthropic study shows AI makes devs dumb - Theo - t3․gg / Key Points: following a "generation then comprehension" approach. These participants generated code, manually copied/pasted it, then asked follow-up questions to improve understanding. Though not particularly fast, they showed higher quiz scores (65%+). A hybrid approach involving code generation with explanations was also noted, though it took more time. **Study Limitations and Criticisms**: The author critiques the study's des... [9]
10. GPT-5.1 is built for normies - Theo - t3․gg: eople that are currently on 40 and make them go nuts. One of my friends who's like deep in the mental health world here, Jason, said that it's going full therapist mode and he approves of it. Not that like you should use it as an alternative to therapy to be very very explicit and clear, but it's less likely to send you down a really dangerous rabbit hole like the other models previously might have. So again, with 4,... [10]

### q084 PASS

- Prompt: What would be a good next video after this one?
- Class: `recommendation`
- Status: `Completed`
- Score: `3`
- Sources: `12`
- Failure: `-`
- Source videos: AI images just got dangerously good (RIP diffusion??) | I moved off of Next.js | It's finally out!!! (Next.js 15 breakdown) | The drama never ends... | The fastest website ever? | What happened to me? | Why is Next.js so slow??

#### Answer

Retrieved evidence for: What would be a good next video after this one?

The answer model is unavailable, so this fallback lists the highest-ranked saved excerpts instead of synthesizing beyond them.

1. It's finally out!!! (Next.js 15 breakdown) - Theo - t3․gg: using a JS file so if I go to a service like pck thing that was built with creat T3 app we would use JS do types to import at type import next. next config it worked it was fine it wasn't great but now nextjs supports a TS file for the next config and this makes it much easier just have a correctly type next config which is more important because of some of the changes coming here in different ways to add things so n... [1]
2. What happened to me? - Theo - t3․gg: of my community, the people hanging out in Twitch chat right now who have been there since day one, watched this whole thing happen, many of which joined me on the same journey. So, knowing all of this, I want to talk a bit about how I choose a topic for a video because another one of the questions I get all of the time is, "What is your dream video that you would do if the algorithm wouldn't kill it?" A question I g... [2]
3. The fastest website ever? - Theo - t3․gg: what's cool about the app is that it's mostly just plain old idiomatic next for example just using route based code splitting next font for automatic fonts next image for image optimizations server components to prevent the JS size from increasing all the things you normally expect partial pre-rendering for largely static delivery with server side invoked Dynamic Parts yep all cool stuff but as Malta says they do add... [3]
4. The drama never ends... - Theo - t3․gg: but it's one I felt I had to. I wanted to do my best to cover this reasonably, and I hope you see that for what it is. Let me know what I did right, and more importantly, what I could do better on. And until next time, peace nerds. [4]
5. It's finally out!!! (Next.js 15 breakdown) - Theo - t3․gg: without versell or even without serverless ler Rob just did a video showing how to deploy to a VPS with nextjs it was really good but they're also seeing some of the things that they like and expect from next that are harder to do in the environments and they're trying to expose those so it's easier to do stuff like an expire time now being a value in the next config that you can configure or stuff like having better... [5]
6. What happened to me? - Theo - t3․gg: audience didn't like the video, that's why it didn't perform. Oh yeah, maybe I should make a video the audience likes. It really does come down to that. There are layers to this, like is the video clickable? Does it start in a way that's entertaining and interesting enough that you continue to watch from there? All of these pieces are important, but I'm not blocked by the algorithm for making certain content. In fact... [6]
7. What happened to me? - Theo - t3․gg: result the way I think about things has changed. There are different pieces of how I would rank a video idea. Obviously, there's my excitement level. Like how excited am I about this topic? There is unique insights. This is an important one for me. Like do I have anything unique to add? If somebody else has a video on the topic and said everything I would want to say, I don't need to do the video. I do a video when I... [7]
8. The fastest website ever? - Theo - t3․gg: in be really nice they're working on getting all of these snuck in to be actually part of nextjs so if you do want these optimizations by the time you watch this video they might already be in next which is really cool that said they were almost all really easy to implement in your code base with a single 150 line of code file you could get half or more of the things that discussing here which is nuts it's so extensi... [8]
9. What happened to me? - Theo - t3․gg: this can change the same way it changed here. If I start doing more of these types of videos and they perform better than expected, I'll lean more into this. For example, the logging video, we screwed up the export initially and the first version that went up was too short and had most of the content missing. So, we had to re-upload it, which destroys the video performance because a lot of people already saw it, so t... [9]
10. AI images just got dangerously good (RIP diffusion??) - Theo - t3․gg: a good starting point, you can click the create video button and generate a video out of the image. So I generated this one and I then went and generated this video with it, which is good until the laptops start to split and you notice the hands. The other one shakes its camera more than Anthropic does during a marketing post, but you get the idea. It's a much better starting point because the nond diffusion model se... [10]
11. I moved off of Next.js - Theo - t3․gg: on top of Next. In fact, I honestly believe if we waited a few more months and I had pushed the next team a little bit harder on the client side SPA stuff, we would have gotten to a point where I could have removed React Router, removed a bunch of these hacks and been happy. But I also really like Tanner and wanted a fresh start. I was briefly considering building my own framework at this point in time. Taking minima... [11]
12. Why is Next.js so slow?? - Theo - t3․gg: al Ping dashboard, I'm signed in here. I'm going to click the dashboard button now. instantly get a response and then the rest has to flow in. There were points where our infer wasn't in a great state and that next state it would stay in that loading spinner for like two plus seconds. That's not the case anymore because we optimized it but it didn't matter cuz when you clicked a link it would immediately go to the th... [12]

### q085 PASS

- Prompt: Which video in my library best expands on this topic?
- Class: `recommendation`
- Status: `Completed`
- Score: `3`
- Sources: `12`
- Failure: `-`
- Source videos: Boneless UI | Need animations? Use this library. | React 19 is finally out! | React Doesn't Scale | Shadcn just changed forever | Tailwind V4 is WAY better than I expected | The most important function in my codebase | What happened to me?

#### Answer

Retrieved evidence for: Which video in my library best expands on this topic?

The answer model is unavailable, so this fallback lists the highest-ranked saved excerpts instead of synthesizing beyond them.

1. What happened to me? - Theo - t3․gg: of my community, the people hanging out in Twitch chat right now who have been there since day one, watched this whole thing happen, many of which joined me on the same journey. So, knowing all of this, I want to talk a bit about how I choose a topic for a video because another one of the questions I get all of the time is, "What is your dream video that you would do if the algorithm wouldn't kill it?" A question I g... [1]
2. Need animations? Use this library. - Theo - t3․gg / Overview: This video covers a major announcement in the web animation ecosystem: Framer Motion, the popular React animation library with over 4.5 million weekly npm downloads, has become an independent open-source project simply called "Motion." The creator, Matt, has left the Framer company after six years to maintain the library independently with Framer's blessing. This separation clarifies the confusing relationship betwee... [2]
3. The most important function in my codebase - Theo - t3․gg / Overview: This video explores the critical problem of error handling in TypeScript and presents three progressively sophisticated solutions. The speaker begins by explaining why TypeScript's default `try/catch` pattern fails to provide type safety for errors, then demonstrates a custom wrapper function that forces explicit error handling. The discussion expands to cover `neverthrow`, a library that implements Result types for.... [3]
4. What happened to me? - Theo - t3․gg: this can change the same way it changed here. If I start doing more of these types of videos and they perform better than expected, I'll lean more into this. For example, the logging video, we screwed up the export initially and the first version that went up was too short and had most of the content missing. So, we had to re-upload it, which destroys the video performance because a lot of people already saw it, so t... [4]
5. Shadcn just changed forever - Theo - t3․gg: TL;DR Shadcn introduced "Shadcn Create," a major new customization system that lets developers build their own themed component library instead of using default styles. The new system is built on Base UI primitives instead of Radix UI, though users can switch between the two. Developers can now customize base component library, preset style, color palette, fonts, border radius, icon sets, and accent styles before gen... [5]
6. What happened to me? - Theo - t3․gg: audience didn't like the video, that's why it didn't perform. Oh yeah, maybe I should make a video the audience likes. It really does come down to that. There are layers to this, like is the video clickable? Does it start in a way that's entertaining and interesting enough that you continue to watch from there? All of these pieces are important, but I'm not blocked by the algorithm for making certain content. In fact... [6]
7. Need animations? Use this library. - Theo - t3․gg: TL;DR Framer Motion has been spun out as an independent open-source project called "Motion," separating from the Framer company to serve the broader web development community beyond just React. The new Motion library introduces vanilla JavaScript APIs, making its animation capabilities available to all frameworks (Vue, Svelte, Angular, etc.), not just React. Motion has a new dedicated homepage at motion.dev featuring... [7]
8. Boneless UI - Theo - t3․gg: ment, styling, and markup—to build custom design systems. Native HTML and CSS are advancing (popover, anchor, dialog, view transitions) to handle functionality that previously required JavaScript. Overview The video discusses an article by Adam that categorizes modern UI component libraries into four playful but descriptive categories: headless, boneless, skinless, and lifeless. The speaker clarifies that these are n... [8]
9. The most important function in my codebase - Theo - t3․gg: ing type-safe error handling with TypeScript's type narrowing. Three solutions for typed error handling exist on a spectrum: the custom `try-catch` wrapper (lowest friction, copy-paste solution), `neverthrow` (library-based Result type that integrates with TypeScript), and Effect.ts (a paradigm-shifting approach that's essentially its own language). `neverthrow` uses a Result type pattern where functions always retur... [9]
10. Tailwind V4 is WAY better than I expected - Theo - t3․gg: alues now work without brackets for numeric inputs (e.g., `h-54`), gradients support angles, and new utility variants like `@min`, `@max`, `group-has`, `not`, and descendant selectors have been added. Overview This video provides an extensive, hands-on review of the newly released Tailwind V4 beta, a major version representing a complete rewrite of the framework's engine. The host explores the transition to a Rust-ba... [10]
11. React Doesn't Scale - Theo - t3․gg: TL;DR A viral Reddit post claims React codebases become disorganized messes at scale, with very few senior engineers truly understanding the library; the video analyzes these claims in depth. The presenter argues most React problems stem from developer inexperience and wrong mental models (especially OOP/class-based thinking), not the framework itself. Key React issues discussed: misuse of `useEffect`, `useState`, `u... [11]
12. React 19 is finally out! - Theo - t3․gg: ML-level concerns. The React Compiler is entirely client-focused and eliminates the need for manual memoization, improving client-side performance significantly without requiring server-side patterns. Overview This video covers the official stable release of React 19, detailing the major changes that delayed the release and the new capabilities the framework now offers. The presenter explains the technical resolution... [12]

### q086 PASS

- Prompt: Which video in my library best challenges this topic?
- Class: `comparison`
- Status: `Completed`
- Score: `3`
- Sources: `12`
- Failure: `-`
- Source videos: Can we put Rust in Angular to make it faster? WASM deep dive | How JS ruined the web | I can't take it anymore. | JavaScript Frameworks in 2025 | Okay, I'm a bit scared now... | Opus 4.6 Is The Best Coding Model Ever Made* | Rate Limiting | Tanner just fixed forms (I'm so hyped) | WWDC was weird. | What happened to me? | You’re all wrong

#### Answer

Retrieved evidence for: Which video in my library best challenges this topic?

The answer model is unavailable, so this fallback lists the highest-ranked saved excerpts instead of synthesizing beyond them.

Comparison frame: both the listed excerpts and their source videos are relevant candidates, while the exact similarities, differences, or counterarguments should be checked against the cited text below.

1. WWDC was weird. - Theo - t3․gg: that showed incorrect icon sizing and alignment as supposed proof of iOS fidelity. **Speaker's critique of Flutter**: The speaker identifies as "Flutter's number one hater who uses accessibility as their main argument," expressing frustration that Flutter's attempt to show improved iOS styling had obvious errors like wrong icon sizes and misalignment. Developer Tools and Open-Source Initiatives **Swift-based contain.... [1]
2. What happened to me? - Theo - t3․gg: of my community, the people hanging out in Twitch chat right now who have been there since day one, watched this whole thing happen, many of which joined me on the same journey. So, knowing all of this, I want to talk a bit about how I choose a topic for a video because another one of the questions I get all of the time is, "What is your dream video that you would do if the algorithm wouldn't kill it?" A question I g... [2]
3. How JS ruined the web - Theo - t3․gg / Key Points: culture often rewards complexity over actual user benefit—engineers who write 30-page proposals for regressions get promoted while those who simply fix problems don't. This cultural problem existed before modern frameworks; the speaker recounts experiences at Twitch where blog managers couldn't embed simple videos correctly despite using WordPress. The problem isn't the tools but rather the culture that incentivizes.... [3]
4. Okay, I'm a bit scared now... - Theo - t3․gg / Key Points: also produced a correct answer (139 and ending in 662). This success rate deeply concerns the creator about the future viability of programming competitions. **Potential Training Data Concern**: The creator raises the possibility that solutions might have been trained on existing publicly available Advent of Code solutions, since participants typically open-source their solutions after competitions end. The creator p... [4]
5. Rate Limiting - Theo - t3․gg: posts? So cool. So yeah, the pros are that this is simple to implement and understand and it's predictable. Predictability is not necessarily a good thing, as I'm sure we'll get into. The con is that this allows for bursts up to x the limit. So yeah, if we're getting near the end here, that once we get close to the end, I could spam and then immediately start getting requests going through again. So you actually can ... [5]
6. You’re all wrong - Theo - t3․gg: our two groups. Sky is blue, sky is gray. We split this. Sky is blue. This group they read about blue skies. This group reads about gray skies and then groups three and four we swap. What do you think happens if you ask each of these people before and after reading how strongly do they feel about this belief? So I am six out of 10 sure the sky is blue. You have this person they say this and then you give them an arti... [6]
7. Opus 4.6 Is The Best Coding Model Ever Made* - Theo - t3․gg: tokens — 2-4x more expensive than GPT 5/5.1, roughly 2x more than GPT 5.2/5.2 Codex. New features include team orchestration with parallel agents in Claude Code and API "effort levels" for reasoning intensity. Downsides noted: the model feels slower (5-10 minutes vs 1-2 minutes for tasks), less pleasant to interact with (more templated responses), and still makes "dumb" mistakes like reporting placeholder credentials... [7]
8. I can't take it anymore. - Theo - t3․gg / Overview: This video is a comprehensive, emotionally charged critique of Apple from a content creator who identifies as both a longtime Apple user and someone who has grown increasingly frustrated with the company's direction. The speaker structures his grievances into three categories: software quality, company policy, and ignorance. He provides detailed examples of software bugs that have persisted for years, criticizes Appl... [8]
9. Tanner just fixed forms (I'm so hyped) - Theo - t3․gg / Key Points: ased on form state), improved debuggability. **Drawbacks of controlled in React Native**: Can cause "sticky keys" problem where typing lags because React takes too long to update. The speaker references a past video with Dan Abramov discussing React Native input issues. The library team acknowledges React Native's unique challenges with controlled inputs in their documentation. Philosophy and API Design Principles **... [9]
10. JavaScript Frameworks in 2025 - Theo - t3․gg: side complexity while ignoring server-side tradeoffs; similarly, the shift from SPA to isomorphic models exposes frontend devs to complexity they previously ignored. React Compiler and Svelte 5 Runes represent opposing compiler philosophies—React Compiler auto-optimizes by adding memoization, while Svelte trades minimal syntax for more expressive reactivity—ironically both frameworks have traded their original philos... [10]
11. Can we put Rust in Angular to make it faster? WASM deep dive - Theo - t3․gg: TL;DR The video explores integrating Rust-compiled WebAssembly into Angular applications for performance-critical tasks like heavy data processing, numbers, video encoding, and image editing. WebAssembly is not a replacement for JavaScript frameworks; DOM bindings remain a bottleneck, and binary sizes can be problematic. It excels at input-to-output transformations. The host attempts to replicate an Angular+Rust tuto... [11]
12. Rate Limiting - Theo - t3․gg: for this. You don't have Twitter. That's for the better. Do you at least have more blog posts? Need somewhere to point people at? Oh, that's actually hilarious. Email and password off should be a last resort. Couldn't agree more. Seems like we're going to agree about a lot of things by the looks of this. Yeah, hilariously well timed. Regardless, take a look at Smudgeai. Fantastic stuff. Love this blog. really nice to... [12]

### q087 PASS

- Prompt: What is the overall tone of this video?
- Class: `tone_or_style_inference`
- Status: `Completed`
- Score: `3`
- Sources: `12`
- Failure: `-`
- Source videos: Defending a disaster (modern frontend development rant) | I hate that this is still happening | Open source is dying | Sonnet 4.5 is the best coding model in the world

#### Answer

Retrieved evidence for: +{Open source is dead now?} What is the overall tone of this video?

The answer model is unavailable, so this fallback lists the highest-ranked saved excerpts instead of synthesizing beyond them.

From the available evidence, these excerpts support only a tentative reading rather than a definitive judgment.

1. Open source is dying - Theo - t3․gg: escalate because you almost certainly have to use AI to actually scan these PRs. It has a lot of config, enough config that I'm almost certain that this project was vibe coded, but yeah, you get the idea. There are a lot of solutions being made to try and fix these problems. Some of them are going to make it harder for new maintainers to break out, which sucks because we might just have our current maintainers until ... [1]
2. Defending a disaster (modern frontend development rant) - Theo - t3․gg: ning Astro and Tailwind. The discussion covers SSR history, CSS methodologies, framework evolution, build complexity, and the "Alex Russell problem" (judging technologies by worst-case implementations). Theo's overall assessment: the author's diagnosis of problems sometimes aligns, but solutions and alternatives are outdated or overlook why modern tools became popular. Overview This video is a detailed reaction by Th... [2]
3. Sonnet 4.5 is the best coding model in the world - Theo - t3․gg: too many bullet points, but generally I found the tone of clawed models to be really good. And if you want a UI that's as good as its tone, check out T3 Chat where you get access to literally every single model for eight bucks a month. It's a pretty absurd deal if you ask me. And I'll make it a little bit more absurd. Use code kind of safe at checkout and you'll get your first month for just $1. Go to t3.hat and chec... [3]
4. Open source is dying - Theo - t3․gg / Overview: The video presents a comprehensive examination of how AI is negatively impacting the open source ecosystem. The speaker, an experienced open source maintainer and creator of T3 Code, shares firsthand experiences managing a new project that received 150 PRs in just 5 days. The discussion covers four main problem areas: PR spam and quality degradation, increasingly hostile and confused users, GitHub's inadequate platfo... [4]
5. Open source is dying - Theo - t3․gg: put so much effort into killing Hacktoberfest. I think this video of mine, don't contribute to open source, is one of the best videos I ever filmed. Not sure who that blonde guy with the mustache is though. Seriously though, like that video, I have been told by so many maintainers how thankful they are for this video more than almost anything I've done as a developer and journalist, YouTuber, whatever you want to cal... [5]
6. Open source is dying - Theo - t3․gg: poorest. Really crippled the image I had in relation to your channel and content. I have been contributing to projects since before you were born. such an attitude, including some YouTube codes of yours that I've never received before. Feel free to block me. Mature. This is somebody who never wrote code before AI. Straight up. And I promise you, you were not contributing to code years ago, [ __ ] And this is the hot ... [6]
7. I hate that this is still happening - Theo - t3․gg: especially in India. So maybe go do that. That's all I got on this one. Please stop spamming these repos with open source. And if you harass anyone as a result of this video, just know I'll be disappointed as [ __ ] I got nothing else to say on this. Let me know what you guys think. And until next time, peace nerds. [7]
8. I hate that this is still happening - Theo - t3․gg: use to make them is very different from the tech I started with. The best thing to make your first video with is the things you already have. You shouldn't buy a bunch of new stuff to inspire you to make the first video. You should do it despite not having the right equipment. And once you get good at it, you'll figure out what your equipment can and can't do and make changes based on what you know. And this is the r... [8]
9. I hate that this is still happening - Theo - t3․gg / TL;DR: A Git/GitHub tutorial by Apna College (6-7 million views) continues to cause thousands of spam PRs on the ExpressJS repository, wasting maintainer time. The video creator argues that Apna College's response has been inadequate—only editing out ~5 seconds to 1 minute after years of damage, and deflecting blame onto students. Open source contribution is being misunderstood as a "magic gateway" to jobs; the video explai... [9]
10. I hate that this is still happening - Theo - t3․gg: Update readme.md. Action. Update readme.md. Naveen kumar. Update readme.md. Ria. Update readme.momd. Update again readme.md. Update readme.momd. Update readme. Update readme. Update readme. Update readme. Update readme. Update readme. I'm going to go actually insane. For those who haven't been around for a long time, I'm Theo. I make videos about software dev stuff. I care a lot about open source, which is why this i... [10]
11. Open source is dying - Theo - t3․gg: want to highlight one particular PR that annoyed me. We had a stailed to-do MD file in the repo that had random things I was working on at some point in it. And someone filed a nonsense PR that tried to fix all of those things and ended up just breaking other things. They didn't get any response from us cuz we were being flooded with PRs. So he randomly tags me and two other people whose PRs merged recently. I was so... [11]
12. Open source is dying - Theo - t3․gg: They're already on the line of giving up. They suddenly have more reason to give up. Significantly more reason. That sucks. That has the potential to cause real long-term damage in this industry. This is how things like the XZ back door happen. If you're not familiar with the story, I'll TLDDR quick. XC is a really important compression library used by a ton of open source software, especially in the Linux ecosystem.... [12]

### q088 PASS

- Prompt: Is the speaker confident, cautious, or speculative?
- Class: `tone_or_style_inference`
- Status: `Completed`
- Score: `3`
- Sources: `1`
- Failure: `-`
- Source videos: I can't believe he was right.

#### Answer

Retrieved evidence for: Is the speaker confident, cautious, or speculative?

The answer model is unavailable, so this fallback lists the highest-ranked saved excerpts instead of synthesizing beyond them.

From the available evidence, these excerpts support only a tentative reading rather than a definitive judgment.

1. I can't believe he was right. - Theo - t3․gg: as I do today, even if my relationship with it is very different than it was a year ago. And I recommend that you reflect yourself and give these things a try. Let me know what y'all think and how you're using these tools today. [1]

### q089 PASS

- Prompt: Does this video sound more like a tutorial, a review, or a discussion?
- Class: `tone_or_style_inference`
- Status: `Completed`
- Score: `3`
- Sources: `12`
- Failure: `-`
- Source videos: Gemini Flash 3 is my new favorite model (yes really) | I need you guys to trust me on this (sorry Anthropic) | Okay, I'm a bit scared now... | Open source is dying | React feels insane | This magic hack makes Next.js possible | We need to talk about Sonnet 4.6 | What happened to me? | Why I moved away from SQL

#### Answer

Retrieved evidence for: +{Open source is dead now?} Does this video sound more like a tutorial, a review, or a discussion?

The answer model is unavailable, so this fallback lists the highest-ranked saved excerpts instead of synthesizing beyond them.

From the available evidence, these excerpts support only a tentative reading rather than a definitive judgment.

1. Open source is dying - Theo - t3․gg: escalate because you almost certainly have to use AI to actually scan these PRs. It has a lot of config, enough config that I'm almost certain that this project was vibe coded, but yeah, you get the idea. There are a lot of solutions being made to try and fix these problems. Some of them are going to make it harder for new maintainers to break out, which sucks because we might just have our current maintainers until ... [1]
2. Open source is dying - Theo - t3․gg / Overview: The video presents a comprehensive examination of how AI is negatively impacting the open source ecosystem. The speaker, an experienced open source maintainer and creator of T3 Code, shares firsthand experiences managing a new project that received 150 PRs in just 5 days. The discussion covers four main problem areas: PR spam and quality degradation, increasingly hostile and confused users, GitHub's inadequate platfo... [2]
3. What happened to me? - Theo - t3․gg: of my community, the people hanging out in Twitch chat right now who have been there since day one, watched this whole thing happen, many of which joined me on the same journey. So, knowing all of this, I want to talk a bit about how I choose a topic for a video because another one of the questions I get all of the time is, "What is your dream video that you would do if the algorithm wouldn't kill it?" A question I g... [3]
4. Okay, I'm a bit scared now... - Theo - t3․gg: take your ideas and turn them into reality okay so there was approximately 20 seconds of showing anything related to the 01 Mini model in this and then a lot of just talking it does fit the AI way which is using way too many words for the thing you're trying to do good old delve yeah I love that I love that Paul Graham keeps getting proven Ming more and more right anyways oh they have an actual coding demo at the bot... [4]
5. Open source is dying - Theo - t3․gg: even more interesting is the content of these questions. It feels like the questions I've been getting are different now where I'm suddenly getting like random Twitch streamers hitting me up about their vibecoded chat app for not chatting with AI but having their chat shown in their stream or I'm talking to people who I used to work with at Twitch that were more on the product side that are building their own solutio... [5]
6. I need you guys to trust me on this (sorry Anthropic) - Theo - t3․gg: This video is not legal advice. I am not responsible for Anthropic's actions and what they do to your account. If you do get banned by using T3 Code, I'm incredibly sorry and I want you to tell me every single thing about that ban so we can make sure it doesn't happen to others. But I cannot control what Anthropic does and their ability to ban you for anything they feel like at any time is not something I can make an... [6]
7. Open source is dying - Theo - t3․gg: we all are nerdy about and care about. I bring this up because there's a couple things that we just experience in life differently because of that. The one I'm imagining right now, and I'm sure a lot of y'all are this one's in chat if you can relate. I used to get a lot of texts from family members, random friends in high school and just people in my life asking random [ __ ] about computers. Anything from, "Can you ... [7]
8. We need to talk about Sonnet 4.6 - Theo - t3․gg: crap and hire great people at soy. To put it simply, Anthropic's playing really dirty with how they're doing stuff. Today's drama that inspired this video is one that actually might affect me in the near future. They made a change to the Claude Code policy around using OOTH tokens obtained through Claude Code subscriptions in other products, tools, or services, including the agent SDK, not being permitted. What does.... [8]
9. This magic hack makes Next.js possible - Theo - t3․gg: next Chase is a complex beast but it is getting simpler I know that sounds a bit contradictory in a video titled the magic powering next like isn't whole Magic just hidden complexity kind of but what if one of these magic things is actually just how JavaScript works and on top of that if this magic thing could make a lot of the other magic that you were doing before go away so everything is much simpler in the end th... [9]
10. Why I moved away from SQL - Theo - t3․gg: apps. I wrote this proposal 4 years ago. Well, give me these data if I hover. Yeah, this was July of 2021. This was right as I was starting to build what became ping.gg, the video call app. And I was getting annoyed about having to write an endpoint and then call it with React Query and not know about the shape of things. I had come fresh out of Twitch where we were using GraphQL for everything and I was using codege... [10]
11. React feels insane - Theo - t3․gg: variable and whenever it changes all the places in the UI are updated. This works really well. Later people started nagging that this omnidirectional data flow is bad. There was a push to a oneway top to bottom bindings instead which does sound technically better but in practice it makes everything more complicated and started a strand of discussion which ended with us all having to use Redux today. So thanks. You we... [11]
12. Gemini Flash 3 is my new favorite model (yes really) - Theo - t3․gg: it doesn't need them. Yeah, you get the idea. Supposedly, people are using this heavily for game development stuff. Astrocade is a company trying to build an AI game production mini studio thing like lovable but for making games and they moved to three flash for the game creation engine and are surprised with how well it's performing again with the spatial awareness wins. I could see that making sense. Gemini 3 flash... [12]

### q090 PASS

- Prompt: What are the recurring terms in this channel's videos?
- Class: `topic_aggregation`
- Status: `Completed`
- Score: `3`
- Sources: `12`
- Failure: `-`
- Source videos: A.I. Backlash Turns Violent + Kara Swisher on Healthmaxxing + The Zuck Bot Is Coming | Anthropic’s Cybersecurity Shock Wave + Ronan Farrow and Andrew Marantz on Their Sam Altman Investigation + One Good Thing | What’s a Hard Fork?

#### Answer

Retrieved evidence for: @{Hard Fork} What are the recurring terms in this channel's videos?

The answer model is unavailable, so this fallback lists the highest-ranked saved excerpts instead of synthesizing beyond them.

1. What’s a Hard Fork? - Hard Fork / At a glance: The video title asks "What’s a Hard Fork?", but the transcript contains no substantive content on this topic. The provided transcript is solely an automated speech recognition (ASR) system metadata note. The text indicates it originated from a local OpenAI-compatible ASR endpoint, not from official RSS show notes. [1]
2. What’s a Hard Fork? - Hard Fork: Podcast ASR smoke transcript. This text came from the local OpenAI-compatible ASR endpoint, not from RSS show notes. [2]
3. What’s a Hard Fork? - Hard Fork / Overview: The video is titled "What’s a Hard Fork?", suggesting an educational focus on blockchain or software development concepts. However, the actual transcript provides no information on this subject. It consists entirely of an ASR metadata placeholder stating the text was generated by a local OpenAI-compatible ASR endpoint rather than sourced from RSS show notes. [3]
4. What’s a Hard Fork? - Hard Fork / Takeaways: The intended educational content regarding what a hard fork is could not be summarized, as the transcript contains only ASR metadata and no actual discussion. [4]
5. Anthropic’s Cybersecurity Shock Wave + Ronan Farrow and Andrew Marantz on Their Sam Altman Investigation + One Good Thing - Hard Fork: be beautiful wherever you happen to be. Wow. They'll send you an umbrella reminder if it's going to precipitate in the next 12 hours and they'll send you a sunscreen alert when the UV index is high. But I'm saving my last two favorites for the end. Number one, they will send you an alert when the Aurora Borealis may be visible where you are. That's beautiful. I haven't gotten that notification yet, but I wake up ever... [5]
6. Anthropic’s Cybersecurity Shock Wave + Ronan Farrow and Andrew Marantz on Their Sam Altman Investigation + One Good Thing - Hard Fork: forced reset for the entire cybersecurity industry and a very significant event in the history of technology. Yeah. Well, and just to make it concrete, we are currently at war with Iran and Iran is currently hacking our critical infrastructure. There's a story in Wired this week about them successfully hacking like water and energy infrastructure. Right now they're able to do that without a mythos quality model. I wo... [6]
7. Anthropic’s Cybersecurity Shock Wave + Ronan Farrow and Andrew Marantz on Their Sam Altman Investigation + One Good Thing - Hard Fork: writer at The New Yorker. They worked on this piece for a very long time, talked to many, many people in and around Sam's orbit and tried to answer the question of like, who is this guy? Yeah. And also why does that matter? Right? We're talking during a week where these systems have arguably experienced a step change in what they can do. And I think those kind of advances just naturally should draw more scrutiny onto... [7]
8. A.I. Backlash Turns Violent + Kara Swisher on Healthmaxxing + The Zuck Bot Is Coming - Hard Fork: is going to remember and it is going to send nasty Nancy to your house. Not nasty Nancy. To teach you a lesson. Well, Casey, do you think that the Mark Zuckerberg AI clone is going to suffer the same fate as the Snoop Dogg and Tom Brady clones? Or do you think this is going to be an enduring management tactic? You know, it's hard to say at this moment. I think we won't really know how successful it's going to be unti... [8]
9. Anthropic’s Cybersecurity Shock Wave + Ronan Farrow and Andrew Marantz on Their Sam Altman Investigation + One Good Thing - Hard Fork: to see there's a range of reactions, right? There's people who have answered that question in a very severe way and looked at the fact pattern that is laid out here and the documentation. that's laid out and said, you know, this is someone who poses an acute danger and should be kept away from an authority position. And then there's people who I mean, hilariously enough, my mother called me and she's like, you know, ... [9]
10. A.I. Backlash Turns Violent + Kara Swisher on Healthmaxxing + The Zuck Bot Is Coming - Hard Fork: I'm Dane Bruggler. I cover the NFL draft for the Athletic. Our draft guide picked up the name "The Beast" because of the crazy amount of information that's included. I'm looking at thousands of players putting together hundreds of scouting reports. I've been covering this year's draft since last year's draft. There is a lot in the beast that you simply can't find anywhere else. This is the kind of in-depth, unique jo... [10]
11. A.I. Backlash Turns Violent + Kara Swisher on Healthmaxxing + The Zuck Bot Is Coming - Hard Fork: mRNA vaccines and AI looking at gene folding. So there was all this real stuff and all this really ridiculous stuff. Right. And so you said sort of like I'm saying a lot of stuff that seems like obviously wrong, but some stuff that seems actually promising. So I want to spend some time and see if I can sort of separate the wheat from the chaff. Right. And I also need to do the stunts because it's funny, right? Like d... [11]
12. A.I. Backlash Turns Violent + Kara Swisher on Healthmaxxing + The Zuck Bot Is Coming - Hard Fork: about it and recently, more recently has admitted it. And so a lot of them were using it for optimization, not depression, but optimization. And this guy was using it for new ideas in his entrepreneurial journey. So... And did you have a lot of new ideas on ketamine when you tried that? I had none. I thought only about you. Kasey naked is what I thought. No. No, have you ever, either of you used it? I have tried keta... [12]

### q091 PASS

- Prompt: Which videos are most similar based on content?
- Class: `comparison`
- Status: `Completed`
- Score: `3`
- Sources: `12`
- Failure: `-`
- Source videos: Google Drive hates developers now | How Minecraft AI ACTUALLY works | JavaScript might get compiler hints? | Life after TypeScript | My hot take on image formats | NVIDIA's first real competition (Google is KILLING it) | OpenAI’s TikTok Clone Is Interesting… | What happened to me?

#### Answer

Retrieved evidence for: Which videos are most similar based on content?

The answer model is unavailable, so this fallback lists the highest-ranked saved excerpts instead of synthesizing beyond them.

Comparison frame: both the listed excerpts and their source videos are relevant candidates, while the exact similarities, differences, or counterarguments should be checked against the cited text below.

1. Life after TypeScript - Theo - t3․gg: these types of things when there's an ecosystem that has it all solved already for the scale they're at, especially because they're targeting these types of enterprises. Point two for why they picked C# is the similarity to TypeScript. I wonder why these two languages were created by the same person. Oh, that might be why. It's only natural they look and feel similar as many people are starting to realize. Heck, Micr... [1]
2. OpenAI’s TikTok Clone Is Interesting… - Theo - t3․gg / Key Points: on Underlying Generation Method**: The creator theorizes that Sora 2 isn't just a single video model but rather an LLM generating screenplays and plans that command other models to generate video pieces, which are then stitched together. This explains how videos exceed the typical 5-second limit seen in other video models. The model appears to generate audio first, then create video scenes, then use software to edit.... [2]
3. How Minecraft AI ACTUALLY works - Theo - t3․gg / Key Points: eo encoding parallel**: The method mirrors video compression techniques. Instead of encoding complete frames for every moment, video uses motion vectors and "diffs" (differences between frames) to reduce data. Most pixels in consecutive frames don't change, so only deltas are encoded. **Why this works for AI models**: Processing motion/delta data is significantly less computationally intensive than processing full 3D... [3]
4. Google Drive hates developers now - Theo - t3․gg: have seen their announcement that they're stopping development of their Android version for similar reasons our experience was different but our circumstances are similar while Google Drive may not be the most popular option for Connection in transmit we know many users rely on it and we often use it here at Panic to send and receive files from game devs that we work with it's saw a decision that we took lightly and.... [4]
5. JavaScript might get compiler hints? - Theo - t3․gg: one. You can't CSS or style an element that's not there anymore. The React team could have put a lot of time trying to hack around that, but instead they waited for view transitions to happen. View transitions are a browser standard that is focused on making it easier to transition elements both between pages during navigation and in and out on the browser in a given window when elements are added and removed. I did.... [5]
6. OpenAI’s TikTok Clone Is Interesting… - Theo - t3․gg / Overview: This video provides an extensive critical analysis of OpenAI's new Sora app—a mobile video generation platform that combines AI video creation with a TikTok-style social feed. The creator, who has early access, spent an entire day testing the platform and hit the 50-video daily rate limit. With deep experience in both AI development and professional video production, the creator offers a multi-faceted critique coveri... [6]
7. What happened to me? - Theo - t3․gg: this can change the same way it changed here. If I start doing more of these types of videos and they perform better than expected, I'll lean more into this. For example, the logging video, we screwed up the export initially and the first version that went up was too short and had most of the content missing. So, we had to re-upload it, which destroys the video performance because a lot of people already saw it, so t... [7]
8. What happened to me? - Theo - t3․gg: of my community, the people hanging out in Twitch chat right now who have been there since day one, watched this whole thing happen, many of which joined me on the same journey. So, knowing all of this, I want to talk a bit about how I choose a topic for a video because another one of the questions I get all of the time is, "What is your dream video that you would do if the algorithm wouldn't kill it?" A question I g... [8]
9. Google Drive hates developers now - Theo - t3․gg: quite as much nowadays but uh prompt was a huge app for doing SSH and connecting to an external terminal on your iOS devices it's like how you would do Dev in quotes on mobile but also transmit and Nova which are were and still kind of are really popular solutions for devs on Mac this was an IDE that was native really cool it's been trumped since but Nova was dope and transmit is still one of the best SFTP and FTP c.... [9]
10. JavaScript might get compiler hints? - Theo - t3․gg: because view transitions are a browser standard that solved that problem because the problem lived in the browser. This is similar. Could the React team have done something crazy with resumability similar to what Quick does? Probably. It would have been hellish, but they could have done it. Instead, I am almost positive they nudged the V8 and Chrome teams to figure out how to do a better experience in the browser for... [10]
11. NVIDIA's first real competition (Google is KILLING it) - Theo - t3․gg / Key Points: xtensively. Google is the only company operating across all four layers simultaneously: apps (Google Search AI overviews, Gemini), foundation models (Gemini models), cloud inference (GCP), and accelerator hardware (TPUs). The AI overview in Google Search is identified as the most popular AI application in the world by usage. Google's TPU Strategy and Ironwood Announcement Google announced Ironwood, their seventh-gene... [11]
12. My hot take on image formats - Theo - t3․gg / Key Points: at's "progressive decoding" feature—loading a low-resolution version first and enhancing progressively—is technically impressive but CPU-intensive. Only Safari supports JPEG XL; Firefox and Chrome do not. Software support for JPEG XL is poor, with few programs able to open the files. The format's complexity and lack of adoption make it unsuitable for mainstream use despite its theoretical advantages for high-fidelity... [12]

### q092 PASS

- Prompt: Which videos are closest in theme to this one?
- Class: `comparison`
- Status: `Completed`
- Score: `3`
- Sources: `2`
- Failure: `-`
- Source videos: I built the same app with 5 different stacks | What happened to me?

#### Answer

Retrieved evidence for: Which videos are closest in theme to this one?

The answer model is unavailable, so this fallback lists the highest-ranked saved excerpts instead of synthesizing beyond them.

Comparison frame: both the listed excerpts and their source videos are relevant candidates, while the exact similarities, differences, or counterarguments should be checked against the cited text below.

1. I built the same app with 5 different stacks - Theo - t3․gg: TL;DR The author built the same "roundest Pokémon" voting app with five different technology stacks they've used throughout their career: Rails, Elixir/Phoenix, Go/GraphQL/React SPA, T3 Stack (Next.js Pages Router), and Next.js App Router with React Server Components. Elixir/Phoenix with LiveView achieved the fastest performance through WebSocket-based diffs and preloading, followed closely by the optimized RSC versi... [1]
2. What happened to me? - Theo - t3․gg: of my community, the people hanging out in Twitch chat right now who have been there since day one, watched this whole thing happen, many of which joined me on the same journey. So, knowing all of this, I want to talk a bit about how I choose a topic for a video because another one of the questions I get all of the time is, "What is your dream video that you would do if the algorithm wouldn't kill it?" A question I g... [2]

### q093 PASS

- Prompt: What patterns do you notice across summaries from this channel?
- Class: `cross_video_synthesis`
- Status: `Completed`
- Score: `3`
- Sources: `12`
- Failure: `-`
- Source videos: A.I. Backlash Turns Violent + Kara Swisher on Healthmaxxing + The Zuck Bot Is Coming | Anthropic’s Cybersecurity Shock Wave + Ronan Farrow and Andrew Marantz on Their Sam Altman Investigation + One Good Thing | What’s a Hard Fork?

#### Answer

Retrieved evidence for: @{Hard Fork} What patterns do you notice across summaries from this channel?

The answer model is unavailable, so this fallback lists the highest-ranked saved excerpts instead of synthesizing beyond them.

1. What’s a Hard Fork? - Hard Fork / At a glance: The video title asks "What’s a Hard Fork?", but the transcript contains no substantive content on this topic. The provided transcript is solely an automated speech recognition (ASR) system metadata note. The text indicates it originated from a local OpenAI-compatible ASR endpoint, not from official RSS show notes. [1]
2. Anthropic’s Cybersecurity Shock Wave + Ronan Farrow and Andrew Marantz on Their Sam Altman Investigation + One Good Thing - Hard Fork: be beautiful wherever you happen to be. Wow. They'll send you an umbrella reminder if it's going to precipitate in the next 12 hours and they'll send you a sunscreen alert when the UV index is high. But I'm saving my last two favorites for the end. Number one, they will send you an alert when the Aurora Borealis may be visible where you are. That's beautiful. I haven't gotten that notification yet, but I wake up ever... [2]
3. What’s a Hard Fork? - Hard Fork / Takeaways: The intended educational content regarding what a hard fork is could not be summarized, as the transcript contains only ASR metadata and no actual discussion. [3]
4. What’s a Hard Fork? - Hard Fork / Overview: The video is titled "What’s a Hard Fork?", suggesting an educational focus on blockchain or software development concepts. However, the actual transcript provides no information on this subject. It consists entirely of an ASR metadata placeholder stating the text was generated by a local OpenAI-compatible ASR endpoint rather than sourced from RSS show notes. [4]
5. Anthropic’s Cybersecurity Shock Wave + Ronan Farrow and Andrew Marantz on Their Sam Altman Investigation + One Good Thing - Hard Fork: to see there's a range of reactions, right? There's people who have answered that question in a very severe way and looked at the fact pattern that is laid out here and the documentation. that's laid out and said, you know, this is someone who poses an acute danger and should be kept away from an authority position. And then there's people who I mean, hilariously enough, my mother called me and she's like, you know, ... [5]
6. What’s a Hard Fork? - Hard Fork: Podcast ASR smoke transcript. This text came from the local OpenAI-compatible ASR endpoint, not from RSS show notes. [6]
7. Anthropic’s Cybersecurity Shock Wave + Ronan Farrow and Andrew Marantz on Their Sam Altman Investigation + One Good Thing - Hard Fork / At a glance: Anthropic announced "Claude Mythos Preview," a highly capable new AI model withheld from the public due to severe cybersecurity risks, instead providing access to a defensive tech consortium. The model can autonomously find zero-day exploits in critical open-source infrastructure (e.g., OpenBSD, FFmpeg) that have evaded human researchers and automated tools for decades. The hosts argue this is not a marketing stunt, ... [7]
8. Anthropic’s Cybersecurity Shock Wave + Ronan Farrow and Andrew Marantz on Their Sam Altman Investigation + One Good Thing - Hard Fork / Key Points: Anthropic's Claude Mythos Preview and the Cybersecurity Shock Wave **Project Glasswing Announcement**: Anthropic announced a new model, "Claude Mythos Preview," under "Project Glasswing" (named after the transparent glasswing butterfly). The model is not being released to the public; instead, access is granted to a consortium of tech companies (Cisco, Broadcom, Microsoft, Apple, Amazon) strictly for defensive cyberse... [8]
9. A.I. Backlash Turns Violent + Kara Swisher on Healthmaxxing + The Zuck Bot Is Coming - Hard Fork: I'm Dane Bruggler. I cover the NFL draft for the Athletic. Our draft guide picked up the name "The Beast" because of the crazy amount of information that's included. I'm looking at thousands of players putting together hundreds of scouting reports. I've been covering this year's draft since last year's draft. There is a lot in the beast that you simply can't find anywhere else. This is the kind of in-depth, unique jo... [9]
10. A.I. Backlash Turns Violent + Kara Swisher on Healthmaxxing + The Zuck Bot Is Coming - Hard Fork: our standards department and record a separate content warning. >> Yeah. >> So, Cara Swisher, welcome back. We're delighted to have you. >> She just means it wasn't accurate. In fact, it was accurate if we want to relitigate it, I'm happy to. I've moved past it now. I've moved past these things. >> Wonderful. >> Because I'm bigger than ever. I built a new one. >> Let's see, oh, my new one, bitch. That's right, New Yo... [10]
11. A.I. Backlash Turns Violent + Kara Swisher on Healthmaxxing + The Zuck Bot Is Coming - Hard Fork / Takeaways: Blocking local data center construction may feel empowering to citizens, but it is an ineffective lever against the broader march of AI; systemic policy solutions—like flexible social safety nets and retraining programs—are necessary to address labor disruption. AI companies face a credibility crisis: claiming existential risk while simultaneously lobbying against transparency and liability creates a deeply anti-demo... [11]
12. A.I. Backlash Turns Violent + Kara Swisher on Healthmaxxing + The Zuck Bot Is Coming - Hard Fork: it. And I'm about to die. And I go, you've got to be kidding. And then I die. I like it. I think that's good. I'm going to revise my answer. I would now like to die with Cara Swisher hovering over me saying, wow. Oh, wow. All right. Cara always an adventure. I fucked. I fucked with Kevin. You say? Look, he's like, oh, fuck. No, I'm thinking about my mortality now. Thanks a lot. You should, because you will make, by t... [12]

### q094 PASS

- Prompt: What patterns do you notice across transcripts from this topic?
- Class: `cross_video_synthesis`
- Status: `Completed`
- Score: `3`
- Sources: `2`
- Failure: `-`
- Source videos: Claude 3.7 is the best model for devs. | The real reason Claude got dumber

#### Answer

Retrieved evidence for: What patterns do you notice across transcripts from this topic?

The answer model is unavailable, so this fallback lists the highest-ranked saved excerpts instead of synthesizing beyond them.

1. The real reason Claude got dumber - Theo - t3․gg: will be used to reduce remediation time in future similar incidents if they should occur. Also notice that nothing here mentions refunds for affected customers. Over 30% of cloud code users were affected and approximately 0% of them got any money back or anything back as a result. Evals and monitoring are important, but these incidents have shown that we also need continuous signal from users when responses from clau... [1]
2. Claude 3.7 is the best model for devs. - Theo - t3․gg: sks, and puzzle problems, comparing it against competitors like O1, O3 Mini, DeepSeek R1, Gemini, and Grok. The video demonstrates practical applications including refactoring production code with "neverthrow" patterns and using the Claude Code CLI, while honestly discussing pricing concerns and areas where the model underperforms (particularly mathematics). Key Points Model Naming and Release Context Claude 3.7 is n... [2]

### q095 PASS

- Prompt: Which summary seems most reliable?
- Class: `transcript_summary_alignment`
- Status: `Completed`
- Score: `3`
- Sources: `12`
- Failure: `-`
- Source videos: Breaking down my current tech stack | Gemini 3.1 Pro is the smartest model ever made | I ranked every AI based on vibes | Is gpt-5.1 the best code model ever? | Jira and Linear are legacy software | Microsoft and OpenAI are breaking up? | OpenAI: Trapped in 2nd place | This awesome CSS feature is blocked by drama (Google and Apple can't agree) | Vercel Finally Caught Up | Where Should You Deploy In 2026? | Which browser should you use right now?

#### Answer

Retrieved evidence for: Which summary seems most reliable?

The answer model is unavailable, so this fallback lists the highest-ranked saved excerpts instead of synthesizing beyond them.

Summary/transcript alignment evidence: these transcript excerpts and summary passages are the strongest grounded signals for judging what the summary supports, misses, or gets wrong.

1. Where Should You Deploy In 2026? - Theo - t3․gg: TL;DR For most applications (98%+), serverless deployment options are sufficient and recommended as a starting point; move to VPS only if you encounter specific needs. Top recommendations (S-tier): Vercel for serverless, Railway and Render for VPS — all offer excellent developer experience, reliability, and reasonable pricing. Cloudflare offers the lowest costs due to unique infrastructure (no Docker, uses V8 isolate... [1]
2. Vercel Finally Caught Up - Theo - t3․gg / Key Points: marginal [2]
3. Gemini 3.1 Pro is the smartest model ever made - Theo - t3․gg: Bench to measure how well models can name skateboarding tricks based on descriptions, combining niche knowledge with spatial recognition. When first created, Gro 4 had the highest score in the high 70s (75 on most recent run), while GPT5 scored 98 during testing at OpenAI's office. Modern OpenAI models have regressed, with the highest current score around 87. Gemini 3.1 Pro Preview "consistently hits 100%" on this b.... [3]
4. Jira and Linear are legacy software - Theo - t3․gg: workspace, and Linear can intelligently refine, synthesize, or take action on context immediately Automations and Non-Developer Adoption The speaker initially overlooked automations in the Codex app and notes most developers have as well Example automations shown: summarize yesterday's Git activity for standup, synthesize weekly PRs/rollouts/incidents/reviews into updates, draft release notes for merged PRs The spea.... [4]
5. Microsoft and OpenAI are breaking up? - Theo - t3․gg / Key Points: lies. [5]
6. This awesome CSS feature is blocked by drama (Google and Apple can't agree) - Theo - t3․gg / Key Points: item numbering appeared scattered). Keyboard navigation could be problematic. Both proposals address this with a proposed `reading-flow` property to ensure accessible navigation. Poll Results A community poll asking preference between approaches showed approximately 80% favoring Google's `display: masonry` approach. The presenter also concluded preferring Google's approach, particularly after realizing the named area... [6]
7. Breaking down my current tech stack - Theo - t3․gg: Doesn't convex have O? Yeah, and they kind of hate it. Talked to them a lot. They're not proud of their off package. It works kind of. I never got it working. Not sure how much of that was their fault versus mine, but it was way easier for me to use pretty much every other option. We should probably break down those options. The O layer. The main two paths you can take here are a service or package. You can roll it e... [7]
8. Which browser should you use right now? - Theo - t3․gg / Overview: This extensive video is a comprehensive review of the current browser landscape, covering major browsers (Chrome, Edge, Firefox, Safari), privacy-focused alternatives (Brave, Vivaldi, Orion), AI-focused browsers (Dia, Comet), and emerging projects (Zen, Helium, Ladybird). The speaker, a notorious browser-hopper who previously championed Arc, systematically evaluates each browser's strengths, weaknesses, UX decisions,... [8]
9. OpenAI: Trapped in 2nd place - Theo - t3․gg: TL;DR OpenAI consistently releases groundbreaking AI capabilities that briefly put them in first place, but competitors quickly catch up or surpass them, leaving OpenAI in "perpetual second place" across most technical categories. OpenAI's true competitive moat is ChatGPT itself—the default AI chat application for most users—which generates 70% of their revenue and keeps users from switching to technically superior a... [9]
10. I ranked every AI based on vibes - Theo - t3․gg: TL;DR The creator ranks AI models into tiers (S through F) based on practical usability, cost, speed, and quality, using his experience building T3 Chat. **S-tier**: Gemini 2.0 Flash (best overall value/default), Claude 3.5 Sonnet (best for code despite high cost), OpenAI o3 Mini (cheap reasoning model). **A-tier**: o3 Mini (initially placed), Claude 3.7 with reasoning (transparent reasoning), Gemini 2.5 Pro (benchma... [10]
11. Which browser should you use right now? - Theo - t3․gg: TL;DR Chrome/Chromium has had a massively positive impact on web standards and is technically the best implementation, but Google's monopolistic tendencies show in forced AI integrations like Gemini. Manifest V3 was the right call for security (preventing malware), not an anti-ad-blocker move, though ad-blocking is now slightly worse in Chrome. Brave is strongly criticized for buggy UX, breaking websites, aggressive ... [11]
12. Is gpt-5.1 the best code model ever? - Theo - t3․gg / Key Points: model remains the default for planning, but overall disappointment is clear. [12]

### q096 PASS

- Prompt: Which summary seems least reliable?
- Class: `transcript_summary_alignment`
- Status: `Completed`
- Score: `3`
- Sources: `7`
- Failure: `-`
- Source videos: Gemini 3.1 Pro is the smartest model ever made | Is gpt-5.1 the best code model ever? | Jira and Linear are legacy software | This awesome CSS feature is blocked by drama (Google and Apple can't agree) | Vercel Finally Caught Up | What is Theo's Worst Take? | Where Should You Deploy In 2026?

#### Answer

Retrieved evidence for: Which summary seems least reliable?

The answer model is unavailable, so this fallback lists the highest-ranked saved excerpts instead of synthesizing beyond them.

Summary/transcript alignment evidence: these transcript excerpts and summary passages are the strongest grounded signals for judging what the summary supports, misses, or gets wrong.

1. Vercel Finally Caught Up - Theo - t3․gg / Key Points: marginal [1]
2. Where Should You Deploy In 2026? - Theo - t3․gg: d locations, and no serious company uses them. Fly.io has world-class DX and unique features (Elixir-native, Flame), but database reliability issues and financial instability make it risky. AWS (EC2/Lambda) is reliable but expensive, difficult to set up, and has poor DX — only choose if your employer already chose it for you. Digital Ocean has excellent documentation but feels lost strategically and may be "circling.... [2]
3. Gemini 3.1 Pro is the smartest model ever made - Theo - t3․gg: I. The CLI has a "potential loop was detected" hook because models loop and fail so frequently. The presenter describes the CLI as "legitimately unusable." File Handling and Basic Operations Problems The model seems "hardcoded" to only read 100 lines at a time, requiring multiple read operations for longer files (reading lines 1-100, then 101-200, etc.). It frequently fails to edit files it just read, passing "bad sy... [3]
4. What is Theo's Worst Take? - Theo - t3․gg / TL;DR: A speaker is asked to identify their "worst take" The speaker initially claims all of their takes are good, which is suggested might itself be their worst take A past "evil" statement about something called "go" is referenced but not detailed The speaker criticizes a storybook as "useless" Another speaker defends the storybook's value for filling up a "known module" [4]
5. Jira and Linear are legacy software - Theo - t3․gg: rst pass BE the plan Similar pattern happened with MCP: it was supposed to be the best way for models to access data, but "it sucked"—until models could use code to use MCP, then it became much better and more reliable The speaker predicts the same pattern: "we're going to reinvent plans a million fucking times over the next year. And then we're going to just go back to code" Code as Planning The speaker advocates fo... [5]
6. This awesome CSS feature is blocked by drama (Google and Apple can't agree) - Theo - t3․gg / Key Points: item numbering appeared scattered). Keyboard navigation could be problematic. Both proposals address this with a proposed `reading-flow` property to ensure accessible navigation. Poll Results A community poll asking preference between approaches showed approximately 80% favoring Google's `display: masonry` approach. The presenter also concluded preferring Google's approach, particularly after realizing the named area... [6]
7. Is gpt-5.1 the best code model ever? - Theo - t3․gg / Key Points: model remains the default for planning, but overall disappointment is clear. [7]

### q097 PASS

- Prompt: What evidence in the transcript supports the summary?
- Class: `transcript_summary_alignment`
- Status: `Completed`
- Score: `3`
- Sources: `3`
- Failure: `-`
- Source videos: I can't believe this is a real statistic... | It's not just you (Claude did get dumber) | So close to Opus at 1/10th the price (GLM-4.7 and Minimax M2.1 showdown)

#### Answer

Retrieved evidence for: What evidence in the transcript supports the summary?

The answer model is unavailable, so this fallback lists the highest-ranked saved excerpts instead of synthesizing beyond them.

Summary/transcript alignment evidence: these transcript excerpts and summary passages are the strongest grounded signals for judging what the summary supports, misses, or gets wrong.

1. I can't believe this is a real statistic... - Theo - t3․gg: to it so he has to tell me when there's good emails sorry Gabriel I need someone to keep up with this how do you feel are you a ghost engineer or are you working with a whole bunch of them let me know what you think and until next time fire the useless people [1]
2. It's not just you (Claude did get dumber) - Theo - t3․gg: They found two separate issues that are now resolved. They're continuing to monitor for ongoing quality issues. But let's look at the timeline for this. A small percentage, notice I don't say the percentage, but a small percentage of Cloud Sonic 4 requests experienced degraded output quality due to a bug from August 5th to September 4th with the impact increasing from August 29th to September 4th. A fix has been roll... [2]
3. So close to Opus at 1/10th the price (GLM-4.7 and Minimax M2.1 showdown) - Theo - t3․gg: [ __ ] as a result. a lot of wait actually looking at this it also got very confused that we were using TRPC for some things even though almost none of the stuff that this feature touched involved the TRPC endpoints those are mostly for legacy data and account management stuff everything else goes through convex this whole feature should have been convex I even indicated that in the original prompt but it still got v... [3]

### q098 PASS

- Prompt: Does the summary miss anything important from the transcript?
- Class: `transcript_summary_alignment`
- Status: `Completed`
- Score: `3`
- Sources: `6`
- Failure: `-`
- Source videos: Delete your CLAUDE.md (and your AGENT.md too) | I can't believe this is a real statistic... | It's not just you (Claude did get dumber) | So close to Opus at 1/10th the price (GLM-4.7 and Minimax M2.1 showdown) | What’s a Hard Fork?

#### Answer

Retrieved evidence for: Does the summary miss anything important from the transcript?

The answer model is unavailable, so this fallback lists the highest-ranked saved excerpts instead of synthesizing beyond them.

Summary/transcript alignment evidence: these transcript excerpts and summary passages are the strongest grounded signals for judging what the summary supports, misses, or gets wrong.

1. It's not just you (Claude did get dumber) - Theo - t3․gg: They found two separate issues that are now resolved. They're continuing to monitor for ongoing quality issues. But let's look at the timeline for this. A small percentage, notice I don't say the percentage, but a small percentage of Cloud Sonic 4 requests experienced degraded output quality due to a bug from August 5th to September 4th with the impact increasing from August 29th to September 4th. A fix has been roll... [1]
2. Delete your CLAUDE.md (and your AGENT.md too) - Theo - t3․gg: ols (bash calls, grep, file reading); they're good at finding information independently. If information is in the codebase (package.json for commands, file structure for architecture), the model can find it—it doesn't need to be duplicated in a context file. The speaker's experiment: running `/init` on a project called "Lawn" generated a claude.md with architecture, commands, key patterns, etc. The agent read the pac... [2]
3. I can't believe this is a real statistic... - Theo - t3․gg: to it so he has to tell me when there's good emails sorry Gabriel I need someone to keep up with this how do you feel are you a ghost engineer or are you working with a whole bunch of them let me know what you think and until next time fire the useless people [3]
4. What’s a Hard Fork? - Hard Fork / At a glance: The video title asks "What’s a Hard Fork?", but the transcript contains no substantive content on this topic. The provided transcript is solely an automated speech recognition (ASR) system metadata note. The text indicates it originated from a local OpenAI-compatible ASR endpoint, not from official RSS show notes. [4]
5. So close to Opus at 1/10th the price (GLM-4.7 and Minimax M2.1 showdown) - Theo - t3․gg: [ __ ] as a result. a lot of wait actually looking at this it also got very confused that we were using TRPC for some things even though almost none of the stuff that this feature touched involved the TRPC endpoints those are mostly for legacy data and account management stuff everything else goes through convex this whole feature should have been convex I even indicated that in the original prompt but it still got v... [5]
6. What’s a Hard Fork? - Hard Fork / Key Points: Transcript Metadata**: The only content in the transcript is a procedural note indicating it is a "smoke transcript" generated by a local OpenAI-compatible ASR endpoint, explicitly stating it did not come from RSS show notes. No definitions, examples, or explanations of a "hard fork" are present. [6]

### q099 PASS

- Prompt: Can you answer this with citations from the source videos?
- Class: `direct_lookup`
- Status: `Completed`
- Score: `3`
- Sources: `10`
- Failure: `-`
- Source videos: A free model just appeared in Cursor (and it’s really good at code) | ChatGPT “Pro” Has Some Real Safety Concerns... | Gemini Flash 3 is my new favorite model (yes really) | I need you guys to trust me on this (sorry Anthropic) | Microsoft and OpenAI are breaking up? | OpenAI’s new API is 200x more expensive than competition | What happened to me? | “Just Use HTML”

#### Answer

Retrieved evidence for: Can you answer this with citations from the source videos?

The answer model is unavailable, so this fallback lists the highest-ranked saved excerpts instead of synthesizing beyond them.

1. OpenAI’s new API is 200x more expensive than competition - Theo - t3․gg: overall like time to answer is faster on 03 mini. a really really good model. But when you use 03 mini high, the cost isn't necessarily represented just by this number because when you use it on high, it's generating way more tokens because that's what the the low medium high is. It's how much time can it spend and how much how many tokens can it generate in the step before it starts answering. So it's almost like th... [1]
2. Microsoft and OpenAI are breaking up? - Theo - t3․gg / Key Points: lies. [2]
3. ChatGPT “Pro” Has Some Real Safety Concerns... - Theo - t3․gg: a correct and incorrect answer answer got about a third of the way through got bored and haven't went back since soon DM to highlight the main strength of the o1 pro mode which is improved reliability we use a stricter evaluation setting a model is only considered to solve a question if it gets the answer right in four out of four attempts so it needs 4X reliability instead of just once here if it got the answer righ... [3]
4. What happened to me? - Theo - t3․gg: he says in his videos. If Lionus is in a video, there's a 90 plus% chance it's a script someone else wrote and a topic somebody else came up with in a video that he is being pulled in to act out. I am not an actor. Even my sponsors can't give me things to say. When one of my sponsors has a specific thing they want me to say in a video, I usually just tell them outright no. Or I help them turn it into not a quote, but... [4]
5. OpenAI’s new API is 200x more expensive than competition - Theo - t3․gg: We did. We got an answer. It finished. It just took forever. And if you didn't notice, let's compare just the length of this answer to the answer that we got from chat GPT. Where's the tab? This is the 03 mini high. This is the equivalent. It's still generating. But if we go to the 01 Pro also, what the hell happened there? I switched to this one. It changed to 03 mini for a sec, then to pro. How does anyone say this... [5]
6. “Just Use HTML” - Theo - t3․gg: [ __ ] websites, but I know [ __ ] better than to pat out this video any [ __ ] more. So, I'm just going to be [ __ ] done. Let me know what you think. Until next time, [ __ ] [6]
7. Gemini Flash 3 is my new favorite model (yes really) - Theo - t3․gg: since this model is smaller, Anthropic trained it to say no more often. And if it says no, I don't know the answer, it will score better here. If it makes up an answer, it scores much worse here. And this is where things get scary. Gemini 3 flash. 91% of the time it doesn't know. It will lie and make up an answer. And this is when you have to be really honest with yourself depending on what your use case is. Imagine.... [7]
8. Gemini Flash 3 is my new favorite model (yes really) - Theo - t3․gg: open weight models because different providers can host them and those different providers vary a lot in how well they host it. Even Google Vertex when hosting Kimmy K2 thinking is pulling almost 200 TPS. So if you want a model that's actually nice to talk to and you're using Google Cloud, don't touch Flash, don't touch Pro, go throw Kimmy K2 on Vertex and you'll get crazy speeds, really good prices, and a much nicer... [8]
9. I need you guys to trust me on this (sorry Anthropic) - Theo - t3․gg: cloud or does it have to be on my own machine? All these questions and more have not been answered well. And while I do mostly have confidence in my answer for a few of those things, it's nearly impossible to know. And I am far from the only person who feels this way. I have so many incredible people who have been DMing me like hi at people that you would never guess hitting me up asking if I have any insights on the... [9]
10. A free model just appeared in Cursor (and it’s really good at code) - Theo - t3․gg: and Inc., but got a decent looking CLI that gives us actual useful information. Not sure if it's reliably running the bench because those numbers are a bit low or if output equals an answer or ignoring internal spaces. Yeah, it's a little too strict with how that encoded that. But yeah, so I guess when you're trying to make a crappy benchmark really fast, Sonic is marginally better at implementation details and also.... [10]

### q100 PASS

- Prompt: Based on my library, what should I learn next?
- Class: `meta_learning_or_next_step`
- Status: `Completed`
- Score: `3`
- Sources: `12`
- Failure: `-`
- Source videos: I don’t really use libraries anymore | Is it time to move on? | React Native Just Got 550% Faster | Svelte 5 Is Like React, But Better | The Biggest React Framework You've Never Heard of | This might change how we build UI forever | Why Everyone Hates Web Components | Why is everyone so unhappy with JavaScript? | Zod finally has competition (...created by Zod?)

#### Answer

Retrieved evidence for: Based on my library, what should I learn next?

The answer model is unavailable, so this fallback lists the highest-ranked saved excerpts instead of synthesizing beyond them.

1. I don’t really use libraries anymore - Theo - t3․gg / Takeaways: Re-evaluate your dependencies**: Go through your package.json and assess whether each library is necessary, whether it's holding you back, and whether AI could help you implement an alternative. **Consider the new calculus**: The math has changed—problem difficulty is lower with AI, perceived risk of dependencies is higher, so the threshold for adopting libraries should be higher. **Think about control**: When you ad... [1]
2. Why Everyone Hates Web Components - Theo - t3․gg: and make changes to it something else I'd say is similar would be like a material UI and other UI libraries these two are pretty close to the base HTML element thing what we have learned as an ecosystem is if you're not this being close to it kind of sucks and that's why we've moved the other way where we have stuff like Shaden and Shaden looks and operates similarly to material UI where you have all the components y... [2]
3. Zod finally has competition (...created by Zod?) - Theo - t3․gg: ion standards. Colin (Zod's creator) worked with creators of Valibot and Arktype to develop the "Standard Schema" spec—a common interface for multiple validation libraries. Standard Schema allows framework and library authors to support multiple validators (Zod, Valibot, Arktype) without writing separate adapters for each. The spec is designed for library/framework authors, not end users; it enables ecosystem-wide in... [3]
4. I don’t really use libraries anymore - Theo - t3․gg / Key Points: for understanding different library types: **Libraries beyond your knowledge**: These are used by people who don't know how to solve the problem themselves. Examples include `is-odd` (literally one line of code) and `leftpad`. The argument against these is that users are outsourcing competency and taking on supply chain risks without understanding them. **Libraries for tedious reimplementation**: Even capable develop... [4]
5. React Native Just Got 550% Faster - Theo - t3․gg: supported: Suspense, Transitions, Automatic Batching, and `useLayoutEffect` finally work correctly in React Native. The new architecture enables concurrent rendering across multiple threads, allowing priority-based updates that can interrupt ongoing renders. Native modules are now written in C++ (Turbo Modules), enabling type-safe, cross-platform code sharing with lazy loading for faster startup. New React Native De.... [5]
6. Is it time to move on? - Theo - t3․gg: I recommend too many new things because I don't and if your goal is to get a job learn old things learn something that's react or older if you really want to get a job ASAP go learn Cobalt or some [ __ ] something ancient job opportunities and productivity opportunities tend to come from using oldish Solutions there's a reason PHP has this huge wave right now of Indie hackers because old things are great they're stab... [6]
7. I don’t really use libraries anymore - Theo - t3․gg / Overview: This video explores how AI-assisted development is fundamentally changing the role and utility of software libraries. The speaker, a developer who has built many projects using various libraries, shares his evolving perspective on dependency management in an era where AI can generate implementations. He discusses his personal experience removing libraries like Tkumi from projects, examines industry examples like Anti... [7]
8. This might change how we build UI forever - Theo - t3․gg: n, dependency management, and a registry schema that opens possibilities for distributing any type of code beyond just UI components. Key Points Shad CN Philosophy and Architecture Shad CN is not a traditional library you install; it provides components that are added directly to your project, meaning you own and can modify all the code. It uses Radix UI (a headless UI library) for behaviors like dialog, dropdown men... [8]
9. I don’t really use libraries anymore - Theo - t3․gg: are still around in 2026. Because as frustrating and annoying as they are to work with, there are often dependencies that companies are building on and around that expect tools like Webpack to work a certain way so that you could integrate them into your codebase a certain way. A tool like this fast float dep is expecting g++ to be installed. Sadly, a lot of Linux distributions don't have that installed even when you... [9]
10. Why is everyone so unhappy with JavaScript? - Theo - t3․gg: arrator actively uses Sets for unique key collections with union/intersection operations but observes they're rarely used in others' code. **Object.groupBy**: New feature allowing creation of objects with keys based on a grouping function—could be useful for coding tasks. **Browser APIs**: WebSockets widely used; PWA usage at 49% (narrator suspects this is declining); geolocation API at 35%—twice as high as WebGL whi... [10]
11. Svelte 5 Is Like React, But Better - Theo - t3․gg: s history. The Svelte team maintains both Svelte and SvelteKit; SvelteKit is presented as one of the best meta-frameworks currently available, praised for its clean data loading system, form actions, and quick learning curve. The website documentation was overhauled from a split structure (svelte.dev, kit.svelte.dev, learn.svelte.dev) into a unified, redesigned experience. Why Svelte Changed: Addressing Limitations i... [11]
12. The Biggest React Framework You've Never Heard of - Theo - t3․gg: b stars, offering a comprehensive alternative to Next.js and Remix with unique features like built-in micro-frontends support and JSX+ extensions. The framework uses React Router under the hood and offers file-based routing similar to Next.js, but with distinct capabilities like SSG by default (opting into SSR requires configuration), a unique data loading pattern, and React Query-like request handling. JSX+ introduc... [12]

