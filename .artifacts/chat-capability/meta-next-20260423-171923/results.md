# Chat Capability Sweep Results

- Generated: `2026-04-23T15:19:50.547498+00:00`
- Base URL: `http://localhost:3544`
- Dataset: `/Users/thorben.woelk/repos/dAstIll/backend/tests/data/chat_capability_prompts.json`
- Prompt count: `5`

## Summary

- Passed prompts: `5/5`
- Answerability pass: `5/5`
- Grounding pass: `5/5`
- Shape pass: `5/5`
- Average score: `3.00`

## Capability Classes

- `meta_learning_or_next_step`: passed `5/5`, avg score `3.00`, failures `-`

## Failures By Class


## Prompt Results

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

1. Did Meta Really Fake Benchmarks? - Theo - t3․gg: dal is just it can handle different things. So things like text, images, video, audio. Multimodal means it can handle different things with just one model. Mixture of experts is the thing I'm talking about for most of the section where I talk about how the parameters are split across different things. Wanted to include this clarification rather than be wrong in the video. Anyways, their focus is on multimodal intelli... [1]
2. What happened to me? - Theo - t3․gg: of the time offline. And to honor the thing I always do on this channel, I'm going to film this video because it's a thing I want to talk about. And in the end, that's what my videos are. But hopefully the people who don't comment, which is over % of you, might have some of your concerns eased by this. My videos aren't based on what makes me money. My money is based on what's exciting to me, much like my topics are, ... [2]
3. What happened to me? - Theo - t3․gg: this can change the same way it changed here. If I start doing more of these types of videos and they perform better than expected, I'll lean more into this. For example, the logging video, we screwed up the export initially and the first version that went up was too short and had most of the content missing. So, we had to re-upload it, which destroys the video performance because a lot of people already saw it, so t... [3]
4. "AI Startups" are over done (finally) - Theo - t3․gg: AI. How many devs are going to try out that [ __ ] thing? The answer is none. Because nobody is trying to replace themselves at their job. They're trying to make the boring parts less boring, the hard parts less hard, and the fun parts more relevant in their day-to-day lives. And that was what Copilot did well. So obviously why combinator companies had to adjust because too many of them were making these types of mis... [4]
5. What happened to me? - Theo - t3․gg: because a ton of other big open source projects are using it from Post Hog to Mastra to Nvidia Storybook Raycast and many more. Let's pick a Raycast one. I love Raycast. Here there was a rough case where custom npx path could have come in as an empty string which would have broken this check. And here we have a trim call that's going to handle that for you. Super easy to fix. Here's a PR somebody opened skate bench i... [5]
6. Which browser should you use right now? - Theo - t3․gg: rtical real estate. It has the worst vertical real estate of any browser I've used. Can't even fit the blog item on the page at the same zoom level. But here's what I wanted. I had done this post in June. What are your biggest frustrations with T3 chat right now? And I wanted to collect all of this data. So, let's do it. Summarize all of the replies to this post. Make sure you check all 500 plus of them. Seems like i... [6]
7. Prisma is removing Rust? - Theo - t3․gg: becomes huge they might change their mind cool personally if you're making a new database I think you should be focused on building a really good typescript orm yourselves something like eddb for example they are rethinking how to work with a relational DB where they're going more relational instead of less most nosql databases have less relational behaviors they have way more and technically they're built on top of.... [7]
8. Anthropic is trying SO hard to fix MCP... - Theo - t3․gg: accuracy issues, building MCB powered systems with multiple servers, or there's more than 10 tools available. It's less beneficial with a small tool library. Less than 10 tools is small now. Oh god. All tools are used frequently in every session and tool definitions are compact. These are when you wouldn't use it. Cool. And then we have programmatic tool calling. This is what we discussed in the previous video where.... [8]
9. Anthropic is lying to us. - Theo - t3․gg: ologizing and saying that this is legit. If you guys don't do that, I am going to just assume you're lying because every single thing is pointing to that. Whether or not this paragraph is true, it is no longer relevant and everything else you guys have said is either verifiably a lie or just makes no sense in the first place. and they seem to even know that too. They have this prompt that they're claiming was used fo... [9]
10. AI images just got dangerously good (RIP diffusion??) - Theo - t3․gg: there music? There better not be music. Every time. Every time. I just want to watch these videos without getting DMCA struck. They trained the model on a joint distribution of images and text, learning not just how images relate to language, but how they relate to each other. Combined with aggressive post training, the resulting model has surprising visual fluency capable of generating images that are useful, consis... [10]

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

