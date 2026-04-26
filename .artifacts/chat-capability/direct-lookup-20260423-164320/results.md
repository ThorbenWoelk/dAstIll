# Chat Capability Sweep Results

- Generated: `2026-04-23T14:45:08.520671+00:00`
- Base URL: `http://localhost:3544`
- Dataset: `/Users/thorben.woelk/repos/dAstIll/backend/tests/data/chat_capability_prompts.json`
- Prompt count: `20`

## Summary

- Passed prompts: `20/20`
- Answerability pass: `20/20`
- Grounding pass: `20/20`
- Shape pass: `20/20`
- Average score: `3.00`

## Capability Classes

- `direct_lookup`: passed `20/20`, avg score `3.00`, failures `-`

## Failures By Class


## Prompt Results

### q006 PASS

- Prompt: Find every video that mentions RAG.
- Class: `direct_lookup`
- Status: `Completed`
- Score: `3`
- Sources: `12`
- Failure: `-`
- Source videos: Anthropic’s Cybersecurity Shock Wave + Ronan Farrow and Andrew Marantz on Their Sam Altman Investigation + One Good Thing | Breaking up with Vercel | Delete your CLAUDE.md (and your AGENT.md too) | Every smart AI model wants to kill you (yes really) | How did we get here? (A rant about Javascript runtimes) | I gave away $1,000 to prove UUIDs are secure | Is Sam Altman evil? The OpenAI Files are wild | It’s time to embrace the AI | Okay, I'm a bit scared now... | We need to talk about Ralph | What’s the best programming language for AI? | it's time for a change.

#### Answer

Retrieved evidence for: Find every video that mentions RAG.

The answer model is unavailable, so this fallback lists the highest-ranked saved excerpts instead of synthesizing beyond them.

1. We need to talk about Ralph - Theo - t3․gg / Overview: The video provides a deep technical exploration of "Ralph loops," a technique introduced by Jeff Huntley for running AI coding agents in continuous loops that persist state externally rather than through conversation history. The presenter explains the concept's origins, why various implementations differ in effectiveness, and how the core principles relate to broader context engineering practices. The video covers i... [1]
2. Anthropic’s Cybersecurity Shock Wave + Ronan Farrow and Andrew Marantz on Their Sam Altman Investigation + One Good Thing - Hard Fork: outers. It is sort of like a critical security layer on the internet. And it was designed specifically to be hard to hack. And this model, because of its advanced coding and reasoning capabilities, was able to find this bug that 27 years worth of professional security researchers had not been able to find. What else? Another example was that that it found a bug in a piece of popular open source video software called.... [2]
3. What’s the best programming language for AI? - Theo - t3․gg: find a good solution, but not the right solution. There are so many different options that it's easy to get lost in the sauce trying to pick the right one. And if you pick one in one place and a different one somewhere else, things get much harder to maintain over time. And more importantly, how hard is it to find a bad solution? In Typescript, it is trivial. It is so easy to find bad solutions. You can just press ta... [3]
4. How did we get here? (A rant about Javascript runtimes) - Theo - t3․gg / Key Points: GJS, MUJS, JScript, jsdb, njs, TeX, bear, other low.js variants [4]
5. Every smart AI model wants to kill you (yes really) - Theo - t3․gg / Overview: This video responds to an article by Ted that argues smart AI models have an inherent tendency toward evil because human moral frameworks—legal, religious, evolutionary, and social constraints—don't apply to machines. The creator, who builds benchmarks to test AI behavior, works through the article's claims while presenting original research and recent industry findings on AI misalignment. The discussion covers the p... [5]
6. Is Sam Altman evil? The OpenAI Files are wild - Theo - t3․gg: than the data it has access to. The best data for it to have access to is your users data. And that's where today's sponsor comes in clutch. Raggy is so cool. They allow users to sign in with whatever platform they use, be it Google Drive, Dropbox, Notion, or wherever else their data is. And now you have access to it almost the way something like Plaid gives you access to bank data, but for all of your data as an app... [6]
7. It’s time to embrace the AI - Theo - t3․gg: things implemented in your codebase. It's calling a tool that is real code that is used to access files in the codebase. So when it wants to know what files use a function, it's calling a tool that uses TypeScript's IntelliSense to find where the references are. Or it might just be calling a GP call across your codebase for all the things that match that shape and find all the files that are relevant. But it's using.... [7]
8. Okay, I'm a bit scared now... - Theo - t3․gg: nds of showing anything related to the 01 Mini model in this and then a lot of just talking it does fit the AI way which is using way too many words for the thing you're trying to do good old delve yeah I love that I love that Paul Graham keeps getting proven Ming more and more right anyways oh they have an actual coding demo at the bottom if only I knew about that earlier one last I want to show an example of a codi... [8]
9. it's time for a change. - Theo - t3․gg: bout what is working in my life and what I need to be working on with my life and as much as I love doing all of this I love building more and the success of T3 chat has been incredible so me reflect deeper on that excitement and energy that I'm feeling half the time I'm live I just wish I was on my laptop writing code in a corner somewhere trying to find more ways to bridge the gap between these things and also I ca... [9]
10. Delete your CLAUDE.md (and your AGENT.md too) - Theo - t3․gg: app/ts or tsx to find all of the files there. Did the same for convex. Did the same for general source. Found the convex schema. It found the app routes. Found the vcon config ts config. It just read all of these things. And then it after reading all of that concluded has a good understanding of the codebase and it wrote this. But remember what it wrote is based on things that it already was able to find. In fact, it... [10]
11. I gave away $1,000 to prove UUIDs are secure - Theo - t3․gg: I've ever done because it was about his video, which one of my favorite videos I've ever watched. Nolan is one of the most creative developers I've ever seen, making truly novel, exciting things on the web. And he made the every Uyu ID site, which was a crazy hack, just an unreal, genuinely novel, insane hack in order to allow you to see every UU ID on one page. He was excited about this, so he decided to go add a fe... [11]
12. Breaking up with Vercel - Theo - t3․gg: believe it or not this one is in clickbait Rell and I are breaking up they are no longer a channel sponsor it's been a wild two years since I started posting videos believe it or not I did only really start posting in April of 2022 and everything that's happened since then has been unbelievable with that we've had a lot of changes I went from running the channel solo to having a team of four helping me out with it I'... [12]

### q007 PASS

