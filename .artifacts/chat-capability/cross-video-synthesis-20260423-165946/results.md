# Chat Capability Sweep Results

- Generated: `2026-04-23T15:00:30.660289+00:00`
- Base URL: `http://localhost:3544`
- Dataset: `/Users/thorben.woelk/repos/dAstIll/backend/tests/data/chat_capability_prompts.json`
- Prompt count: `9`

## Summary

- Passed prompts: `9/9`
- Answerability pass: `9/9`
- Grounding pass: `9/9`
- Shape pass: `9/9`
- Average score: `3.00`

## Capability Classes

- `cross_video_synthesis`: passed `9/9`, avg score `3.00`, failures `-`

## Failures By Class


## Prompt Results

### q003 PASS

- Prompt: Summarize the latest video from each channel I follow.
- Class: `cross_video_synthesis`
- Status: `Completed`
- Score: `3`
- Sources: `6`
- Failure: `-`
- Source videos: Claude Code's latest update is really cool (when it works...) | Claude Cowork: a small taste of AGI | OpenAI just dropped their Cursor killer | This model is kind of a disaster. | Vercel Finally Caught Up | What happened to me?

#### Answer

Retrieved evidence for: Summarize the latest video from each channel I follow.

The answer model is unavailable, so this fallback lists the highest-ranked saved excerpts instead of synthesizing beyond them.