1. What happened to me? - Theo - t3․gg: of my community, the people hanging out in Twitch chat right now who have been there since day one, watched this whole thing happen, many of which joined me on the same journey. So, knowing all of this, I want to talk a bit about how I choose a topic for a video because another one of the questions I get all of the time is, "What is your dream video that you would do if the algorithm wouldn't kill it?" A question I g... [1]
2. Open source is dying - Theo - t3․gg / Overview: The video presents a comprehensive examination of how AI is negatively impacting the open source ecosystem. The speaker, an experienced open source maintainer and creator of T3 Code, shares firsthand experiences managing a new project that received 150 PRs in just 5 days. The discussion covers four main problem areas: PR spam and quality degradation, increasingly hostile and confused users, GitHub's inadequate platfo... [2]
3. Open source is dying - Theo - t3․gg: escalate because you almost certainly have to use AI to actually scan these PRs. It has a lot of config, enough config that I'm almost certain that this project was vibe coded, but yeah, you get the idea. There are a lot of solutions being made to try and fix these problems. Some of them are going to make it harder for new maintainers to break out, which sucks because we might just have our current maintainers until ... [3]
4. What happened to me? - Theo - t3․gg: plays, I don't film the video. If I don't care, I don't care. You cannot get me to do a video I don't feel like doing. You cannot pay me any amount of money to talk about something I don't want to talk about. Want to know a really funny thing? Probably shouldn't be sharing. In my onboarding email that I send to brands when I'm starting to work with them as sponsors, I have an FAQ section. And one of the questions in.... [4]
5. Open source is dying - Theo - t3․gg: put so much effort into killing Hacktoberfest. I think this video of mine, don't contribute to open source, is one of the best videos I ever filmed. Not sure who that blonde guy with the mustache is though. Seriously though, like that video, I have been told by so many maintainers how thankful they are for this video more than almost anything I've done as a developer and journalist, YouTuber, whatever you want to cal... [5]
6. My hot take on image formats - Theo - t3․gg: and drop it into word it doesn't show up properly if they try to send it as an email attachment it is a file instead of an image these types of things are real complaints and they make a lot of sense but the Alternatives I've seen people propose make none another way to look at this video isn't so much a rant about why I love webp rather this is a rant but why I hate av1 yes I'm going to come out and say it the avif.... [6]
7. Open source is dying - Theo - t3․gg: They're already on the line of giving up. They suddenly have more reason to give up. Significantly more reason. That sucks. That has the potential to cause real long-term damage in this industry. This is how things like the XZ back door happen. If you're not familiar with the story, I'll TLDDR quick. XC is a really important compression library used by a ton of open source software, especially in the Linux ecosystem.... [7]
8. JavaScript Frameworks in 2025 - Theo - t3․gg: is absolutely own language it has been for a bit but this is the like tripling down on it I'm kind of disappointed that to my memorization questions on interviews everyone can now just answer with just use the compiler man very fair point you know times our are stupidly so this is why you guys got to watch my react compiler content I go so deep on these things and no one cares it's yeah the I think we're more aligned... [8]
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