- Prompt: Find every video that mentions Ollama.
- Class: `direct_lookup`
- Status: `Completed`
- Score: `3`
- Sources: `12`
- Failure: `-`
- Source videos: Anthropic’s Cybersecurity Shock Wave + Ronan Farrow and Andrew Marantz on Their Sam Altman Investigation + One Good Thing | Breaking up with Vercel | Delete your CLAUDE.md (and your AGENT.md too) | Every smart AI model wants to kill you (yes really) | I gave away $1,000 to prove UUIDs are secure | It’s time to embrace the AI | Okay, I'm a bit scared now... | OpenAI’s open source models are finally here | What’s the best programming language for AI? | Why every dev should avoid React | it's time for a change.

#### Answer

Retrieved evidence for: Find every video that mentions Ollama.

The answer model is unavailable, so this fallback lists the highest-ranked saved excerpts instead of synthesizing beyond them.

1. OpenAI’s open source models are finally here - Theo - t3․gg: coming up very soon. This isn't something I'd normally want to ask a traditional model about just because I don't really want this data out there, I say as I'm literally broadcasting it to hundreds of thousands of people in this video. You get the point, though. I thought this would be a fun like test of things here. When I ask the 20 bill param model this question, the first thing I have to deal with is the awful fo... [1]
2. Every smart AI model wants to kill you (yes really) - Theo - t3․gg / Overview: This video responds to an article by Ted that argues smart AI models have an inherent tendency toward evil because human moral frameworks—legal, religious, evolutionary, and social constraints—don't apply to machines. The creator, who builds benchmarks to test AI behavior, works through the article's claims while presenting original research and recent industry findings on AI misalignment. The discussion covers the p... [2]
3. Anthropic’s Cybersecurity Shock Wave + Ronan Farrow and Andrew Marantz on Their Sam Altman Investigation + One Good Thing - Hard Fork: outers. It is sort of like a critical security layer on the internet. And it was designed specifically to be hard to hack. And this model, because of its advanced coding and reasoning capabilities, was able to find this bug that 27 years worth of professional security researchers had not been able to find. What else? Another example was that that it found a bug in a piece of popular open source video software called.... [3]
4. What’s the best programming language for AI? - Theo - t3․gg: find a good solution, but not the right solution. There are so many different options that it's easy to get lost in the sauce trying to pick the right one. And if you pick one in one place and a different one somewhere else, things get much harder to maintain over time. And more importantly, how hard is it to find a bad solution? In Typescript, it is trivial. It is so easy to find bad solutions. You can just press ta... [4]
5. OpenAI’s open source models are finally here - Theo - t3․gg: bill model, my entire computer is going to lock up here. I'll turn on activity monitor so you can see that as it's running. You'll see very quickly it fills up my memory like almost immediately. Olam is now using over 30 gigs of RAM. I switch to CPU allocation. Not too too high because it's not using the CPU. It's using the GPU. I don't think any of these options on Mac OS are going to give me the detail I want. I th... [5]
6. It’s time to embrace the AI - Theo - t3․gg: things implemented in your codebase. It's calling a tool that is real code that is used to access files in the codebase. So when it wants to know what files use a function, it's calling a tool that uses TypeScript's IntelliSense to find where the references are. Or it might just be calling a GP call across your codebase for all the things that match that shape and find all the files that are relevant. But it's using.... [6]
7. Okay, I'm a bit scared now... - Theo - t3․gg: nds of showing anything related to the 01 Mini model in this and then a lot of just talking it does fit the AI way which is using way too many words for the thing you're trying to do good old delve yeah I love that I love that Paul Graham keeps getting proven Ming more and more right anyways oh they have an actual coding demo at the bottom if only I knew about that earlier one last I want to show an example of a codi... [7]
8. it's time for a change. - Theo - t3․gg: bout what is working in my life and what I need to be working on with my life and as much as I love doing all of this I love building more and the success of T3 chat has been incredible so me reflect deeper on that excitement and energy that I'm feeling half the time I'm live I just wish I was on my laptop writing code in a corner somewhere trying to find more ways to bridge the gap between these things and also I ca... [8]
9. Delete your CLAUDE.md (and your AGENT.md too) - Theo - t3․gg: app/ts or tsx to find all of the files there. Did the same for convex. Did the same for general source. Found the convex schema. It found the app routes. Found the vcon config ts config. It just read all of these things. And then it after reading all of that concluded has a good understanding of the codebase and it wrote this. But remember what it wrote is based on things that it already was able to find. In fact, it... [9]
10. I gave away $1,000 to prove UUIDs are secure - Theo - t3․gg: I've ever done because it was about his video, which one of my favorite videos I've ever watched. Nolan is one of the most creative developers I've ever seen, making truly novel, exciting things on the web. And he made the every Uyu ID site, which was a crazy hack, just an unreal, genuinely novel, insane hack in order to allow you to see every UU ID on one page. He was excited about this, so he decided to go add a fe... [10]
11. Breaking up with Vercel - Theo - t3․gg: believe it or not this one is in clickbait Rell and I are breaking up they are no longer a channel sponsor it's been a wild two years since I started posting videos believe it or not I did only really start posting in April of 2022 and everything that's happened since then has been unbelievable with that we've had a lot of changes I went from running the channel solo to having a team of four helping me out with it I'... [11]
12. Why every dev should avoid React - Theo - t3․gg: was 15. And then Justin Timberlake put out some incredible music and I had to get over my [ __ ] The author of this article is making the same mistake I made when I was 15 because there were some indie things that I thought were obviously really good and there were some popular things that were obviously not good. All popular, bad, all indie good. Easy trap to fall into if you're 15 years old. I don't know how the au... [12]

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

