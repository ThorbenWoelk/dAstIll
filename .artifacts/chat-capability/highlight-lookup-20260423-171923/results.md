# Chat Capability Sweep Results

- Generated: `2026-04-23T15:19:54.484587+00:00`
- Base URL: `http://localhost:3544`
- Dataset: `/Users/thorben.woelk/repos/dAstIll/backend/tests/data/chat_capability_prompts.json`
- Prompt count: `6`

## Summary

- Passed prompts: `0/6`
- Answerability pass: `6/6`
- Grounding pass: `0/6`
- Shape pass: `6/6`
- Average score: `2.00`

## Capability Classes

- `highlight_lookup`: passed `0/6`, avg score `2.00`, failures `no_sources`

## Failures By Class

- `no_sources`: q062, q063, q064, q065, q070, q071

## Prompt Results

### q062 FAIL

- Prompt: What highlights have I saved from this video?
- Class: `highlight_lookup`
- Status: `Completed`
- Score: `2`
- Sources: `12`
- Failure: `no_sources`
- Source videos: 20 years later, PNG 3.0 is finally here | Defending my product from the dumbest possible haters | Did Anthropic just kill Figma? | Git is holding us back | Making Minecraft 100x faster (by rewriting it in Rust) | Microsoft’s new vim alternative is written in Rust??? | Open source is dying | Saving the web from Javascript bloat | TOON: “JSON for AI” (is it any good?)
- Notes: highlight prompt did not use the saved highlights tool

#### Answer

Retrieved evidence for: +{Open source is dead now?} What highlights have I saved from this video?

The answer model is unavailable, so this fallback lists the highest-ranked saved excerpts instead of synthesizing beyond them.