1. What happened to me? - Theo - t3․gg: of my community, the people hanging out in Twitch chat right now who have been there since day one, watched this whole thing happen, many of which joined me on the same journey. So, knowing all of this, I want to talk a bit about how I choose a topic for a video because another one of the questions I get all of the time is, "What is your dream video that you would do if the algorithm wouldn't kill it?" A question I g... [1]
2. Open source is dying - Theo - t3․gg / Overview: The video presents a comprehensive examination of how AI is negatively impacting the open source ecosystem. The speaker, an experienced open source maintainer and creator of T3 Code, shares firsthand experiences managing a new project that received 150 PRs in just 5 days. The discussion covers four main problem areas: PR spam and quality degradation, increasingly hostile and confused users, GitHub's inadequate platfo... [2]
3. Open source is dying - Theo - t3․gg: escalate because you almost certainly have to use AI to actually scan these PRs. It has a lot of config, enough config that I'm almost certain that this project was vibe coded, but yeah, you get the idea. There are a lot of solutions being made to try and fix these problems. Some of them are going to make it harder for new maintainers to break out, which sucks because we might just have our current maintainers until ... [3]
4. What happened to me? - Theo - t3․gg: plays, I don't film the video. If I don't care, I don't care. You cannot get me to do a video I don't feel like doing. You cannot pay me any amount of money to talk about something I don't want to talk about. Want to know a really funny thing? Probably shouldn't be sharing. In my onboarding email that I send to brands when I'm starting to work with them as sponsors, I have an FAQ section. And one of the questions in.... [4]
5. Open source is dying - Theo - t3․gg: put so much effort into killing Hacktoberfest. I think this video of mine, don't contribute to open source, is one of the best videos I ever filmed. Not sure who that blonde guy with the mustache is though. Seriously though, like that video, I have been told by so many maintainers how thankful they are for this video more than almost anything I've done as a developer and journalist, YouTuber, whatever you want to cal... [5]
6. My hot take on image formats - Theo - t3․gg: and drop it into word it doesn't show up properly if they try to send it as an email attachment it is a file instead of an image these types of things are real complaints and they make a lot of sense but the Alternatives I've seen people propose make none another way to look at this video isn't so much a rant about why I love webp rather this is a rant but why I hate av1 yes I'm going to come out and say it the avif.... [6]
7. Open source is dying - Theo - t3․gg: They're already on the line of giving up. They suddenly have more reason to give up. Significantly more reason. That sucks. That has the potential to cause real long-term damage in this industry. This is how things like the XZ back door happen. If you're not familiar with the story, I'll TLDDR quick. XC is a really important compression library used by a ton of open source software, especially in the Linux ecosystem.... [7]
8. JavaScript Frameworks in 2025 - Theo - t3․gg: is absolutely own language it has been for a bit but this is the like tripling down on it I'm kind of disappointed that to my memorization questions on interviews everyone can now just answer with just use the compiler man very fair point you know times our are stupidly so this is why you guys got to watch my react compiler content I go so deep on these things and no one cares it's yeah the I think we're more aligned... [8]
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