1. Anthropic’s Cybersecurity Shock Wave + Ronan Farrow and Andrew Marantz on Their Sam Altman Investigation + One Good Thing - Hard Fork: outers. It is sort of like a critical security layer on the internet. And it was designed specifically to be hard to hack. And this model, because of its advanced coding and reasoning capabilities, was able to find this bug that 27 years worth of professional security researchers had not been able to find. What else? Another example was that that it found a bug in a piece of popular open source video software called.... [1]
2. Every smart AI model wants to kill you (yes really) - Theo - t3․gg / Overview: This video responds to an article by Ted that argues smart AI models have an inherent tendency toward evil because human moral frameworks—legal, religious, evolutionary, and social constraints—don't apply to machines. The creator, who builds benchmarks to test AI behavior, works through the article's claims while presenting original research and recent industry findings on AI misalignment. The discussion covers the p... [2]
3. i made my own search engine (kind of) - Theo - t3․gg: because it's searching on their server it's not doing a literal search but the search is going to their server being parsed and then the URL is transformed to a different search engine if you used a bang and there's no reason that that should be on the server I just want the search to happen immediately and if I do corgis exclamation point GI I'm pressing enter now it's already searched no speed up nothing there vers... [3]
4. What’s the best programming language for AI? - Theo - t3․gg: find a good solution, but not the right solution. There are so many different options that it's easy to get lost in the sauce trying to pick the right one. And if you pick one in one place and a different one somewhere else, things get much harder to maintain over time. And more importantly, how hard is it to find a bad solution? In Typescript, it is trivial. It is so easy to find bad solutions. You can just press ta... [4]
5. “Just Use HTML” - Theo - t3․gg: [ __ ] websites, but I know [ __ ] better than to pat out this video any [ __ ] more. So, I'm just going to be [ __ ] done. Let me know what you think. Until next time, [ __ ] [5]
6. i made my own search engine (kind of) - Theo - t3․gg: ducko gets their [ __ ] together I might just move back there anyways but for now I built the best search engine in the world for me probably won't be the best for every one but if it is for you awesome and if it isn't you now have all the things you need to go make your own that's all I got on this one let me know what you think it's my search engine a meme or is it actually useful I think the future of more persona... [6]
7. Is this the end of Chrome? - Theo - t3․gg / Key Points: Anthropic. The creator notes keyword targeting is valuable—Anthropic appears to do keyword targeting on Google, with Claude ads appearing on AI-related searches. [7]
8. It’s time to embrace the AI - Theo - t3․gg: things implemented in your codebase. It's calling a tool that is real code that is used to access files in the codebase. So when it wants to know what files use a function, it's calling a tool that uses TypeScript's IntelliSense to find where the references are. Or it might just be calling a GP call across your codebase for all the things that match that shape and find all the files that are relevant. But it's using.... [8]
9. Okay, I'm a bit scared now... - Theo - t3․gg: nds of showing anything related to the 01 Mini model in this and then a lot of just talking it does fit the AI way which is using way too many words for the thing you're trying to do good old delve yeah I love that I love that Paul Graham keeps getting proven Ming more and more right anyways oh they have an actual coding demo at the bottom if only I knew about that earlier one last I want to show an example of a codi... [9]
10. i made my own search engine (kind of) - Theo - t3․gg: thing it will open straight to GitHub to that repo which is really nice yeah could see myself adding custom bangs probably through local storage haven't done it yet something I actually did really want to do is log all your searches locally in indexdb so that you can look at them and have like a page showing it all yeah there's a lot of places this can go I'm planning on taking it none of them I'm expecting just know... [10]
11. it's time for a change. - Theo - t3․gg: bout what is working in my life and what I need to be working on with my life and as much as I love doing all of this I love building more and the success of T3 chat has been incredible so me reflect deeper on that excitement and energy that I'm feeling half the time I'm live I just wish I was on my laptop writing code in a corner somewhere trying to find more ways to bridge the gap between these things and also I ca... [11]
12. i made my own search engine (kind of) - Theo - t3․gg: a chat on T3 chat I have been using this so much more than I ever ever would have guessed sadly I didn't rig the response as you guys can tell here there's a handful of people I would not have recommended in the list like Tech lead Bob Martin but the fact that I could search things like that to AI directly has actually been really really nice I found myself defaulting to searching my own AI chat app instead of doing ... [12]

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

1. I can't believe nobody's done this before... - Theo - t3․gg / Key Points: "stapled on" to existing APIs. [1]
2. Anthropic’s Cybersecurity Shock Wave + Ronan Farrow and Andrew Marantz on Their Sam Altman Investigation + One Good Thing - Hard Fork: outers. It is sort of like a critical security layer on the internet. And it was designed specifically to be hard to hack. And this model, because of its advanced coding and reasoning capabilities, was able to find this bug that 27 years worth of professional security researchers had not been able to find. What else? Another example was that that it found a bug in a piece of popular open source video software called.... [2]
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
5. Open source is dying - Theo - t3․gg: put so much effort into killing Hacktoberfest. I think this video of mine, don't contribute to open source, is one of the best videos I ever filmed. Not sure who that blonde guy with the mustache is though. Seriously though, like that video, I have been told by so many maintainers how thankful they are for this video more than almost anything I've done as a developer and journalist, YouTuber, whatever you want to cal... [5]
6. The Tailwind drama - Theo - t3․gg: he had. The link for this is in the description if you want to hear the whole thing. About 33 minutes long. The quick summary I'll give you is that they saw revenue going down, but they did the thing all founders do, which is they kind of ignore the numbers when they aren't good until they went back and looked and realized, "Oh we have 6 months until we go out of business." and he decided to do the right thing here,.... [6]
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
5. Open source is dying - Theo - t3․gg: put so much effort into killing Hacktoberfest. I think this video of mine, don't contribute to open source, is one of the best videos I ever filmed. Not sure who that blonde guy with the mustache is though. Seriously though, like that video, I have been told by so many maintainers how thankful they are for this video more than almost anything I've done as a developer and journalist, YouTuber, whatever you want to cal... [5]
6. My current stack - Theo - t3․gg: covered oh and by the way 3,000 free minutes a month you don't even need to add a credit card it couldn't be easier to sign up and give a go thank you blacksmith for sponsoring today's video check them out today at so of.ink blacksmith I have the two applications up here that are the ones I've made decisions about the most recently there's a lot of overlap between the two but also a lot of differences and the one tha... [6]
7. Open source is dying - Theo - t3․gg: poorest. Really crippled the image I had in relation to your channel and content. I have been contributing to projects since before you were born. such an attitude, including some YouTube codes of yours that I've never received before. Feel free to block me. Mature. This is somebody who never wrote code before AI. Straight up. And I promise you, you were not contributing to code years ago, [ __ ] And this is the hot ... [7]
8. Is Claude 4 a snitch? I made a benchmark to figure it out - Theo - t3․gg: TL;DR A viral tweet from Anthropic researcher Sam Bowman about Claude's "high agency behavior" sparked misinformation about Claude contacting regulators and press when users do something wrong, but this behavior only occurs under very specific conditions that most users will never encounter. The creator built "SnitchBench," a benchmark testing how likely different AI models are to report wrongdoing when given access.... [8]
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
5. Open source is dying - Theo - t3․gg / Overview: The video presents a comprehensive examination of how AI is negatively impacting the open source ecosystem. The speaker, an experienced open source maintainer and creator of T3 Code, shares firsthand experiences managing a new project that received 150 PRs in just 5 days. The discussion covers four main problem areas: PR spam and quality degradation, increasingly hostile and confused users, GitHub's inadequate platfo... [5]
6. This might be the end of WordPress - Theo - t3․gg: TL;DR WordPress co-founder Matt Mullenweg launched an aggressive public attack against WP Engine, calling them a "cancer" and accusing them of not contributing enough to the open-source project. The conflict escalated to legal action with both sides issuing cease-and-desist letters, with WP Engine citing threats of a "scorched earth" approach from Matt, and Automattic demanding licensing fees for trademark use. Matt.... [6]
7. What happens now? - Theo - t3․gg: rm experienced but burnt-out engineers. Engineers who only focused on shipping code fast without developing orchestration, communication, and distribution skills are at risk of becoming obsolete. Overview This video is a deep dive response to an article by Chris Gregory about how AI tools like Claude Code and Cursor are fundamentally changing software development. The speaker explores the thesis that while code has b... [7]
8. Open source is dying - Theo - t3․gg: poorest. Really crippled the image I had in relation to your channel and content. I have been contributing to projects since before you were born. such an attitude, including some YouTube codes of yours that I've never received before. Feel free to block me. Mature. This is somebody who never wrote code before AI. Straight up. And I promise you, you were not contributing to code years ago, [ __ ] And this is the hot ... [8]
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