1. Microsoft’s new vim alternative is written in Rust??? - Theo - t3․gg: Cool. chmod /edit. Aha, look at that. Even a dumbass like me can compile this. You can create new files, open them, open and close the editor. You have all the hot keys you need. Control S for save, not command S. Don't don't be a Mac person. This is a Windows product, remember. So, you got to reach over slightly if you're a Mac user. Hit that. Control S, control O for open. Option E for edit in order to get into thi... [1]
2. Open source is dying - Theo - t3․gg / Overview: The video presents a comprehensive examination of how AI is negatively impacting the open source ecosystem. The speaker, an experienced open source maintainer and creator of T3 Code, shares firsthand experiences managing a new project that received 150 PRs in just 5 days. The discussion covers four main problem areas: PR spam and quality degradation, increasingly hostile and confused users, GitHub's inadequate platfo... [2]
3. Open source is dying - Theo - t3․gg: escalate because you almost certainly have to use AI to actually scan these PRs. It has a lot of config, enough config that I'm almost certain that this project was vibe coded, but yeah, you get the idea. There are a lot of solutions being made to try and fix these problems. Some of them are going to make it harder for new maintainers to break out, which sucks because we might just have our current maintainers until ... [3]
4. Saving the web from Javascript bloat - Theo - t3․gg: has lots of other subdeps. We just can't see them. Apparently Pierre diffs library has a lot of subdeps. I did not realize that like half our dependency graph comes from the diffs library. Interesting. To be fair, there are some core depths in here that have to be a little complex like shiki for the syntax highlighting. Like that's just not a trivial thing. And then the has to util HTML that turns the syntax tree int... [4]
5. Open source is dying - Theo - t3․gg: put so much effort into killing Hacktoberfest. I think this video of mine, don't contribute to open source, is one of the best videos I ever filmed. Not sure who that blonde guy with the mustache is though. Seriously though, like that video, I have been told by so many maintainers how thankful they are for this video more than almost anything I've done as a developer and journalist, YouTuber, whatever you want to cal... [5]
6. Did Anthropic just kill Figma? - Theo - t3․gg: Code create this design. You can save it as a template as well. Cool. So I know what we have to do. The T3 code site is a real website that exists, but it's far from great. So, let's fix that. Let's do our first prototype. We'll do a wireframe create. We'll attach a codebase. Let's pull out whisper for this one. They have a speech button here, too, but uh I don't trust that. I would like to redesign this page. The ma... [6]
7. Defending my product from the dumbest possible haters - Theo - t3․gg: you don't need security before you have a 100 or more paying users except for some special businesses devs once again thinking that they save time by doing a useless thing faster I'll be honest I expected some amount of push back with what we built with upload thing I'm not going to sit here and pretend that we did everything perfectly but I certainly did not expect the push back to be as straight up dumb as a lot of... [7]
8. Open source is dying - Theo - t3․gg: feel awesome. Those messages make my goddamn day. Seeing somebody hit me up about how they were a line cook for a decade, learning code on the side, didn't feel like they could really do it, but watching my videos made them feel more like this crazy tech world we were in was a place they could fit, and now they have awesome tech jobs. My video isn't what did it. My channel isn't what did it. They did it. But that mes... [8]
9. 20 years later, PNG 3.0 is finally here - Theo - t3․gg: iPhone or Mac, I would highly recommend you go check out the most recent Digital Foundry video, the Death Stranding one. We don't encode our videos in HDR, so this will not look particularly good at all coming from me. But if you have a high-end monitor or a system that actually knows what HDR is, the intro for this video that Digital Foundry just put out is within the best HDR demos I've ever seen in my life. A comb... [9]
10. TOON: “JSON for AI” (is it any good?) - Theo - t3․gg: Tokenization Playground The creator tests Tune's tokenization playground with various data structures. Initial simple comparison showed: flattened JSON at 38 tokens, YAML at 50, and Tune at 32. The compression from flattened JSON to Tune was noted as "only like four tokens" in one test—"barely notable." When testing with non-standard, non-uniform structures, results were surprising: JSON at 371 tokens, flattened JSON... [10]
11. Making Minecraft 100x faster (by rewriting it in Rust) - Theo - t3․gg: discussion covers the technical challenges of reverse-engineering Minecraft's proprietary server protocol, the trade-offs involved in abandoning plugin compatibility, and the impressive community projects that have pushed Redstone computing to its limits through specialized server software. Key Points The Problem with Current Minecraft Hosting The host notes that despite years of optimization efforts, Java Minecraft.... [11]
12. Git is holding us back - Theo - t3․gg: between commits is where Git breaks down: you can't commit during merge conflicts, there's no native support for stacked PRs, and tracking work-in-progress requires hacky workflows. Zed IDE raised $32 million from Sequoia and announced Delta DB, an operation-based version control system using CRDTs that tracks every edit—not just commits—and is designed to interoperate with Git while enabling real-time collaboration.... [12]

### q063 FAIL

- Prompt: Show me all highlights related to search.
- Class: `highlight_lookup`
- Status: `Completed`
- Score: `2`
- Sources: `11`
- Failure: `no_sources`
- Source videos: Better than Claude and 10x cheaper?? GLM-4.6 is so good | Cursor just started their war on GitHub | Git is holding us back | Is this the end of Chrome? | Tailwind V4 is WAY better than I expected | Why is every React site so slow? | i made my own search engine (kind of) | o3-mini is really good (but does it beat deepseek?)
- Notes: highlight prompt did not use the saved highlights tool

#### Answer

Retrieved evidence for: Show me all highlights related to search.

The answer model is unavailable, so this fallback lists the highest-ranked saved excerpts instead of synthesizing beyond them.