1. What happened to me? - Theo - t3․gg: of my community, the people hanging out in Twitch chat right now who have been there since day one, watched this whole thing happen, many of which joined me on the same journey. So, knowing all of this, I want to talk a bit about how I choose a topic for a video because another one of the questions I get all of the time is, "What is your dream video that you would do if the algorithm wouldn't kill it?" A question I g... [1]
2. I need you guys to trust me on this (sorry Anthropic) - Theo - t3․gg / Key Points: ecause labs make hosting nearly impossible through generous subscriptions paired with expensive APIs. Community Confusion and Anthropic's Lack of Clarity The creator has received DMs from many prominent people asking for insights because official answers are nonexistent. Matt Pocock (known for TypeScript work) publicly asked two questions: (1) Can he use an OAuth token from a subscription to power the Claude Agent SD... [2]
3. Claude Cowork: a small taste of AGI - Theo - t3․gg: thing hard to know for sure seems potentially very good says that co-work can only access files that you grant access to. It looks to me like they're mounting those files in a containerized environment, which should mean we can trust co-work not to be able to access things outside of the sandbox. Here's the reply he got with his question about drafts. Most ready to publish frequently argued questions against LLMs cl.... [3]
4. The drama never ends... - Theo - t3․gg: but it's one I felt I had to. I wanted to do my best to cover this reasonably, and I hope you see that for what it is. Let me know what I did right, and more importantly, what I could do better on. And until next time, peace nerds. [4]
5. What happens now? - Theo - t3․gg: complicated, then everyone could be a YouTuber. Cuz that's the hard part. Cuz that's the first problem you ran into. The radio thing even happens to an extent here, too. If the airplane radios were easier, everyone could land the plane. No, you [ __ ] can't. Be realistic here. 34 of men answer yes to this question. Fun fact, the majority of men think they can land the plane. I bring this up because of a real conversa... [5]
6. I need you guys to trust me on this (sorry Anthropic) - Theo - t3․gg: trying to relieve his confusion here and assumed as somebody who just asked nicely that they would get an answer. Clearly not familiar with how Anthropic does things. Again, as always, no shade to Thoric. He has been put between a rock and a hard place here where he clearly wants to do right to the community, but he's not allowed to answer the important hard questions here. Sorry, this has been confusing. I know we s... [6]
7. I can't believe he was right. - Theo - t3․gg / Key Points: an screenshot a problem, show it to an AI, and iterate—potentially never understanding the underlying issue. **Uncertainty about solutions**: The creator expresses genuine confusion about how junior developers should learn now and may create a dedicated follow-up video on this topic. **Recommendations for early engineers**: Read generated code, especially for projects meant to be maintained Use chat apps to ask quest... [7]
8. Claude Code's latest update is really cool (when it works...) - Theo - t3․gg: t has every model in a row showing success fail rates, average time to complete, and average cost. I think I have costs built into here right now. I might not. Write me a plan for implementing all of this. You should probably use ink for the UI UX portion, but I'm down for other suggestions. Okay, it's asking if I want to do plan mode. Cool. We'll do plan mode. Do I want to proceed? Let it read the metrics file. Sure... [8]
9. Anthropic study shows AI makes devs dumb - Theo - t3․gg / Key Points: following a "generation then comprehension" approach. These participants generated code, manually copied/pasted it, then asked follow-up questions to improve understanding. Though not particularly fast, they showed higher quiz scores (65%+). A hybrid approach involving code generation with explanations was also noted, though it took more time. **Study Limitations and Criticisms**: The author critiques the study's des... [9]
10. GPT-5.1 is built for normies - Theo - t3․gg: eople that are currently on 40 and make them go nuts. One of my friends who's like deep in the mental health world here, Jason, said that it's going full therapist mode and he approves of it. Not that like you should use it as an alternative to therapy to be very very explicit and clear, but it's less likely to send you down a really dangerous rabbit hole like the other models previously might have. So again, with 4,... [10]

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