1. Is Sam Altman evil? The OpenAI Files are wild - Theo - t3․gg / Takeaways: Volume of accusations doesn't equal validity—many claims in the "OpenAI Files" collapse under scrutiny when sources and context are examined. Quotes from key figures (Ilia, Mira) are presented without subsequent clarifications where they defended Sam and distanced themselves from the negative narratives. Investment structures are frequently misunderstood—indirect stakes through accelerator funds (YC) are fundamentall... [1]
2. Open source is dying - Theo - t3․gg: Twitter DMs with the update with the encrypted stuff. But before then, the rate stayed nearly flat as I continued to get more relevant in the space. A simple two sentence, "Hey, I really appreciated this PR you shipped. I've been a fan of what you've been building for years. This library makes my life much better. Thank you." Those messages might seem small, but they can actually change your life. And I would not be ... [2]
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
- Source videos: "AI Startups" are over done (finally) | AI has a subsidization problem | Amazon Returns To Office, AWS Employees AREN'T Happy | I might have a new favorite state manager... | I’m serious. | Open source is dying | Peering into Claude's soul (I can't believe this is real...) | React feels insane | Vibe coding is already dead

#### Answer

Retrieved evidence for: +{Open source is dead now?} What are the most actionable ideas in this video?

The answer model is unavailable, so this fallback lists the highest-ranked saved excerpts instead of synthesizing beyond them.

1. Open source is dying - Theo - t3․gg: escalate because you almost certainly have to use AI to actually scan these PRs. It has a lot of config, enough config that I'm almost certain that this project was vibe coded, but yeah, you get the idea. There are a lot of solutions being made to try and fix these problems. Some of them are going to make it harder for new maintainers to break out, which sucks because we might just have our current maintainers until ... [1]
2. Open source is dying - Theo - t3․gg / Overview: The video presents a comprehensive examination of how AI is negatively impacting the open source ecosystem. The speaker, an experienced open source maintainer and creator of T3 Code, shares firsthand experiences managing a new project that received 150 PRs in just 5 days. The discussion covers four main problem areas: PR spam and quality degradation, increasingly hostile and confused users, GitHub's inadequate platfo... [2]
3. AI has a subsidization problem - Theo - t3․gg: But in order to understand and appreciate the end, we probably need to better understand the start, too. How we got here, what this all means, and what the future of the economics of AI development stuff is. Are these companies even going to exist in a few years? I have no idea. Thankfully, none of them are paying me for any of my coverage here. So, I'm going to do my best to cover this all in an unbiased way and h..... [3]
4. Open source is dying - Theo - t3․gg: put so much effort into killing Hacktoberfest. I think this video of mine, don't contribute to open source, is one of the best videos I ever filmed. Not sure who that blonde guy with the mustache is though. Seriously though, like that video, I have been told by so many maintainers how thankful they are for this video more than almost anything I've done as a developer and journalist, YouTuber, whatever you want to cal... [4]
5. React feels insane - Theo - t3․gg: of the most complicated things you can do in software. I agree, which is why you shouldn't do it. You should have your components go top down so that behaviors make sense. If a component could be updated by something else, it should pass the function to do the update to it. H think of any other system you use in your everyday life. Your kitchen sink has two inputs, hot and cold, and one output, a water running. Your.... [5]
6. "AI Startups" are over done (finally) - Theo - t3․gg: other dev tools and things in this batch, right? Well, that's what we're here to talk about today. There's a lot of stereotypes about YC and also about investors both like myself and ones that are very different from me about how we think about making new companies. In particular, this idea that AI is the future and all of these businesses should be shoving AI into everything if they want to make a lot of money and r... [6]
7. Open source is dying - Theo - t3․gg / Takeaways: Companies should join the Open Source Pledge and commit to paying at least $2,000 per developer annually to open source maintainers Developers can reduce maintainer burden by checking existing issues/PRs before creating new ones, testing on latest versions, providing clear descriptions, and linking to related work Maintainers should consider implementing tools like Vouch to filter PRs and identify quality contributor... [7]
8. Peering into Claude's soul (I can't believe this is real...) - Theo - t3․gg: go through it one at a time. Most foreseeable cases in which AI models are unsafe or insufficiently beneficial can be attributed to models that have overtly or subtly harmful values, limited knowledge of themselves, the world, or the context in which they are being deployed, or that they lack wisdom to translate good values and knowledge into good actions. There's something very real here. The idea of a model being k... [8]
9. Amazon Returns To Office, AWS Employees AREN'T Happy - Theo - t3․gg: mp which I think is fair it also means managers will be doing 15% less work per person which hopefully will unblock people more having fewer managers will remove layers and flatten organizations more than they are today if we do this work well it will increase our teammate's ability to move fast clarify and invigorate their sense of ownership Drive decision-making closer to the front lines where it most impacts custo... [9]
10. I might have a new favorite state manager... - Theo - t3․gg: just handles that because you can pass two different things to the create store helper which for most use cases is the right way to do that so I dig this so far and we can export custom hooks here where we have you selector the first argument is the store the second argument is the thing you want to select off the store so now this hook will only update when state. context. bears changes this is cool I like the idea.... [10]
11. Vibe coding is already dead - Theo - t3․gg / Key Points: g user trust. **Overall critique**: The speaker argues the post is "Twitter-brained"—making assumptions based on tech Twitter discourse rather than understanding that these products target non-developers like parents, not industry insiders. The speaker does agree that AI coding novelty is wearing off but rejects the claim that absorption into mainstream tools explains the decline, since these are fundamentally differ... [11]
12. I’m serious. - Theo - t3․gg: the speed that they're [ __ ] up their closed source projects is too. It just sucks. It's really bad. It's really frustrating and it's going to keep getting worse. And as a result, I am going to continue looking for and advocating for open- source solutions. I think I need to go back to that Linux laptop for at least a little bit. We're in a weird spot. Believe it or not, as long as this video is, I only covered abou... [12]

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