1. Cursor just started their war on GitHub - Theo - t3․gg: their editor which feels like it's slowly falling to pieces. This is why I went to their office with my team on Monday to just sit with them and complain for like hours straight about all of the bugs that we've been experiencing. and they've now committed to not focusing on new features for a bit and just going hardcore in bug fixing mode. Every day they've been giving my team status updates on all the bugs they fixe... [1]
2. i made my own search engine (kind of) - Theo - t3․gg / Key Points: contributor pointed out that `!ghr` (GitHub repo hotlink) parsed slashes incorrectly; the creator added custom handling so queries like `!ghr ping.gg/upload` correctly open the specific repository. **Potential Future Features**: Ideas mentioned but not implemented include logging searches locally in IndexedDB to view history, and allowing custom bangs via local storage. [2]
3. i made my own search engine (kind of) - Theo - t3․gg: a chat on T3 chat I have been using this so much more than I ever ever would have guessed sadly I didn't rig the response as you guys can tell here there's a handful of people I would not have recommended in the list like Tech lead Bob Martin but the fact that I could search things like that to AI directly has actually been really really nice I found myself defaulting to searching my own AI chat app instead of doing ... [3]
4. Better than Claude and 10x cheaper?? GLM-4.6 is so good - Theo - t3․gg / Key Points: ing approximately one-tenth the price for output tokens. GLM 4.6 also performs better than GLM 4.5 in comparisons. Token efficiency is highlighted: GLM 4.6 uses fewer tokens per request than Deepseek or Kimmy, showing Labs focusing on reducing token overusage. Pricing and Economics GLM 4.6 is positioned as dramatically cheaper than frontier models - described as "10x cheaper" in the video title. A GLM coding plan cos... [4]
5. Is this the end of Chrome? - Theo - t3․gg / Key Points: Anthropic. The creator notes keyword targeting is valuable—Anthropic appears to do keyword targeting on Google, with Claude ads appearing on AI-related searches. [5]
6. Why is every React site so slow? - Theo - t3․gg: ormance issues on many major sites (GitHub, Pinterest, DoorDash, Twitch) due to components re-rendering when they don't need to. React by default re-checks everything below a state change; it does not automatically skip components whose props haven't changed—you must manually add memoization or use React Compiler. Manual memoization (`React.memo`, `useCallback`, `useMemo`) is error-prone; a single inline object or fu... [6]
7. Tailwind V4 is WAY better than I expected - Theo - t3․gg: TL;DR Tailwind V4 is described as the biggest overhaul to date, featuring a complete engine rewrite in Rust for dramatically faster builds (up to 5x faster cold starts, 100x faster incremental builds when no new CSS is needed). The framework moves to a CSS-first configuration system, eliminating the traditional `tailwind.config.js` file and allowing themes, plugins, and settings to be defined directly in CSS files. T... [7]
8. i made my own search engine (kind of) - Theo - t3․gg: to do quick searches like that or here what happened to the chat message there might have my setup in a slightly corrupted sayate CU I'm debugging on my own account all the time my my chat history is a [ __ ] mess for all the weird things I do but the point I'm trying to make here is I've been really surprised at how often I'm using my search engine to search my chat app and it's kind of rewired my brain I'm sure som... [8]
9. o3-mini is really good (but does it beat deepseek?) - Theo - t3․gg / Key Points: o1. "Medium" reasoning effort beats o1. "High" reasoning effort "smokes" o1. This comes at 9x cheaper and significantly faster speeds. **Lex Fridman's perspective**: Lex tweeted that o3-mini is good, but DeepSeek R1 offers similar performance, is still cheaper, and reveals its reasoning. The creator agrees this is the "biggest thing"—DeepSeek doesn't hide reasoning while OpenAI does. Lex predicts the DeepSeek moment ... [9]
10. Better than Claude and 10x cheaper?? GLM-4.6 is so good - Theo - t3․gg: from ZI (Zhipu AI) is a new openweight model that achieves performance competitive with Claude Sonnet 4 at roughly 1/10th the cost, with a 48.6% win rate against Sonnet 4 in head-to-head comparisons. The model shows significant improvements over GLM 4.5, including longer context windows up to 200k tokens, superior coding performance, and better reasoning capabilities with tool use during inference. ZI distinguishes i... [10]
11. Git is holding us back - Theo - t3․gg: TL;DR Git has been essential to modern software development and open source for nearly 20 years, but its design—originally built for Linus Torvalds' specific email-based patch workflow—no longer fits how developers work today, especially with AI agents contributing to codebases. The gap between commits is where Git breaks down: you can't commit during merge conflicts, there's no native support for stacked PRs, and tr... [11]

### q064 FAIL