1. Zod finally has competition (...created by Zod?) - Theo - t3․gg: ion standards. Colin (Zod's creator) worked with creators of Valibot and Arktype to develop the "Standard Schema" spec—a common interface for multiple validation libraries. Standard Schema allows framework and library authors to support multiple validators (Zod, Valibot, Arktype) without writing separate adapters for each. The spec is designed for library/framework authors, not end users; it enables ecosystem-wide in... [1]
2. Why Everyone Hates Web Components - Theo - t3․gg: and make changes to it something else I'd say is similar would be like a material UI and other UI libraries these two are pretty close to the base HTML element thing what we have learned as an ecosystem is if you're not this being close to it kind of sucks and that's why we've moved the other way where we have stuff like Shaden and Shaden looks and operates similarly to material UI where you have all the components y... [2]
3. I don’t really use libraries anymore - Theo - t3․gg / Takeaways: Re-evaluate your dependencies**: Go through your package.json and assess whether each library is necessary, whether it's holding you back, and whether AI could help you implement an alternative. **Consider the new calculus**: The math has changed—problem difficulty is lower with AI, perceived risk of dependencies is higher, so the threshold for adopting libraries should be higher. **Think about control**: When you ad... [3]
4. I don’t really use libraries anymore - Theo - t3․gg / Key Points: for understanding different library types: **Libraries beyond your knowledge**: These are used by people who don't know how to solve the problem themselves. Examples include `is-odd` (literally one line of code) and `leftpad`. The argument against these is that users are outsourcing competency and taking on supply chain risks without understanding them. **Libraries for tedious reimplementation**: Even capable develop... [4]
5. Is it time to move on? - Theo - t3․gg: I recommend too many new things because I don't and if your goal is to get a job learn old things learn something that's react or older if you really want to get a job ASAP go learn Cobalt or some [ __ ] something ancient job opportunities and productivity opportunities tend to come from using oldish Solutions there's a reason PHP has this huge wave right now of Indie hackers because old things are great they're stab... [5]
6. React Native Just Got 550% Faster - Theo - t3․gg: supported: Suspense, Transitions, Automatic Batching, and `useLayoutEffect` finally work correctly in React Native. The new architecture enables concurrent rendering across multiple threads, allowing priority-based updates that can interrupt ongoing renders. Native modules are now written in C++ (Turbo Modules), enabling type-safe, cross-platform code sharing with lazy loading for faster startup. New React Native De.... [6]
7. This might change how we build UI forever - Theo - t3․gg: n, dependency management, and a registry schema that opens possibilities for distributing any type of code beyond just UI components. Key Points Shad CN Philosophy and Architecture Shad CN is not a traditional library you install; it provides components that are added directly to your project, meaning you own and can modify all the code. It uses Radix UI (a headless UI library) for behaviors like dialog, dropdown men... [7]
8. I don’t really use libraries anymore - Theo - t3․gg / Overview: This video explores how AI-assisted development is fundamentally changing the role and utility of software libraries. The speaker, a developer who has built many projects using various libraries, shares his evolving perspective on dependency management in an era where AI can generate implementations. He discusses his personal experience removing libraries like Tkumi from projects, examines industry examples like Anti... [8]
9. I don’t really use libraries anymore - Theo - t3․gg: are still around in 2026. Because as frustrating and annoying as they are to work with, there are often dependencies that companies are building on and around that expect tools like Webpack to work a certain way so that you could integrate them into your codebase a certain way. A tool like this fast float dep is expecting g++ to be installed. Sadly, a lot of Linux distributions don't have that installed even when you... [9]
10. Why is everyone so unhappy with JavaScript? - Theo - t3․gg: arrator actively uses Sets for unique key collections with union/intersection operations but observes they're rarely used in others' code. **Object.groupBy**: New feature allowing creation of objects with keys based on a grouping function—could be useful for coding tasks. **Browser APIs**: WebSockets widely used; PWA usage at 49% (narrator suspects this is declining); geolocation API at 35%—twice as high as WebGL whi... [10]
11. Svelte 5 Is Like React, But Better - Theo - t3․gg: s history. The Svelte team maintains both Svelte and SvelteKit; SvelteKit is presented as one of the best meta-frameworks currently available, praised for its clean data loading system, form actions, and quick learning curve. The website documentation was overhauled from a split structure (svelte.dev, kit.svelte.dev, learn.svelte.dev) into a unified, redesigned experience. Why Svelte Changed: Addressing Limitations i... [11]
12. The Biggest React Framework You've Never Heard of - Theo - t3․gg: b stars, offering a comprehensive alternative to Next.js and Remix with unique features like built-in micro-frontends support and JSX+ extensions. The framework uses React Router under the hood and offers file-based routing similar to Next.js, but with distinct capabilities like SSG by default (opting into SSR requires configuration), a unique data loading pattern, and React Query-like request handling. JSX+ introduc... [12]