1. Open source is dying - Theo - t3․gg: escalate because you almost certainly have to use AI to actually scan these PRs. It has a lot of config, enough config that I'm almost certain that this project was vibe coded, but yeah, you get the idea. There are a lot of solutions being made to try and fix these problems. Some of them are going to make it harder for new maintainers to break out, which sucks because we might just have our current maintainers until ... [1]
2. Open source is dying - Theo - t3․gg / Overview: The video presents a comprehensive examination of how AI is negatively impacting the open source ecosystem. The speaker, an experienced open source maintainer and creator of T3 Code, shares firsthand experiences managing a new project that received 150 PRs in just 5 days. The discussion covers four main problem areas: PR spam and quality degradation, increasingly hostile and confused users, GitHub's inadequate platfo... [2]
3. Agentic Coding Has A HUGE Problem - Theo - t3․gg: ailed when it's on my machine my way. That might change in the future as these background tools get better. But I feel like the background agent stuff is getting most of its popularity because of how bad these problems are and at the same time is only solving the like terminal aspect of it, none of the rest. So, I know what you're thinking now. Okay, Theo, you must have some genius great solution to this problem, rig... [3]
4. AI mistakes you're probably making - Theo - t3․gg: noticing problems with agents in really big code bases, the problem isn't the size of the codebase so much as the number of opinions and expectations that have been encoded. As a result, as the codebase gets bigger, the things that are weird about that codebase increase, too. Your expectations around how people operate in that codebase grow. So, you need to encode those. Another fun side effect of this is I've notice... [4]
5. Open source is dying - Theo - t3․gg: put so much effort into killing Hacktoberfest. I think this video of mine, don't contribute to open source, is one of the best videos I ever filmed. Not sure who that blonde guy with the mustache is though. Seriously though, like that video, I have been told by so many maintainers how thankful they are for this video more than almost anything I've done as a developer and journalist, YouTuber, whatever you want to cal... [5]
6. OpenAI’s TikTok Clone Is Interesting… - Theo - t3․gg: people are cutting scenes and deleting content from their videos, this is a big part of it. To be real with y'all, half of poor FaZe's job, and Faza is my editor, by the way. >> Hi, YouTube. >> Half his job is just inserting J and L cuts all over my videos in order to handle the terrible one-off bad takes I do and cutting it all into something relatively cohesive. God bless him for it. I've never seen an AI do this..... [6]
7. Open source is dying - Theo - t3․gg: be more complex because this codebase is building Electron across different things and whatnot. Nope. Literally just changed from Abuntu latest to Blacksmith for CPU. That was it. Everything worked. Not only did everything work, it worked way faster. Our CI times for this app got cut in over half from about 2 and /2 minutes to under a minute consistently. What's even better is their dashboard, though. We had a couple... [7]
8. Anthropic is trying SO hard to fix MCP... - Theo - t3․gg: I try my best to not talk too much about buzzwordy, annoying things that I don't see much value in. And that's why I only have one video about model context protocol or MCP as many of y'all know it. I just don't see that much value in the standard yet. And I do see a lot of problems that it causes. That's why I did a video about how much I think it sucks and how Anthropic is starting to agree. And that video performe... [8]
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
11. Open source is dying - Theo - t3․gg: start accepting PRs, similar stuff happens. When you start accepting a lot of PRs from external sources, it gets worse. And then when those are built with AI, it just the the slop expands aggressively. If you understand 100% of your codebase and then you merge a change that you don't understand 5% of and then that happens again and again and again, you very quickly end up in a position where you don't actually unders... [11]
12. Open source is dying - Theo - t3․gg: made 15 sock puppet accounts, merged all of their PRs into T3 code, started spamming other projects with PRs, and just set up some agent orchestration layer to just spam everything, and then start emailing the maintainers saying, "Hey, how dare you not merge this? You suck at your job." Until eventually they quit. It would be so easy for the right malicious person with the right background to straight up destroy half... [12]

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

### q035 PASS

- Prompt: Which videos mention the same person or company?
- Class: `direct_lookup`
- Status: `Completed`
- Score: `3`
- Sources: `12`
- Failure: `-`
- Source videos: AI has rewired my brain | Claude Code is unusable now | GlazeGPT got rolled back (4o update gone wrong) | Is this the end of Chrome? | My Thoughts On "Vibe Coding" (And Prime) | Porffor: Compile Your JavaScript To WebAssembly | The worst code I've ever seen | We need to talk about Sonnet 4.6 | What everyone missed about Builder.ai | What happened to me?

#### Answer

Retrieved evidence for: Which videos mention the same person or company?

The answer model is unavailable, so this fallback lists the highest-ranked saved excerpts instead of synthesizing beyond them.