- Prompt: Show me all highlights related to summaries.
- Class: `highlight_lookup`
- Status: `Completed`
- Score: `2`
- Sources: `8`
- Failure: `no_sources`
- Source videos: Better than Claude and 10x cheaper?? GLM-4.6 is so good | Cursor just started their war on GitHub | Git is holding us back | React feels insane | Tailwind V4 is WAY better than I expected | Vercel Finally Caught Up | Why is every React site so slow?
- Notes: highlight prompt did not use the saved highlights tool

#### Answer

Retrieved evidence for: Show me all highlights related to summaries.

The answer model is unavailable, so this fallback lists the highest-ranked saved excerpts instead of synthesizing beyond them.

1. Vercel Finally Caught Up - Theo - t3․gg / Key Points: marginal [1]
2. Cursor just started their war on GitHub - Theo - t3․gg: their editor which feels like it's slowly falling to pieces. This is why I went to their office with my team on Monday to just sit with them and complain for like hours straight about all of the bugs that we've been experiencing. and they've now committed to not focusing on new features for a bit and just going hardcore in bug fixing mode. Every day they've been giving my team status updates on all the bugs they fixe... [2]
3. React feels insane - Theo - t3․gg / Key Points: understanding a tool doesn't mean it's bad. [3]
4. Better than Claude and 10x cheaper?? GLM-4.6 is so good - Theo - t3․gg / Key Points: ing approximately one-tenth the price for output tokens. GLM 4.6 also performs better than GLM 4.5 in comparisons. Token efficiency is highlighted: GLM 4.6 uses fewer tokens per request than Deepseek or Kimmy, showing Labs focusing on reducing token overusage. Pricing and Economics GLM 4.6 is positioned as dramatically cheaper than frontier models - described as "10x cheaper" in the video title. A GLM coding plan cos... [4]
5. Why is every React site so slow? - Theo - t3․gg: ormance issues on many major sites (GitHub, Pinterest, DoorDash, Twitch) due to components re-rendering when they don't need to. React by default re-checks everything below a state change; it does not automatically skip components whose props haven't changed—you must manually add memoization or use React Compiler. Manual memoization (`React.memo`, `useCallback`, `useMemo`) is error-prone; a single inline object or fu... [5]
6. Tailwind V4 is WAY better than I expected - Theo - t3․gg: TL;DR Tailwind V4 is described as the biggest overhaul to date, featuring a complete engine rewrite in Rust for dramatically faster builds (up to 5x faster cold starts, 100x faster incremental builds when no new CSS is needed). The framework moves to a CSS-first configuration system, eliminating the traditional `tailwind.config.js` file and allowing themes, plugins, and settings to be defined directly in CSS files. T... [6]
7. Better than Claude and 10x cheaper?? GLM-4.6 is so good - Theo - t3․gg: from ZI (Zhipu AI) is a new openweight model that achieves performance competitive with Claude Sonnet 4 at roughly 1/10th the cost, with a 48.6% win rate against Sonnet 4 in head-to-head comparisons. The model shows significant improvements over GLM 4.5, including longer context windows up to 200k tokens, superior coding performance, and better reasoning capabilities with tool use during inference. ZI distinguishes i... [7]
8. Git is holding us back - Theo - t3․gg: TL;DR Git has been essential to modern software development and open source for nearly 20 years, but its design—originally built for Linus Torvalds' specific email-based patch workflow—no longer fits how developers work today, especially with AI agents contributing to codebases. The gap between commits is where Git breaks down: you can't commit during merge conflicts, there's no native support for stacked PRs, and tr... [8]

### q065 FAIL

- Prompt: Show me all highlights related to evaluation.
- Class: `highlight_lookup`
- Status: `Completed`
- Score: `2`
- Sources: `8`
- Failure: `no_sources`
- Source videos: Better than Claude and 10x cheaper?? GLM-4.6 is so good | Cursor just started their war on GitHub | Git is holding us back | React feels insane | Tailwind V4 is WAY better than I expected | Vercel Finally Caught Up | Why is every React site so slow?
- Notes: highlight prompt did not use the saved highlights tool

