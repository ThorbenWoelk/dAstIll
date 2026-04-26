# Chat Capability Sweep Results

- Generated: `2026-04-23T15:11:31.256506+00:00`
- Base URL: `http://localhost:3544`
- Dataset: `/Users/thorben.woelk/repos/dAstIll/backend/tests/data/chat_capability_prompts.json`
- Prompt count: `10`

## Summary

- Passed prompts: `5/10`
- Answerability pass: `10/10`
- Grounding pass: `10/10`
- Shape pass: `5/10`
- Average score: `2.50`

## Capability Classes

- `tone_or_style_inference`: passed `5/10`, avg score `2.50`, failures `shape_mismatch`

## Failures By Class

- `shape_mismatch`: q024, q025, q087, q088, q089

## Prompt Results

### q024 FAIL

- Prompt: What is the most confusing or uncertain part of the discussion?
- Class: `tone_or_style_inference`
- Status: `Completed`
- Score: `2`
- Sources: `4`
- Failure: `shape_mismatch`
- Source videos: I can't believe this is a real statistic... | So I've had gpt-5 for a bit now... | The Wordpress Drama Interview (this got cited in a lawsuit lol) | Where Should You Deploy In 2026?
- Notes: tone or style inference answer did not include a visible caveat

#### Answer

Retrieved evidence for: What is the most confusing or uncertain part of the discussion?

The answer model is unavailable, so this fallback lists the highest-ranked saved excerpts instead of synthesizing beyond them.