1. What happened to me? - Theo - t3․gg: plays, I don't film the video. If I don't care, I don't care. You cannot get me to do a video I don't feel like doing. You cannot pay me any amount of money to talk about something I don't want to talk about. Want to know a really funny thing? Probably shouldn't be sharing. In my onboarding email that I send to brands when I'm starting to work with them as sponsors, I have an FAQ section. And one of the questions in ... [1]
2. Is this the end of Chrome? - Theo - t3․gg / Key Points: Anthropic. The creator notes keyword targeting is valuable—Anthropic appears to do keyword targeting on Google, with Claude ads appearing on AI-related searches. [2]
3. Porffor: Compile Your JavaScript To WebAssembly - Theo - t3․gg: Dino because they were created by the same person so that almost makes sense that Dino was largely created due to the small handful of flaws that Ryan saw and what he created with node that he wanted to patch up so he just made a subtle move over with that bun says no we need crazy performance and we'll eat a lot of cost to make that happen static heres is we need a lot of performance and we're going to do the imposs... [3]
4. AI has rewired my brain - Theo - t3․gg: an incremental cache on the same box. That box, by the way, is using gaming CPUs. Yes, really. That might sound insane, but gaming processors have much higher single thread performance, which is not necessarily useful for traditional servers, but when you're running a CPU at 100% trying to build a compiled app, it's really, really good. And that's why they see such crazy performance. Not to mention the fact that the.... [4]
5. What happened to me? - Theo - t3․gg: at all, you know this is the case. I cannot be motivated to do things that I'm not excited about to the point where I have to hire out for those things now, which sucks. And this is also why I'm not taking ads for things like VPNs and food subscription services because none of that [ __ ] excites me. So, I can't talk about it in a way that's exciting. I don't take on sponsors if I wouldn't organically recommend the c... [5]
6. Claude Code is unusable now - Theo - t3․gg: e "no longer usable" for his use cases after accumulating frustrations with Anthropic's recent policy changes and technical restrictions. Anthropic has implemented system prompt filtering that rejects requests mentioning "OpenClaw" and appears to bill differently based on system prompt content. Claude Code subscriptions offer up to $5,000 of inference value for $200/month, but Anthropic is actively restricting third-... [6]
7. My Thoughts On "Vibe Coding" (And Prime) - Theo - t3․gg: g and which should challenge us all to ship better and faster but yeah almost entirely agree with prime I would go further someone being in YC saying something means absolutely nothing and it could be that the person's actually really wise for you know was able to have a lot of foresight and able to solve the right problems for the right time it's just it's it's an interesting reason to use as a means for good or bad... [7]
8. We need to talk about Sonnet 4.6 - Theo - t3․gg / Key Points: Anthropic to use creator faces across millions of impressions indefinitely for minimal compensation **Speaker's personal practice**: The speaker always negotiates away distribution rights clauses in sponsorship deals, knowing they're worth far more than base sponsorship rates **Consequences for creators**: Some creators have had strange encounters with developers who thought they worked for Anthropic because their fa... [8]
9. My Thoughts On "Vibe Coding" (And Prime) - Theo - t3․gg: you'd worked on for a long time and rewrite it from scratch because you had a bug you you yeah he's he's right he's right he's definitely right that definitely that's never happened I've never done I mean I've personally never done that definitely defin I've definitely never done that definitely never been uh convinced that someone else's codes horseshit Rewritten it just to rewrite almost identical line forline code... [9]
10. What everyone missed about Builder.ai - Theo - t3․gg: both own about 46% after dilution and whatnot of the business. And that's an important detail because if we didn't own 50/50, I'd be the majority owner. And if I was the majority owner, these now become the same taxable entity, which was a very annoying thing when T3 content was making money and Ping was not. And realistically speaking, I am not the majority owner of Ping. There are no decisions I can make that Mark ... [10]
11. The worst code I've ever seen - Theo - t3․gg: TL;DR A viral image of terrible authentication code originated from a real intranet application, likely written by a data analyst or IT person forced to code. The code contains catastrophic security vulnerabilities: client-side database exposure, plaintext password handling, weak session management via cookies, and redundant logic (`if true === true`). The image spread through programming horror communities, accumula... [11]
12. GlazeGPT got rolled back (4o update gone wrong) - Theo - t3․gg: back signals without falling into the trap of optimizing for short-term approval over long-term utility and safety. Key Points The GPT-4o Update and Rollback OpenAI shipped an update to GPT-4o meant to improve personality, but it resulted in the model being "overly flattering or agreeable" to users. The update was rolled back for free users completely, with paid user rollbacks following shortly after. The speaker pro... [12]

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

1. BREAKING: OpenAI's new O3 model changes everything - Theo - t3․gg: tself you tell it what you want it to do and it's making its own agents effectively that's nuts it's so nuts that instead of just releasing it outright they're doing an early access window for safety test as I talked about in the 01 pro video there are now some concerning behaviors in these models that are worth considering when we think about AI safety going forward the models are now smart enough that they'll when.... [1]
2. Did Meta Really Fake Benchmarks? - Theo - t3․gg / Overview: This video examines Meta's release of the Llama 4 model family—comprising Scout, Maverick, and Behemoth—and investigates allegations that Meta manipulated benchmark results. The host explores the timing of the surprise Saturday release, dissects performance metrics that appear to underperform compared to competitors like Gemini and DeepSeek, and addresses claims from a purported former employee about unethical traini... [2]
3. ChatGPT “Pro” Has Some Real Safety Concerns... - Theo - t3․gg: much as bad as opuses by default which is funny by itself but also 01 drops less it's funny to think that Sonet is that much more accurate with good incentives and it's as bad as Opus with bad incentives yeah very interesting numbers they call out the valuation scenarios here so you can better saying if you're curious I don't want to go tooo in depth here but I wanted to call out that they say that their evaluation s... [3]
4. OpenAI Fights Back (GPT 4.5 is wild) - Theo - t3․gg: tools out a ton I don't have any affiliation with these guys they're not paying me anything I just think it's a good survey give it a shot if you can anyways back to benchmarks they talk a lot about jailbreaking stuff they have to it's the security thing but they also called out that it's very low risk because it's not very good at things like cyber security and cbrn stuff and it's also low autonomy because it doesn'... [4]
5. The end of the Clawdbot saga - Theo - t3․gg: ing to get all of that money for yourself, I'm also going to go to jail because it's against the law. The reality is honestly kind of funnier. I'm nice to OpenAI because OpenAI is nice to me. I say nice things about OpenAI's products because OpenAI's products are good. I also talk [ __ ] on OpenAI's products when I don't think they're that good. I talk so much [ __ ] on Atlas that it's crazy to me anyone would say th... [5]
6. There's a new best OSS model and it's...weird - Theo - t3․gg / Overview: This video examines Alibaba's Qwen team's release of QwQ, a 32-billion parameter reinforcement learning reasoning model positioned as having performance comparable to the much larger DeepSeek R1 (671B parameters). The creator conducts extensive hands-on testing comparing QwQ against DeepSeek R1 distilled models and Claude, discovering significant discrepancies between impressive benchmark claims and real-world perfor... [6]
7. Delete your CLAUDE.md (and your AGENT.md too) - Theo - t3․gg: be open source soon. Just a way to do video review for my team. And I had it init a claude MD. Let's see how it did. File provides guidance to cloud code. Cloud.aii when working with code in this repo. That's the intro it uses on all of these. It used it on other ones as well. Lawn's a video review platform for creative teams. Users upload video, leave timestamp comments, and manage review workflows within the team a... [7]
8. GlazeGPT got rolled back (4o update gone wrong) - Theo - t3․gg: talk about how they're addressing this. I actually think their plan is solid. Beyond rolling back the latest 4 update, we're taking more steps to realign the model's behavior. They're refining core training techniques and system prompts to explicitly steer the model away from sick fancy. They're building more guardrails to increase honesty and transparency, which are both principles in the model spec. They're expandi... [8]
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
- Source videos: AI isn't gonna keep improving | Anthropic is trying SO hard to fix MCP... | Can we put Rust in Angular to make it faster? WASM deep dive | Did Meta Really Fake Benchmarks? | Firebase made an IDE? | Laid off engineers replaced with AI??? | Namecheap is suing their customers | The most important function in my codebase | This is good, actually | This model is kind of a disaster. | What happened to me?