#### Answer

Retrieved evidence for: Show me all highlights related to evaluation.

The answer model is unavailable, so this fallback lists the highest-ranked saved excerpts instead of synthesizing beyond them.

1. Cursor just started their war on GitHub - Theo - t3․gg: their editor which feels like it's slowly falling to pieces. This is why I went to their office with my team on Monday to just sit with them and complain for like hours straight about all of the bugs that we've been experiencing. and they've now committed to not focusing on new features for a bit and just going hardcore in bug fixing mode. Every day they've been giving my team status updates on all the bugs they fixe... [1]
2. Vercel Finally Caught Up - Theo - t3․gg / Key Points: marginal [2]
3. Better than Claude and 10x cheaper?? GLM-4.6 is so good - Theo - t3․gg / Key Points: ing approximately one-tenth the price for output tokens. GLM 4.6 also performs better than GLM 4.5 in comparisons. Token efficiency is highlighted: GLM 4.6 uses fewer tokens per request than Deepseek or Kimmy, showing Labs focusing on reducing token overusage. Pricing and Economics GLM 4.6 is positioned as dramatically cheaper than frontier models - described as "10x cheaper" in the video title. A GLM coding plan cos... [3]
4. React feels insane - Theo - t3․gg / Key Points: understanding a tool doesn't mean it's bad. [4]
5. Why is every React site so slow? - Theo - t3․gg: ormance issues on many major sites (GitHub, Pinterest, DoorDash, Twitch) due to components re-rendering when they don't need to. React by default re-checks everything below a state change; it does not automatically skip components whose props haven't changed—you must manually add memoization or use React Compiler. Manual memoization (`React.memo`, `useCallback`, `useMemo`) is error-prone; a single inline object or fu... [5]
6. Tailwind V4 is WAY better than I expected - Theo - t3․gg: TL;DR Tailwind V4 is described as the biggest overhaul to date, featuring a complete engine rewrite in Rust for dramatically faster builds (up to 5x faster cold starts, 100x faster incremental builds when no new CSS is needed). The framework moves to a CSS-first configuration system, eliminating the traditional `tailwind.config.js` file and allowing themes, plugins, and settings to be defined directly in CSS files. T... [6]
7. Better than Claude and 10x cheaper?? GLM-4.6 is so good - Theo - t3․gg: from ZI (Zhipu AI) is a new openweight model that achieves performance competitive with Claude Sonnet 4 at roughly 1/10th the cost, with a 48.6% win rate against Sonnet 4 in head-to-head comparisons. The model shows significant improvements over GLM 4.5, including longer context windows up to 200k tokens, superior coding performance, and better reasoning capabilities with tool use during inference. ZI distinguishes i... [7]
8. Git is holding us back - Theo - t3․gg: TL;DR Git has been essential to modern software development and open source for nearly 20 years, but its design—originally built for Linus Torvalds' specific email-based patch workflow—no longer fits how developers work today, especially with AI agents contributing to codebases. The gap between commits is where Git breaks down: you can't commit during merge conflicts, there's no native support for stacked PRs, and tr... [8]

### q070 FAIL

- Prompt: Find highlights that support a specific claim.
- Class: `highlight_lookup`
- Status: `Completed`
- Score: `2`
- Sources: `12`
- Failure: `no_sources`
- Source videos: AI isn't gonna keep improving | Anthropic’s Cybersecurity Shock Wave + Ronan Farrow and Andrew Marantz on Their Sam Altman Investigation + One Good Thing | Astro stole my favorite parts of Next (and I'm hyped) | Does Shopify Regret React Native? | GPT-4.1 is here, and it was built for developers | Microsoft’s new vim alternative is written in Rust??? | React feels insane | Tailwind V4 is WAY better than I expected | The talk that changed the web
- Notes: highlight prompt did not use the saved highlights tool

#### Answer

Retrieved evidence for: Find highlights that support a specific claim.

The answer model is unavailable, so this fallback lists the highest-ranked saved excerpts instead of synthesizing beyond them.