1. This model is kind of a disaster. - Theo - t3․gg: ing the company's reputation. OpenAI models (like 5.4) are contrasted favorably, demonstrating better self-awareness about knowledge cutoffs, web search utilization, and consistency across tasks. Overview This video provides an extensive, critical review of Anthropic's newly released Claude Opus 4.7 model. While acknowledging that Opus 4.7 shows improvements in benchmark scores, instruction following, vision, and mem... [1]
2. What happened to me? - Theo - t3․gg: this can change the same way it changed here. If I start doing more of these types of videos and they perform better than expected, I'll lean more into this. For example, the logging video, we screwed up the export initially and the first version that went up was too short and had most of the content missing. So, we had to re-upload it, which destroys the video performance because a lot of people already saw it, so t... [2]
3. OpenAI just dropped their Cursor killer - Theo - t3․gg: ees for parallel work on the same project, cloud environments, automations (cron-like scheduled prompts), MCP servers/skills integration, and multi-project management. The speaker finds this represents a shift from "commanding code editors via AI" to "orchestrating agents that control code for us," making terminal-based UIs feel obsolete for real coding work. Overview The video provides an in-depth, hands-on review o... [3]
4. Claude Code's latest update is really cool (when it works...) - Theo - t3․gg: ync sub-agent architecture allows the main agent to spin up background tasks that run in parallel without blocking—described as similar to React's Suspense pattern for blocking vs. non-blocking operations. The video documents numerous frustrations: high API costs ($1.56 wasted on a failed task, ~$5+ spent across the session), broken features (the `/rename` command doesn't exist despite being announced), and CLI UX is... [4]
5. Claude Cowork: a small taste of AGI - Theo - t3․gg: ore capable, though potentially riskier. The product represents a step toward AGI by allowing AI to do actual work (moving files, controlling browsers) rather than just generating text responses. Overview This video provides a detailed hands-on review of Anthropic's newly released "Claude Co-work" product, a desktop application designed to bring Claude Code capabilities to non-technical users. The creator, who has ex... [5]
6. Vercel Finally Caught Up - Theo - t3․gg: r" called Bot ID, and an AI gateway. Active CPU billing dramatically narrows the cost gap between Vercel and Cloudflare for long-running, low-CPU requests (like AI inference streaming), bringing the difference from potentially ~100x down to roughly ~2x, while preserving Node compatibility and faster CPUs. Vercel Sandbox allows safe execution of untrusted/AI-generated code via an SDK, competing with Cloudflare's conta... [6]

### q033 PASS

- Prompt: How has the conversation around this topic evolved across my library?
- Class: `cross_video_synthesis`
- Status: `Completed`
- Score: `3`
- Sources: `12`
- Failure: `-`
- Source videos: I don’t really use libraries anymore | It’s time to embrace the AI | Open source is dying | The painful truth about startups (my story) | What happened to me? | What is Theo's Worst Take? | You don't want to be a manager.

#### Answer

Retrieved evidence for: How has the conversation around this topic evolved across my library?

The answer model is unavailable, so this fallback lists the highest-ranked saved excerpts instead of synthesizing beyond them.

1. The painful truth about startups (my story) - Theo - t3․gg: conversations. I learned so much from those developers and I would be the fun, excited guy coming in with the cool new things and they'd be the realistic people showing me how it would or more importantly wouldn't work based on reality. I loved those conversations and getting to just talk about things without worrying about what level someone is, what disclosures they have, what they are and aren't allowed to know, j... [1]
2. I don’t really use libraries anymore - Theo - t3․gg / Key Points: level, increasing after a Christmas slump `leftpad` has weird spikes (people download it as a meme), but overall downloads are going up over time This is counterintuitive—while the need to install these has decreased (you can vibe code alternatives), downloads are increasing because more people are building things with AI assistance and may not know better. The speaker notes `leftpad` functionality is now built into ... [2]
3. I don’t really use libraries anymore - Theo - t3․gg: be very very helpful in fact let's ask it in T3 chat to do the same thing here it's actually finding some useful stuff especially comparing and contrasting my global cloudmomd with the internal one like this is a useful thing and now it's asking questions about which things I want to keep track of that's great this is again valuable this could have been a library he could have written a library that I would run that ... [3]
4. It’s time to embrace the AI - Theo - t3․gg: lie. There's plenty of things I can't trust an LLM with. No LLM has any of access to prod here. But I've been first responder on an incident and fed 40. not 04 mini, not a smarter reasoning model, just bog standard 40 log transcripts and watched it in seconds spot LVM metadata corruption issues on a host we've been complaining about for months. Am I better than an LLM agent at interrogating open source logs and honey... [4]
5. Open source is dying - Theo - t3․gg: we all are nerdy about and care about. I bring this up because there's a couple things that we just experience in life differently because of that. The one I'm imagining right now, and I'm sure a lot of y'all are this one's in chat if you can relate. I used to get a lot of texts from family members, random friends in high school and just people in my life asking random [ __ ] about computers. Anything from, "Can you ... [5]
6. You don't want to be a manager. - Theo - t3․gg: the right things. So I hire based on that. I speak based on that. I mentor based on that. I ship based on that. I do everything based on that. I want to build alignment with the people around me. But if everything I just said sounds terrible, stick to being an IC. I know senior and principal engineers that have not established these skills that have not built this solution to these types of problems that still get wa... [6]
7. I don’t really use libraries anymore - Theo - t3․gg / TL;DR: AI tools are fundamentally changing the calculus of when to use external libraries versus implementing solutions yourself, making it easier to "vibe code" alternatives. The speaker is actively removing libraries from projects when they cause problems, finding it often easier to rewrite functionality than fight with problematic dependencies. Libraries fall into categories: those beyond your knowledge (beginner-level p... [7]
8. I don’t really use libraries anymore - Theo - t3․gg: TL;DR AI tools are fundamentally changing the calculus of when to use external libraries versus implementing solutions yourself, making it easier to "vibe code" alternatives. The speaker is actively removing libraries from projects when they cause problems, finding it often easier to rewrite functionality than fight with problematic dependencies. Libraries fall into categories: those beyond your knowledge (beginner-l... [8]
9. What happened to me? - Theo - t3․gg: There's a comment I've been seeing a lot lately and I wanted to take the time to address it. He's usually in the format of something like, "Man, I missed the old Theo videos. I really liked when Theo would talk about tech and new frameworks and TypeScript, and now all he does is shill AI stuff that he makes money off of." I have a lot of thoughts about this. My first one is that when I look at my channel, sure, there... [9]
10. What is Theo's Worst Take? - Theo - t3․gg / Overview: This brief exchange involves a discussion about identifying the speaker's worst take or opinion. The conversation touches on the speaker's self-assessment of their takes, a past controversial statement, and a specific critique of a storybook item. The dialogue ends with one speaker conceding a point about the storybook's utility. [10]
11. I don’t really use libraries anymore - Theo - t3․gg / Key Points: sync engine himself ("Theo's awful sync engine" for T3 chat), leading him to eventually adopt Convex instead Convex is cited as an example of a dependency that's easier for AI tools to work with because it lives in the codebase as source code. How AI Changes the Dependency Calculus The traditional math for library adoption was: **problem difficulty × how badly you need it × risk of adoption**. This has shifted: **Ris... [11]
12. I don’t really use libraries anymore - Theo - t3․gg / Key Points: for understanding different library types: **Libraries beyond your knowledge**: These are used by people who don't know how to solve the problem themselves. Examples include `is-odd` (literally one line of code) and `leftpad`. The argument against these is that users are outsourcing competency and taking on supply chain risks without understanding them. **Libraries for tedious reimplementation**: Even capable develop... [12]

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

1. Vercel Finally Caught Up - Theo - t3․gg / Key Points: marginal [1]
2. How I Built T3 Chat in 5 Days - Theo - t3․gg: rabbit they make code review way easier by doing a first pass on your PRS and leaving a bunch of useful feedback summarizing drawing diagrams and so much more this is a real poll request where we're no longer allowing people to upload exe files without paying long story go check out my pirate software video if you want to know more about that but here's what code rabbit did summarize the poll request giving a bunch..... [2]
3. Getting emotional over a million checkboxes - Theo - t3․gg: going to post a video unless he's doing something else similarly groundbreaking and I want all of y'all to sub to him as well I'm going to go put his channel Link in the description now because if my view count on this video is higher than his sub count I'm disappointed in y'all because this type of genuinely novel approach to building cool unique things on the web and then sharing it is something that we absolutely.... [3]
4. Which browser should you use right now? - Theo - t3․gg: used. Can't even fit the blog item on the page at the same zoom level. But here's what I wanted. I had done this post in June. What are your biggest frustrations with T3 chat right now? And I wanted to collect all of this data. So, let's do it. Summarize all of the replies to this post. Make sure you check all 500 plus of them. Seems like it worked, right? Here's a fun thing. All of those were ones that are in the fi... [4]
5. Delete your CLAUDE.md (and your AGENT.md too) - Theo - t3․gg: app/ts or tsx to find all of the files there. Did the same for convex. Did the same for general source. Found the convex schema. It found the app routes. Found the vcon config ts config. It just read all of these things. And then it after reading all of that concluded has a good understanding of the codebase and it wrote this. But remember what it wrote is based on things that it already was able to find. In fact, it... [5]
6. Which browser should you use right now? - Theo - t3․gg: I recorded that one. For the most part, the video is more relevant than ever. But if you're wondering why I didn't mention that Atlassian bought Browser Company, it's cuz I filmed that video 2 days before Atlassian bought Browser Company. So yeah, take that as you will. Everything else I say is still very true. Speaking of not caring, data Surf is vaporware. They have been teasing it forever on a wait list. I've neve... [6]

### q042 PASS

- Prompt: Summarize all videos that mention transcripts.
- Class: `cross_video_synthesis`
- Status: `Completed`
- Score: `3`
- Sources: `6`
- Failure: `-`
- Source videos: Delete your CLAUDE.md (and your AGENT.md too) | Getting emotional over a million checkboxes | How I Built T3 Chat in 5 Days | What’s a Hard Fork? | Which browser should you use right now?

#### Answer

Retrieved evidence for: Summarize all videos that mention transcripts.

The answer model is unavailable, so this fallback lists the highest-ranked saved excerpts instead of synthesizing beyond them.

1. How I Built T3 Chat in 5 Days - Theo - t3․gg: rabbit they make code review way easier by doing a first pass on your PRS and leaving a bunch of useful feedback summarizing drawing diagrams and so much more this is a real poll request where we're no longer allowing people to upload exe files without paying long story go check out my pirate software video if you want to know more about that but here's what code rabbit did summarize the poll request giving a bunch..... [1]
2. What’s a Hard Fork? - Hard Fork / At a glance: The video title asks "What’s a Hard Fork?", but the transcript contains no substantive content on this topic. The provided transcript is solely an automated speech recognition (ASR) system metadata note. The text indicates it originated from a local OpenAI-compatible ASR endpoint, not from official RSS show notes. [2]
3. Getting emotional over a million checkboxes - Theo - t3․gg: going to post a video unless he's doing something else similarly groundbreaking and I want all of y'all to sub to him as well I'm going to go put his channel Link in the description now because if my view count on this video is higher than his sub count I'm disappointed in y'all because this type of genuinely novel approach to building cool unique things on the web and then sharing it is something that we absolutely.... [3]
4. Which browser should you use right now? - Theo - t3․gg: used. Can't even fit the blog item on the page at the same zoom level. But here's what I wanted. I had done this post in June. What are your biggest frustrations with T3 chat right now? And I wanted to collect all of this data. So, let's do it. Summarize all of the replies to this post. Make sure you check all 500 plus of them. Seems like it worked, right? Here's a fun thing. All of those were ones that are in the fi... [4]
5. Delete your CLAUDE.md (and your AGENT.md too) - Theo - t3․gg: app/ts or tsx to find all of the files there. Did the same for convex. Did the same for general source. Found the convex schema. It found the app routes. Found the vcon config ts config. It just read all of these things. And then it after reading all of that concluded has a good understanding of the codebase and it wrote this. But remember what it wrote is based on things that it already was able to find. In fact, it... [5]
6. Which browser should you use right now? - Theo - t3․gg: I recorded that one. For the most part, the video is more relevant than ever. But if you're wondering why I didn't mention that Atlassian bought Browser Company, it's cuz I filmed that video 2 days before Atlassian bought Browser Company. So yeah, take that as you will. Everything else I say is still very true. Speaking of not caring, data Surf is vaporware. They have been teasing it forever on a wait list. I've neve... [6]

### q043 PASS

- Prompt: Summarize all videos that mention highlights.
- Class: `cross_video_synthesis`
- Status: `Completed`
- Score: `3`
- Sources: `9`
- Failure: `-`
- Source videos: Delete your CLAUDE.md (and your AGENT.md too) | Getting emotional over a million checkboxes | Grok 4 just dropped, it’s the best model right now (yes really) | How I Built T3 Chat in 5 Days | It’s time to embrace the AI | Vercel Finally Caught Up | Which browser should you use right now?

#### Answer

Retrieved evidence for: Summarize all videos that mention highlights.

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

1. Hacking LightHouse Scores - Theo - t3․gg / Key Points: The Context and Purpose of Lighthouse Lighthouse scores are used by product managers, marketing teams, and developers to evaluate frameworks and shame poorly performing sites (with Angular frequently targeted). Despite the focus on scores, the video argues that Lighthouse's greatest value is sparking conversations about performance and accessibility—even if imperfect, it has encouraged developers to build better, mor... [1]
2. How I Built T3 Chat in 5 Days - Theo - t3․gg: rabbit they make code review way easier by doing a first pass on your PRS and leaving a bunch of useful feedback summarizing drawing diagrams and so much more this is a real poll request where we're no longer allowing people to upload exe files without paying long story go check out my pirate software video if you want to know more about that but here's what code rabbit did summarize the poll request giving a bunch..... [2]
3. Hacking LightHouse Scores - Theo - t3․gg: very large and complex web performance puzzle and without field data I'm not sure any of this matters anyways couldn't agree more so let's take a look at how to hack these scores tldr show the smallest amount of LCP qualifying content on load to boost the FCP and LCP scores until the lighthouse tests have likely finished I've seen this before pages that will delay a big paint until they think Lighthouse is done so th... [3]
4. Getting emotional over a million checkboxes - Theo - t3․gg: going to post a video unless he's doing something else similarly groundbreaking and I want all of y'all to sub to him as well I'm going to go put his channel Link in the description now because if my view count on this video is higher than his sub count I'm disappointed in y'all because this type of genuinely novel approach to building cool unique things on the web and then sharing it is something that we absolutely.... [4]
5. Hacking LightHouse Scores - Theo - t3․gg: contribute to the final score you can play around with the sliders on the lighthouse scoring calculator interesting I know they had that yeah there's a calculator so you can see as these things move how much does it matter so if everything else was great but the FCP took 6 seconds so it took six seconds for the page to to show anything but everything else was fast you're still getting a 90 that feels kind of shitty t... [5]
6. Which browser should you use right now? - Theo - t3․gg: used. Can't even fit the blog item on the page at the same zoom level. But here's what I wanted. I had done this post in June. What are your biggest frustrations with T3 chat right now? And I wanted to collect all of this data. So, let's do it. Summarize all of the replies to this post. Make sure you check all 500 plus of them. Seems like it worked, right? Here's a fun thing. All of those were ones that are in the fi... [6]
7. Hacking LightHouse Scores - Theo - t3․gg / Overview: This video explores Google Lighthouse performance scoring, examining how scores are calculated, their real-world relevance, and whether they can be manipulated. Hosted by a developer discussing a blog post by Salma (sponsored by Sentry), the content systematically breaks down each Lighthouse metric, demonstrates multiple hacks to artificially inflate scores, and argues that Lighthouse is a useful but rough guide that... [7]
8. Delete your CLAUDE.md (and your AGENT.md too) - Theo - t3․gg: app/ts or tsx to find all of the files there. Did the same for convex. Did the same for general source. Found the convex schema. It found the app routes. Found the vcon config ts config. It just read all of these things. And then it after reading all of that concluded has a good understanding of the codebase and it wrote this. But remember what it wrote is based on things that it already was able to find. In fact, it... [8]
9. Which browser should you use right now? - Theo - t3․gg: I recorded that one. For the most part, the video is more relevant than ever. But if you're wondering why I didn't mention that Atlassian bought Browser Company, it's cuz I filmed that video 2 days before Atlassian bought Browser Company. So yeah, take that as you will. Everything else I say is still very true. Speaking of not caring, data Surf is vaporware. They have been teasing it forever on a wait list. I've neve... [9]
10. Hacking LightHouse Scores - Theo - t3․gg: to go use the app means that this number is meanless but also since these things are being indexed they don't have to care there are likely many other situations where apps serve user generated content and you might be unable to control the LCP element entirely particularly regarding images images in video are the bane of Lighthouse existence it's so bad for example if you can control the sizes of all images on your ... [10]
11. Hacking LightHouse Scores - Theo - t3․gg / Key Points: user input responses **Cumulative Layout Shift (CLS)**: 25% weight — measures unexpected visual shifts during page load **Largest Contentful Paint (LCP)**: 25% weight — marks when main content has likely loaded **First Contentful Paint (FCP)**: 10% weight — first point where users see anything on screen **Speed Index (SI)**: 10% weight — measures how quickly content is visually displayed during page load Thresholds f... [11]
12. Hacking LightHouse Scores - Theo - t3․gg: our users we feel like the fastest website in the world but to other users that might be on mobile sites trying to make a quick like thing in their Bank the things that they're going to be looking for here are going to be entirely different so that's an important piece to consider as we go through this is that these metrics even if they're bad might not actually show you what the experiences like for those users our ... [12]

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
2. What’s a Hard Fork? - Hard Fork: Podcast ASR smoke transcript. This text came from the local OpenAI-compatible ASR endpoint, not from RSS show notes. [2]
3. What’s a Hard Fork? - Hard Fork / Key Points: Transcript Metadata**: The only content in the transcript is a procedural note indicating it is a "smoke transcript" generated by a local OpenAI-compatible ASR endpoint, explicitly stating it did not come from RSS show notes. No definitions, examples, or explanations of a "hard fork" are present. [3]
4. What’s a Hard Fork? - Hard Fork / Takeaways: The intended educational content regarding what a hard fork is could not be summarized, as the transcript contains only ASR metadata and no actual discussion. [4]
5. Anthropic’s Cybersecurity Shock Wave + Ronan Farrow and Andrew Marantz on Their Sam Altman Investigation + One Good Thing - Hard Fork / Key Points: senior executives talking about succession at OpenAI. Former public company CEOs (Instacart, Nextdoor, Slack) have been brought in as top lieutenants, introducing "sharp and pointy elbows" and professionalizing influences to counter the "JV board" Altman previously stacked. **The Broader Systemic Issue**: The reporters argue that while individual integrity matters, the core issue is the lack of regulatory guardrails ... [5]
6. Anthropic’s Cybersecurity Shock Wave + Ronan Farrow and Andrew Marantz on Their Sam Altman Investigation + One Good Thing - Hard Fork / Takeaways: The gap between frontier AI capabilities and public access has reopened, meaning powerful, potentially dangerous models now exist in private hands without public oversight or regulatory supervision. Basic cybersecurity hygiene (password managers, MFA, unique passwords) is a critical immediate action for individuals as AI models dramatically lower the barrier for finding and exploiting software vulnerabilities. When e... [6]
7. Anthropic’s Cybersecurity Shock Wave + Ronan Farrow and Andrew Marantz on Their Sam Altman Investigation + One Good Thing - Hard Fork: allegations that he lies repeatedly about things big and small. Well, one of my favorites was when you quote him telling you that he wears a gray sweater every day to avoid decision fatigue. And then he shows up for a his next interview in a green sweater. That felt like a really satisfying detail. That was just for you, Casey. I was wondering if you were going to catch that. I appreciate that eye for fashion that yo... [7]
8. A.I. Backlash Turns Violent + Kara Swisher on Healthmaxxing + The Zuck Bot Is Coming - Hard Fork / Key Points: to want oversight. Kara Swisher on "Healthmaxxing" and Longevity **Kara Swisher Wants to Live Forever**: Swisher’s new CNN docu-series explores the tech elite's obsession with longevity and biohacking. She describes the title as tongue-in-cheek; her goal is to separate legitimate health advancements from narcissistic wellness grifts. **Experiments and Stunts**: Swisher tried various trendy longevity treatments, inclu... [8]
9. A.I. Backlash Turns Violent + Kara Swisher on Healthmaxxing + The Zuck Bot Is Coming - Hard Fork / At a glance: Recent weeks have seen anti-AI sentiment escalate into violence, including a Molotov cocktail attack on Sam Altman's home and a shooting at an Indiana city councilman's house over a data center vote. Public trust in AI and the government's ability to regulate it is plummeting, driven by economic fears, elite-driven deployment, and AI companies actively opposing accountability measures. Data center construction is fac... [9]
10. A.I. Backlash Turns Violent + Kara Swisher on Healthmaxxing + The Zuck Bot Is Coming - Hard Fork / Key Points: AI Backlash Turns Violent **Attempted attack on Sam Altman**: A 20-year-old man threw a Molotov cocktail at the gate of Sam Altman's San Francisco home. The suspect possessed a document outlining anti-AI views and a list of names and addresses of other AI executives, investors, and board members. He then headed toward OpenAI headquarters. Fortunately, no one was hurt. **Shooting in Indiana**: Indianapolis City Counci... [10]
11. A.I. Backlash Turns Violent + Kara Swisher on Healthmaxxing + The Zuck Bot Is Coming - Hard Fork: One of them is a world of extreme acceleration in AI capabilities during the Trump term, right? Before 2028. And in that world, it really matters to have good relationships with Republican lawmakers and the White House. There's another world in which they are having to plan for a new president in 2029. And maybe that's a Democrat, maybe it's a Republican, but like maybe this stuff all takes until 2029 or so to get re... [11]
12. Anthropic’s Cybersecurity Shock Wave + Ronan Farrow and Andrew Marantz on Their Sam Altman Investigation + One Good Thing - Hard Fork: build this frontier even though it's dangerous and we're going to guide it to this safer place. But, you know, you did build the thing in the first place. So, I just like reminding people of that tension because it is not actually inevitable that we build these systems and yet we do often act as if that were the case. Yeah. Last thing, a lot of the people I know who are plugged into the cybersecurity world are being ... [12]

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