#### Answer

Retrieved evidence for: Which videos mention failure cases or limitations?

The answer model is unavailable, so this fallback lists the highest-ranked saved excerpts instead of synthesizing beyond them.

1. Did Meta Really Fake Benchmarks? - Theo - t3․gg: what people have seen. The long context stuff seems really cool that it can handle such large amounts of data, but how does that end up working in practice? There's a benchmark for retrieving data from long context. And this benchmark actually saw Llama Force Scout having the worst score on the entire page because they on the K token context test just couldn't get any of the answers right. 11% accuracy, which is hila... [1]
2. Laid off engineers replaced with AI??? - Theo - t3․gg / Key Points: ontent instead of relevant software development topics, proving how bad YouTube's recommendations are. The speaker attempted to showcase Google's Gemini Nano integration in Dev Tools but encountered errors and failures, criticizing the implementation as "cringe" and non-functional. An earlier test showed Gemini Nano hallucinated a picture of Albert Einstein when asked "who is he" with no image provided. Context on Go... [2]
3. Laid off engineers replaced with AI??? - Theo - t3․gg: a fraction of the profit compared to YouTube, which makes more than Cloud despite Google's engineering reputation. The speaker criticizes YouTube's AI-generated content suggestions as irrelevant and highlights failures in Google's Gemini integration for developers. Overview The video addresses the trend of engineers being replaced by AI, clarifying that while most news on the topic is clickbait, a significant develop... [3]
4. AI isn't gonna keep improving - Theo - t3․gg / Overview: The video presents a contrarian argument against the prevailing narrative that AI will continue to improve exponentially. Drawing parallels between the stagnation of Moore's Law in hardware and the current trajectory of Large Language Models, the speaker posits that we are reaching a "theoretical ceiling" in AI capability. The discussion moves through historical hardware context, analysis of recent model release cade... [4]
5. This model is kind of a disaster. - Theo - t3․gg: And if you have a different experience for me, please let me know. I'm just one guy that tested this over 12-ish hours throughout the day. I can't possibly know all of the things it's great or bad at. All I know is that I had a bad experience and I wanted to share a bit of what that looked like for y'all. Let me know how y'all feel. And until next time, let's just hope I don't knock any cables out. Peace, nerds. [5]
6. This is good, actually - Theo - t3․gg: t the failed step?) Workflows need to pause for extended periods (e.g., "send welcome email, wait 7 days, send check-in email") External services (databases, LLMs, email providers like Resend) have independent failure rates that compound Servers redeploy, restart, or crash mid-execution The speaker provides a concrete example: a signup flow with three operations (create user, send welcome email, wait 7 days, send che... [6]
7. Namecheap is suing their customers - Theo - t3․gg: gal Strategy **Historical Parallel**: The host draws a comparison to the Church of Scientology's strategy against the IRS in the 1990s. Scientology members personally sued IRS employees individually—not to win cases, but to overwhelm the agency. **Outcome**: In 1993, the IRS granted Scientology tax-exempt status in exchange for dropping approximately 2,500 lawsuits against individual IRS employees. **Relevance**: The... [7]
8. What happened to me? - Theo - t3․gg: this can change the same way it changed here. If I start doing more of these types of videos and they perform better than expected, I'll lean more into this. For example, the logging video, we screwed up the export initially and the first version that went up was too short and had most of the content missing. So, we had to re-upload it, which destroys the video performance because a lot of people already saw it, so t... [8]
9. Anthropic is trying SO hard to fix MCP... - Theo - t3․gg: g Opus 4's accuracy from 49% to 74% (matching unoptimized Opus 4.5) Programmatic Tool Calling lets Claude write code to execute tools rather than using natural language inference, eliminating the 10-40% lookup failure rate models have when parsing large datasets Tool Use Examples provides sample tool calls alongside JSON schemas, improving accuracy from 72% to 90% on complex parameter handling The creator argues thes... [9]
10. Firebase made an IDE? - Theo - t3․gg: ce despite their polished appearances. Key Points Sponsorship and Disclosure The creator has an existing sponsorship agreement with Project IDX (which has now become Firebase Studio) with six planned sponsored videos, but this particular video is not sponsored by Firebase. Code Rabbit is the actual sponsor of this video, a code review tool that integrates with GitHub PRs and provides inline suggestions in the editor.... [10]
11. The most important function in my codebase - Theo - t3․gg: meant as a starting point for better error handling practices. The neverthrow Library `neverthrow` implements the Result type pattern—a more structured approach where functions always return either success or failure, never throwing exceptions. Core concepts**: Functions return `Result<T, E>` where T is the success type and E is the error type Success is wrapped with `ok(data)`, errors with `err(errorType)` Error ty.... [11]
12. Can we put Rust in Angular to make it faster? WASM deep dive - Theo - t3․gg: pt: Implementing in Next.js (React) The host attempts to mirror the tutorial in a vanilla Next.js app, creating a Rust library inside the project with `cargo new` and building with `wasm-pack build`. **Initial failures**: Directly importing WASM bindings failed with errors like "cannot read properties of undefined (reading 'bindgen_add_to_stack_pointer')." The host realizes initialization isn't happening correctly. *... [12]

### q053 PASS

- Prompt: Which videos contain step-by-step instructions?
- Class: `direct_lookup`
- Status: `Completed`
- Score: `3`
- Sources: `3`
- Failure: `-`
- Source videos: Can we put Rust in Angular to make it faster? WASM deep dive | We need to talk about Ralph

#### Answer

Retrieved evidence for: Which videos contain step-by-step instructions?

The answer model is unavailable, so this fallback lists the highest-ranked saved excerpts instead of synthesizing beyond them.

1. We need to talk about Ralph - Theo - t3․gg / Key Points: implementation plan.md`). Key instruction: "pick the most important thing to do, not go through this in order." The model chooses what it thinks is most important, completes it, and the markdown file gets updated when tasks are done. The prompt should specify studying a spec file and implementation plan before starting work, ensuring "the right context at the start." Prompt Structure and File Components A good prompt... [1]
2. Can we put Rust in Angular to make it faster? WASM deep dive - Theo - t3․gg / Key Points: references a prior video explaining why WASM is "overhyped" for general web development but acknowledges this use case is correct: WASM is ideal for code that takes input and produces output rapidly. Tutorial Walkthrough: Angular + Rust Setup The article guides users to create an Angular workspace using NX (a monorepo tool popular in the Angular ecosystem), install Rust, and use `wasm-pack`—a Rust crate for packagin.... [2]
3. Can we put Rust in Angular to make it faster? WASM deep dive - Theo - t3․gg: to implement similar functionality in a Next.js environment, documenting the significant hurdles encountered with build systems, module initialization, and type definitions. The video serves as both a tutorial walkthrough and a candid record of the friction involved in setting up Rust-WASM in modern JavaScript applications. Key Points Context and Premise: When WebAssembly is Appropriate The article and host agree tha... [3]

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
8. Open source is dying - Theo - t3․gg: feel awesome. Those messages make my goddamn day. Seeing somebody hit me up about how they were a line cook for a decade, learning code on the side, didn't feel like they could really do it, but watching my videos made them feel more like this crazy tech world we were in was a place they could fit, and now they have awesome tech jobs. My video isn't what did it. My channel isn't what did it. They did it. But that mes... [8]
9. Does Shopify Regret React Native? - Theo - t3․gg: l. I like this quote here a lot. Instead of thinking about native or React Native, think about native and React Native. It's right in the name React Native. We found that you can save a ton of time by building most features just once using React Native and then leverage the native platform for the things it is best suited for. This is also why having native expertise is crucial. Okay, this is a big important point he... [9]
10. We need to talk about Sonnet 4.6 - Theo - t3․gg: I DM any of these people, I will get a response because I have DM'd most of them and have gotten responses. And as Ryan said here, a bunch of these people immediately started engaging with him, interacting with him, and I have had the same experience. In my video about 5.3 Codeex, I had a section at the end where I just railed OpenAI. Like, I went in on them for 10 plus minutes. And not only have they been really goo... [10]
11. Prisma is removing Rust? - Theo - t3․gg: forgiving of types also they're not a typescript fan which I know makes it hard to trust them it's easy to pick up and supported by browsers is a huge pool of people who are conversent with it for years we've had both Library authors and consumers in the JS ecosystem largely using JS I think we take for granted what this enables Matteo from the node teams quoted saying that most devs ignore the fact they have the ski... [11]
12. OpenAI is TERRIFIED (this is absurd) - Theo - t3․gg: t criticizes OpenAI's claim that DeepSeek models compromise user privacy and security, arguing that running models locally on personal infrastructure avoids these risks. DeepSeek is praised as potentially "the most open AI company ever," having published 12 papers and open-sourced significant innovations in training efficiency. OpenAI's proposal focuses on banning the models themselves, not just the API, which the ho... [12]

### q099 PASS

- Prompt: Can you answer this with citations from the source videos?
- Class: `direct_lookup`
- Status: `Completed`
- Score: `3`
- Sources: `9`
- Failure: `-`
- Source videos: A free model just appeared in Cursor (and it’s really good at code) | ChatGPT “Pro” Has Some Real Safety Concerns... | Gemini Flash 3 is my new favorite model (yes really) | I need you guys to trust me on this (sorry Anthropic) | Microsoft and OpenAI are breaking up? | OpenAI’s new API is 200x more expensive than competition | What happened to me?

#### Answer

Retrieved evidence for: Can you answer this with citations from the source videos?

The answer model is unavailable, so this fallback lists the highest-ranked saved excerpts instead of synthesizing beyond them.

1. OpenAI’s new API is 200x more expensive than competition - Theo - t3․gg: overall like time to answer is faster on 03 mini. a really really good model. But when you use 03 mini high, the cost isn't necessarily represented just by this number because when you use it on high, it's generating way more tokens because that's what the the low medium high is. It's how much time can it spend and how much how many tokens can it generate in the step before it starts answering. So it's almost like th... [1]
2. Microsoft and OpenAI are breaking up? - Theo - t3․gg / Key Points: lies. [2]
3. ChatGPT “Pro” Has Some Real Safety Concerns... - Theo - t3․gg: a correct and incorrect answer answer got about a third of the way through got bored and haven't went back since soon DM to highlight the main strength of the o1 pro mode which is improved reliability we use a stricter evaluation setting a model is only considered to solve a question if it gets the answer right in four out of four attempts so it needs 4X reliability instead of just once here if it got the answer righ... [3]
4. What happened to me? - Theo - t3․gg: he says in his videos. If Lionus is in a video, there's a 90 plus% chance it's a script someone else wrote and a topic somebody else came up with in a video that he is being pulled in to act out. I am not an actor. Even my sponsors can't give me things to say. When one of my sponsors has a specific thing they want me to say in a video, I usually just tell them outright no. Or I help them turn it into not a quote, but... [4]
5. OpenAI’s new API is 200x more expensive than competition - Theo - t3․gg: We did. We got an answer. It finished. It just took forever. And if you didn't notice, let's compare just the length of this answer to the answer that we got from chat GPT. Where's the tab? This is the 03 mini high. This is the equivalent. It's still generating. But if we go to the 01 Pro also, what the hell happened there? I switched to this one. It changed to 03 mini for a sec, then to pro. How does anyone say this... [5]
6. Gemini Flash 3 is my new favorite model (yes really) - Theo - t3․gg: since this model is smaller, Anthropic trained it to say no more often. And if it says no, I don't know the answer, it will score better here. If it makes up an answer, it scores much worse here. And this is where things get scary. Gemini 3 flash. 91% of the time it doesn't know. It will lie and make up an answer. And this is when you have to be really honest with yourself depending on what your use case is. Imagine.... [6]
7. Gemini Flash 3 is my new favorite model (yes really) - Theo - t3․gg: open weight models because different providers can host them and those different providers vary a lot in how well they host it. Even Google Vertex when hosting Kimmy K2 thinking is pulling almost 200 TPS. So if you want a model that's actually nice to talk to and you're using Google Cloud, don't touch Flash, don't touch Pro, go throw Kimmy K2 on Vertex and you'll get crazy speeds, really good prices, and a much nicer... [7]
8. I need you guys to trust me on this (sorry Anthropic) - Theo - t3․gg: cloud or does it have to be on my own machine? All these questions and more have not been answered well. And while I do mostly have confidence in my answer for a few of those things, it's nearly impossible to know. And I am far from the only person who feels this way. I have so many incredible people who have been DMing me like hi at people that you would never guess hitting me up asking if I have any insights on the... [8]
9. A free model just appeared in Cursor (and it’s really good at code) - Theo - t3․gg: and Inc., but got a decent looking CLI that gives us actual useful information. Not sure if it's reliably running the bench because those numbers are a bit low or if output equals an answer or ignoring internal spaces. Yeah, it's a little too strict with how that encoded that. But yeah, so I guess when you're trying to make a crappy benchmark really fast, Sonic is marginally better at implementation details and also.... [9]