1. The Wordpress Drama Interview (this got cited in a lawsuit lol) - Theo - t3․gg: I'm going to follow now because I'm learning stuff from you so thank you this is what I'm here for is nering out about these deep detail things but uh well can we can we ask about the uh you know WP engine WordPress engine confusion yeah um I can do that how do you want to phrase that um so could be a be confused as one thing but there's a lot of other things like I mentioned before with like next doth with next UI w... [1]
2. So I've had gpt-5 for a bit now... - Theo - t3․gg: GPT-5's public release, meaning the creator may have seen things others haven't yet. Skatebench Performance Skatebench is a benchmark testing how well models can name skateboarding tricks—described as not the most meaningful benchmark but interesting for its range of results. Previous best model scores were around 70%, and o3 Pro achieved 93-94%. GPT-5 achieved a perfect 100% score initially at the OpenAI office, an.... [2]
3. Where Should You Deploy In 2026? - Theo - t3․gg: TL;DR For most applications (98%+), serverless deployment options are sufficient and recommended as a starting point; move to VPS only if you encounter specific needs. Top recommendations (S-tier): Vercel for serverless, Railway and Render for VPS — all offer excellent developer experience, reliability, and reasonable pricing. Cloudflare offers the lowest costs due to unique infrastructure (no Docker, uses V8 isolate... [3]
4. I can't believe this is a real statistic... - Theo - t3․gg: to it so he has to tell me when there's good emails sorry Gabriel I need someone to keep up with this how do you feel are you a ghost engineer or are you working with a whole bunch of them let me know what you think and until next time fire the useless people [4]

### q025 FAIL

- Prompt: What does the speaker assume the audience already knows?
- Class: `tone_or_style_inference`
- Status: `Completed`
- Score: `2`
- Sources: `2`
- Failure: `shape_mismatch`
- Source videos: What is Theo's Worst Take? | “Just Use HTML”
- Notes: tone or style inference answer did not include a visible caveat

#### Answer

Retrieved evidence for: What does the speaker assume the audience already knows?

The answer model is unavailable, so this fallback lists the highest-ranked saved excerpts instead of synthesizing beyond them.

1. What is Theo's Worst Take? - Theo - t3․gg / Overview: This brief exchange involves a discussion about identifying the speaker's worst take or opinion. The conversation touches on the speaker's self-assessment of their takes, a past controversial statement, and a specific critique of a storybook item. The dialogue ends with one speaker conceding a point about the storybook's utility. [1]
2. “Just Use HTML” - Theo - t3․gg: [ __ ] websites, but I know [ __ ] better than to pat out this video any [ __ ] more. So, I'm just going to be [ __ ] done. Let me know what you think. Until next time, [ __ ] [2]

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

1. It’s time to embrace the AI - Theo - t3․gg / Key Points: e. His perspective changed through using Cursor extensively (tab autocomplete, command-I, command-K features), building T3 Chat, and working with newer agents and models. He notes that conversations with still-skeptical friends now feel strange because his views have shifted so dramatically after giving the tools a serious try when they improved. The Article and Its Author The source article is "A heartfelt provocati... [1]
2. What is Theo's Worst Take? - Theo - t3․gg: and what is my worst take you said all of your takes were good and so maybe that's your worst take I remember you saying something evil about go there were so many of them uh that story book is useless why is it not useless uh because you you need to fill up your known module somehow you got me on that one [2]
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
- Sources: `4`
- Failure: `-`
- Source videos: GlazeGPT got rolled back (4o update gone wrong) | React is killing the web | So close to Opus at 1/10th the price (GLM-4.7 and Minimax M2.1 showdown) | Why is everyone so unhappy with JavaScript?

#### Answer

Retrieved evidence for: What are the most optimistic views in my library?

The answer model is unavailable, so this fallback lists the highest-ranked saved excerpts instead of synthesizing beyond them.

1. React is killing the web - Theo - t3․gg: in Chrome that have been made around how yielding occurs during these asynchronous workloads. Suspense lets you break the page into loadable chunks. Transitions let you prioritize user input and show immediate optimistic updates. Activity lets you defer hidden content. View transition will let you coordinate animations for batches of UI. Suspense list will let you specify the order that UI loads in. all with simple d... [1]
2. So close to Opus at 1/10th the price (GLM-4.7 and Minimax M2.1 showdown) - Theo - t3․gg: ely 1/60th of Opus 4.1's original pricing). GLM 4.7 excels at UI/design tasks and visual outputs, while MiniMax M2.1 excels at long-running coding tasks, planning, and sustained multi-file changes. Both models are open-weight (M2.1 weights expected to drop around Christmas), runnable on consumer hardware, and represent a major shift in what's possible for budget-conscious developers. Overview This video provides an i... [2]
3. GlazeGPT got rolled back (4o update gone wrong) - Theo - t3․gg: r than providing grounded responses. Unlike the pre-internet era, modern technology and AI can validate and amplify fringe beliefs and delusions without the social checks that previously existed. The speaker shares an example where a friend's conversation with ChatGPT spiraled into nonsense involving made-up scientific terms because the model simply reinforced whatever context was provided. OpenAI's response includes... [3]
4. Why is everyone so unhappy with JavaScript? - Theo - t3․gg / Key Points: Most want native JS types to resemble TypeScript. Developer Happiness Trends **The disturbing pattern**: Everything is moving left toward negative sentiment—even tools like Vite and frameworks that didn't have major changes. Angular is the only thing that moved right (more positive) because it was already so far left and made improvements. **Web tech happiness**: Hasn't moved in five years. **JS happiness**: Slightly... [4]

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

1. What happened to me? - Theo - t3․gg: this can change the same way it changed here. If I start doing more of these types of videos and they perform better than expected, I'll lean more into this. For example, the logging video, we screwed up the export initially and the first version that went up was too short and had most of the content missing. So, we had to re-upload it, which destroys the video performance because a lot of people already saw it, so t... [1]
2. I might have a new favorite state manager... - Theo - t3․gg / Key Points: is a hack. **Cleaner Selectors**: In Zustand, you have to select functions off the store as if they were values, even though they never change. This is conceptually confusing—functions should just be callable, not selected. Event-Driven Architecture **Store.send API**: XState Store uses `store.send({ type: 'increasePopulation', by: 10 })` to trigger transitions. This is fundamentally different from Zustand's direct f... [2]
3. This awesome CSS feature is blocked by drama (Google and Apple can't agree) - Theo - t3․gg: read in a second but I want to start with Adam argy's comments because he's he's Deep In The Weeds here also works at Google has a lot of things to to say in all of these and I'm excited for his thoughts here are his points on why he doesn't like grid level three as the way to do masonry point one a masonry layout isn't a grid there is no shared row lines only columns so it has to ignore all sorts of syntax to accomm... [3]
4. What happened to me? - Theo - t3․gg: audience didn't like the video, that's why it didn't perform. Oh yeah, maybe I should make a video the audience likes. It really does come down to that. There are layers to this, like is the video clickable? Does it start in a way that's entertaining and interesting enough that you continue to watch from there? All of these pieces are important, but I'm not blocked by the algorithm for making certain content. In fact... [4]
5. The Actual Dumbest Thing About Try/Catch - Theo - t3․gg: I'll be honest error handling in JavaScript kind of sucks I know hot take but that's what we're here for right seriously though try catch is it's a disaster there's a lot of subtlety for the things that are wrong with it but there's one particular piece that I don't think it's talked about enough and I saw a tweet that inspired me to make this video the piece that we're talking about here is the scoping huge shout ou... [5]
6. It’s actually over now - Theo - t3․gg: started in a garage doing like door-to-door sales and showing off to computer nerds. You don't start with the fancy marketing video. You start by being real humans. And they tried a little too hard to do the marketing thing. And what's really funny is I talked to a lot of these earlier stage companies and they want to do their own elaborate YouTube stuff. Both because they see me as a YouTuber. They're like, "Hey, ho... [6]
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

1. What happened to me? - Theo - t3․gg: this can change the same way it changed here. If I start doing more of these types of videos and they perform better than expected, I'll lean more into this. For example, the logging video, we screwed up the export initially and the first version that went up was too short and had most of the content missing. So, we had to re-upload it, which destroys the video performance because a lot of people already saw it, so t... [1]
2. OpenAI’s TikTok Clone Is Interesting… - Theo - t3․gg / Key Points: Model Architecture and Technical Implementation Underlying Generation Method**: The creator theorizes that Sora 2 isn't just a single video model but rather an LLM generating screenplays and plans that command other models to generate video pieces, which are then stitched together. This explains how videos exceed the typical 5-second limit seen in other video models. The model appears to generate audio first, then cr... [2]
3. What happened to me? - Theo - t3․gg: of my community, the people hanging out in Twitch chat right now who have been there since day one, watched this whole thing happen, many of which joined me on the same journey. So, knowing all of this, I want to talk a bit about how I choose a topic for a video because another one of the questions I get all of the time is, "What is your dream video that you would do if the algorithm wouldn't kill it?" A question I g... [3]
4. Defending a disaster (modern frontend development rant) - Theo - t3․gg: and second most recruiting agencies are garbage you shouldn't need me to tell you that we all get spammed with them every day they have no idea what they're doing they're not even technical not only is g2i technical they are some of the most technical this is the crew that runs react Miami which is my favorite react conference and it's not even close the amount of fun I had there last year was unbelievable and I will... [4]
5. OpenAI’s TikTok Clone Is Interesting… - Theo - t3․gg / TL;DR: OpenAI released a new Sora mobile app that combines AI video generation with a TikTok-style social feed, featuring character consistency through a "Cameo" feature and longer-form videos with audio-video synchronization. The model demonstrates notable technical improvements including music generation with hooks and decent delivery, plus J-cut/L-cut editing techniques, though video generation remains expensive with a 5... [5]
6. “Just Use HTML” - Theo - t3․gg: [ __ ] websites, but I know [ __ ] better than to pat out this video any [ __ ] more. So, I'm just going to be [ __ ] done. Let me know what you think. Until next time, [ __ ] [6]
7. JavaScript runs on literally everything now - Theo - t3․gg: and even parts of the operating system are being moved to JavaScript and react native at least I'm safe on my Mac and on my PlayStation and my other consoles right well obviously the Xbox is running react native too I hope that's kind of obvious because react native Windows Xbox also kind of Windows what might surprise you is another console it's kind of a poorly kept secret but the PlayStation 5 uses react native 2.... [7]
8. How did we get here? (A rant about Javascript runtimes) - Theo - t3․gg / Key Points: GJS, MUJS, JScript, jsdb, njs, TeX, bear, other low.js variants [8]
9. What happened to me? - Theo - t3․gg: audience didn't like the video, that's why it didn't perform. Oh yeah, maybe I should make a video the audience likes. It really does come down to that. There are layers to this, like is the video clickable? Does it start in a way that's entertaining and interesting enough that you continue to watch from there? All of these pieces are important, but I'm not blocked by the algorithm for making certain content. In fact... [9]
10. What happens now? - Theo - t3․gg / Overview: This video is a deep dive response to an article by Chris Gregory about how AI tools like Claude Code and Cursor are fundamentally changing software development. The speaker explores the thesis that while code has become cheap to produce, software remains expensive because the real costs — problem understanding, maintenance, distribution, and architecture — haven't changed. The discussion covers the rise of "disposab... [10]
11. What happened to me? - Theo - t3․gg: have gotten 5k plays. A out of 10 would have gotten 40k plays. a 10 out of 10 would have gotten like k plays. That was the range before. The weird thing that's happened is due to the massive change in who is watching my channel and the interest of the people who are watching is the gap between these has gotten massive. Even a six, seven or eight out of 10 topic is going to perform significantly worse. This has been w... [11]
12. OpenAI’s TikTok Clone Is Interesting… - Theo - t3․gg: TL;DR OpenAI released a new Sora mobile app that combines AI video generation with a TikTok-style social feed, featuring character consistency through a "Cameo" feature and longer-form videos with audio-video synchronization. The model demonstrates notable technical improvements including music generation with hooks and decent delivery, plus J-cut/L-cut editing techniques, though video generation remains expensive wi... [12]

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

### q087 FAIL

- Prompt: What is the overall tone of this video?
- Class: `tone_or_style_inference`
- Status: `Completed`
- Score: `2`
- Sources: `12`
- Failure: `shape_mismatch`
- Source videos: Defending a disaster (modern frontend development rant) | I hate that this is still happening | Open source is dying | Sonnet 4.5 is the best coding model in the world
- Notes: tone or style inference answer did not include a visible caveat

#### Answer

Retrieved evidence for: +{Open source is dead now?} What is the overall tone of this video?

The answer model is unavailable, so this fallback lists the highest-ranked saved excerpts instead of synthesizing beyond them.

1. Open source is dying - Theo - t3․gg: escalate because you almost certainly have to use AI to actually scan these PRs. It has a lot of config, enough config that I'm almost certain that this project was vibe coded, but yeah, you get the idea. There are a lot of solutions being made to try and fix these problems. Some of them are going to make it harder for new maintainers to break out, which sucks because we might just have our current maintainers until ... [1]
2. Open source is dying - Theo - t3․gg / Overview: The video presents a comprehensive examination of how AI is negatively impacting the open source ecosystem. The speaker, an experienced open source maintainer and creator of T3 Code, shares firsthand experiences managing a new project that received 150 PRs in just 5 days. The discussion covers four main problem areas: PR spam and quality degradation, increasingly hostile and confused users, GitHub's inadequate platfo... [2]
3. Sonnet 4.5 is the best coding model in the world - Theo - t3․gg: too many bullet points, but generally I found the tone of clawed models to be really good. And if you want a UI that's as good as its tone, check out T3 Chat where you get access to literally every single model for eight bucks a month. It's a pretty absurd deal if you ask me. And I'll make it a little bit more absurd. Use code kind of safe at checkout and you'll get your first month for just $1. Go to t3.hat and chec... [3]
4. Defending a disaster (modern frontend development rant) - Theo - t3․gg: ning Astro and Tailwind. The discussion covers SSR history, CSS methodologies, framework evolution, build complexity, and the "Alex Russell problem" (judging technologies by worst-case implementations). Theo's overall assessment: the author's diagnosis of problems sometimes aligns, but solutions and alternatives are outdated or overlook why modern tools became popular. Overview This video is a detailed reaction by Th... [4]
5. Open source is dying - Theo - t3․gg: put so much effort into killing Hacktoberfest. I think this video of mine, don't contribute to open source, is one of the best videos I ever filmed. Not sure who that blonde guy with the mustache is though. Seriously though, like that video, I have been told by so many maintainers how thankful they are for this video more than almost anything I've done as a developer and journalist, YouTuber, whatever you want to cal... [5]
6. Open source is dying - Theo - t3․gg: poorest. Really crippled the image I had in relation to your channel and content. I have been contributing to projects since before you were born. such an attitude, including some YouTube codes of yours that I've never received before. Feel free to block me. Mature. This is somebody who never wrote code before AI. Straight up. And I promise you, you were not contributing to code years ago, [ __ ] And this is the hot ... [6]
7. I hate that this is still happening - Theo - t3․gg: especially in India. So maybe go do that. That's all I got on this one. Please stop spamming these repos with open source. And if you harass anyone as a result of this video, just know I'll be disappointed as [ __ ] I got nothing else to say on this. Let me know what you guys think. And until next time, peace nerds. [7]
8. I hate that this is still happening - Theo - t3․gg: use to make them is very different from the tech I started with. The best thing to make your first video with is the things you already have. You shouldn't buy a bunch of new stuff to inspire you to make the first video. You should do it despite not having the right equipment. And once you get good at it, you'll figure out what your equipment can and can't do and make changes based on what you know. And this is the r... [8]
9. I hate that this is still happening - Theo - t3․gg / TL;DR: A Git/GitHub tutorial by Apna College (6-7 million views) continues to cause thousands of spam PRs on the ExpressJS repository, wasting maintainer time. The video creator argues that Apna College's response has been inadequate—only editing out ~5 seconds to 1 minute after years of damage, and deflecting blame onto students. Open source contribution is being misunderstood as a "magic gateway" to jobs; the video explai... [9]
10. I hate that this is still happening - Theo - t3․gg: Update readme.md. Action. Update readme.md. Naveen kumar. Update readme.md. Ria. Update readme.momd. Update again readme.md. Update readme.momd. Update readme. Update readme. Update readme. Update readme. Update readme. Update readme. I'm going to go actually insane. For those who haven't been around for a long time, I'm Theo. I make videos about software dev stuff. I care a lot about open source, which is why this i... [10]
11. Open source is dying - Theo - t3․gg: want to highlight one particular PR that annoyed me. We had a stailed to-do MD file in the repo that had random things I was working on at some point in it. And someone filed a nonsense PR that tried to fix all of those things and ended up just breaking other things. They didn't get any response from us cuz we were being flooded with PRs. So he randomly tags me and two other people whose PRs merged recently. I was so... [11]
12. Open source is dying - Theo - t3․gg: They're already on the line of giving up. They suddenly have more reason to give up. Significantly more reason. That sucks. That has the potential to cause real long-term damage in this industry. This is how things like the XZ back door happen. If you're not familiar with the story, I'll TLDDR quick. XC is a really important compression library used by a ton of open source software, especially in the Linux ecosystem.... [12]

### q088 FAIL

- Prompt: Is the speaker confident, cautious, or speculative?
- Class: `tone_or_style_inference`
- Status: `Completed`
- Score: `2`
- Sources: `2`
- Failure: `shape_mismatch`
- Source videos: I can't believe he was right. | Will Manifest V3 Kill Chrome?
- Notes: tone or style inference answer did not include a visible caveat

#### Answer

Retrieved evidence for: Is the speaker confident, cautious, or speculative?

The answer model is unavailable, so this fallback lists the highest-ranked saved excerpts instead of synthesizing beyond them.

1. Will Manifest V3 Kill Chrome? - Theo - t3․gg: we'll see how long they're able to support that I think that's all I got how are you feeling though are you staying on Chrome or is this the push you needed to move off let me know in the comments and until next time peace nards [1]
2. I can't believe he was right. - Theo - t3․gg: as I do today, even if my relationship with it is very different than it was a year ago. And I recommend that you reflect yourself and give these things a try. Let me know what y'all think and how you're using these tools today. [2]

### q089 FAIL

- Prompt: Does this video sound more like a tutorial, a review, or a discussion?
- Class: `tone_or_style_inference`
- Status: `Completed`
- Score: `2`
- Sources: `12`
- Failure: `shape_mismatch`
- Source videos: Gemini Flash 3 is my new favorite model (yes really) | I need you guys to trust me on this (sorry Anthropic) | Okay, I'm a bit scared now... | Open source is dying | React feels insane | This magic hack makes Next.js possible | We need to talk about Sonnet 4.6 | What happened to me? | Why I moved away from SQL
- Notes: tone or style inference answer did not include a visible caveat

#### Answer

Retrieved evidence for: +{Open source is dead now?} Does this video sound more like a tutorial, a review, or a discussion?

The answer model is unavailable, so this fallback lists the highest-ranked saved excerpts instead of synthesizing beyond them.

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