1. Microsoft’s new vim alternative is written in Rust??? - Theo - t3․gg / Key Points: Microsoft and because the developer grew frustrated with C's lack of features after enjoying them in Zig. The developer's criticisms of Rust included weak allocator support, difficulty building trees, and the lack of cursors for linked lists in stable Rust. Features and Functionality The editor supports mouse mode, allowing users to click and select text directly in the terminal interface. Standard hotkeys include Co... [1]
2. React feels insane - Theo - t3․gg / Key Points: understanding a tool doesn't mean it's bad. [2]
3. The talk that changed the web - Theo - t3․gg: has kind of battle tested philosophies, right? Like, did you find the meme that fast? God damn dumb. Yeah, this classic in JSX. It is JavaScript minus the HTML part. Condition content other condition other content fallback. I even prefer to make this a function and if condition return content. If other condition return other content bottom out return fallback spelt it own thing view yeah view we understand how functi... [3]
4. Tailwind V4 is WAY better than I expected - Theo - t3․gg / Key Points: d this live and found it works for integers and for decimals ending in `.5` (e.g., `h-12.5` works), but not other decimals like `12.4` or `12.8`. **Concerns About Arbitrary Values**: The host expresses concern that this flexibility removes a discipline benefit of Tailwind—keeping sizes consistent via a scale. They worry it could lead to "cursed" CSS with inconsistent values across a codebase, and slightly undermine t... [4]
5. Anthropic’s Cybersecurity Shock Wave + Ronan Farrow and Andrew Marantz on Their Sam Altman Investigation + One Good Thing - Hard Fork / At a glance: announced "Claude Mythos Preview," a highly capable new AI model withheld from the public due to severe cybersecurity risks, instead providing access to a defensive tech consortium. The model can autonomously find zero-day exploits in critical open-source infrastructure (e.g., OpenBSD, FFmpeg) that have evaded human researchers and automated tools for decades. The hosts argue this is not a marketing stunt, as releas.... [5]
6. Microsoft’s new vim alternative is written in Rust??? - Theo - t3․gg: lt CLI editor in Windows 11 after previewing in the Windows Insider program. A lead developer prototyped the editor in four languages (C, C++, Zig, and Rust), ultimately choosing Rust due to internal Microsoft support, despite preferring Zig. The project was created to fill the gap in 64-bit Windows, which lacks a built-in CLI editor, and to avoid the complexity of modal editors like Vim for new users. The codebase i... [6]
7. Tailwind V4 is WAY better than I expected - Theo - t3․gg / Key Points: ative Cascade Layers**: V4 is built on native CSS cascade layers (`@layer`), improving how styles override each other. **Wide-Gamut Colors (oklch)**: The host enthusiastically notes the move to oklch for color support, calling RGB "dead" for modern HDR workflows. This enables wider color spectrum control. **Container Queries, `@starting-style`, Popovers**: These and other modern CSS features now have first-class supp... [7]
8. AI isn't gonna keep improving - Theo - t3․gg / Key Points: how one wouldn't work on iterating CPUs to find massive performance wins. The speaker argues that just as Apple moved past raw CPU speed by inventing efficiency cores, performance cores, and specialized encoders, AI needs to move beyond raw LLM scaling. The "Bitter Lesson" and Its Potential Reversal The video discusses Rich Sutton's famous 2019 blog post "The Bitter Lesson," which argues that AI methods using massive... [8]
9. GPT-4.1 is here, and it was built for developers - Theo - t3․gg: amples from real companies. By the way, 4.1 was 53% more accurate than 40 on in their internal benchmark at Blue J, which is tax AI. Cool. Job and accuracy key to both system performance and user satisfaction. highlights 4.1's improved comprehension of complex regulations and it ability to follow nuance instructions over long contexts. Imagine if we spent all this time fixing America's tax system instead of building.... [9]
10. Does Shopify Regret React Native? - Theo - t3․gg / Key Points: p downloads and syncs data to process offline transactions) JavaScript cannot run in background when switching between apps The philosophy is "native AND React Native, not native OR React Native." Training and Support Structures Shopify invested in training: Self-served courses covering production-ready React Native development Office hours with proficient React Native developers for Q&A, pair programming, and code r... [10]
11. Astro stole my favorite parts of Next (and I'm hyped) - Theo - t3․gg: t better compatibility with things like Cloud flare and other Edge run times as well as running those locally awesome I'm into it o responsive images and SVG let's see what they have here for us image cropping support that is interesting I haven't seen any framework do this where when you set a fit with a width and height it crops out the things that don't fit so you don't have to send the whole image just to render.... [11]
12. Tailwind V4 is WAY better than I expected - Theo - t3․gg: d a test project by detecting breaking changes and consolidating backwards-compatibility overrides in a single CSS layer. Arbitrary values now work without brackets for numeric inputs (e.g., `h-54`), gradients support angles, and new utility variants like `@min`, `@max`, `group-has`, `not`, and descendant selectors have been added. Overview This video provides an extensive, hands-on review of the newly released Tailw... [12]

### q071 FAIL

- Prompt: Find highlights that contradict a specific claim.
- Class: `highlight_lookup`
- Status: `Completed`
- Score: `2`
- Sources: `6`
- Failure: `no_sources`
- Source videos: A PHP framework just raised a bunch of money | Anthropic is lying to us. | Defending a disaster (modern frontend development rant) | Is Sam Altman evil? The OpenAI Files are wild | React feels insane | We need to talk about Sonnet 4.6
- Notes: highlight prompt did not use the saved highlights tool

#### Answer

Retrieved evidence for: Find highlights that contradict a specific claim.

The answer model is unavailable, so this fallback lists the highest-ranked saved excerpts instead of synthesizing beyond them.

1. A PHP framework just raised a bunch of money - Theo - t3․gg: e, though the funding aligns with Taylor Otwell's larger ambitions. Laravel Cloud mirrors Vercel's model: an open-source framework creating a paid deployment/infrastructure platform, with announcement language that is highly similar to Vercel's marketing. The speaker critiques the PHP community's historical disdain for VC-funded open source projects, highlighting the contradiction now that Laravel has accepted the sa... [1]
2. React feels insane - Theo - t3․gg / Key Points: understanding a tool doesn't mean it's bad. [2]
3. Anthropic is lying to us. - Theo - t3․gg: equest dramatically, and Miniax had a legitimate product using Claude. The speaker challenges Anthropic's safety claims, questions their motives for potentially weaponizing against open-source competition, and highlights Anthropic's pattern of making similar accusations against competitors. Core irony noted: Anthropic trained on scraped internet data but now cries foul when others potentially do similar extraction fr... [3]
4. Defending a disaster (modern frontend development rant) - Theo - t3․gg: TL;DR Theo reacts to Frank Taylor's article criticizing modern frontend development, finding agreement on some principles (content-first thinking, avoiding unnecessary complexity, not chasing shiny new tech) but strongly disagreeing with many technical criticisms. Key disagreements include: the author's defense of CSS global scoping (Theo argues naming conflicts are real problems in large codebases), criticism of Rea... [4]
5. We need to talk about Sonnet 4.6 - Theo - t3․gg: /month Claude Code subscription offers subsidized inference worth up to $2,700, creating an impossible competitive landscape for third-party developers who must pay full API rates Anthropic's Agent SDK policy contradictions—first allowing subscription use, then prohibiting it—have left developers in legal and business limbo without clear answers OpenAI contrasts sharply with Anthropic by embracing developers, allowi.... [5]
6. Is Sam Altman evil? The OpenAI Files are wild - Theo - t3․gg / Overview: The video provides an extensive point-by-point rebuttal of the "OpenAI Files," a document containing various accusations against Sam Altman and OpenAI. The speaker, who acknowledges potential bias as someone in the Y Combinator ecosystem and as a paying customer/competitor of OpenAI, spent significant time investigating each claim. The analysis systematically deconstructs accusations ranging from SEC filing errors to... [6]

