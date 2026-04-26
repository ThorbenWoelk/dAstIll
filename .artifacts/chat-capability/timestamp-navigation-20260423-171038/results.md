# Chat Capability Sweep Results

- Generated: `2026-04-23T15:11:19.805259+00:00`
- Base URL: `http://localhost:3544`
- Dataset: `/Users/thorben.woelk/repos/dAstIll/backend/tests/data/chat_capability_prompts.json`
- Prompt count: `8`

## Summary

- Passed prompts: `1/8`
- Answerability pass: `8/8`
- Grounding pass: `1/8`
- Shape pass: `7/8`
- Average score: `2.00`

## Capability Classes

- `timestamp_navigation`: passed `1/8`, avg score `2.00`, failures `no_sources`

## Failures By Class

- `no_sources`: q072, q075, q076, q077, q078, q079, q080

## Prompt Results

### q072 FAIL

- Prompt: What parts of this video seem most worth revisiting?
- Class: `timestamp_navigation`
- Status: `Completed`
- Score: `1`
- Sources: `12`
- Failure: `no_sources`
- Source videos: A breakdown of style solutions for 2025 | I hate that this is still happening | I'm so f***ing tired of Obsidian. | It's not just you (Claude did get dumber) | It’s time to embrace the AI | JavaScript Frameworks in 2025 | Open source is dying | Vibe coding is already dead
- Notes: timestamp-oriented answer did not surface timestamp information or a timing caveat | timestamp-navigation answer did not identify a section or time

#### Answer

Retrieved evidence for: +{Open source is dead now?} What parts of this video seem most worth revisiting?

The answer model is unavailable, so this fallback lists the highest-ranked saved excerpts instead of synthesizing beyond them.

1. JavaScript Frameworks in 2025 - Theo - t3․gg: do to make the run and better what I saw was all of this complexity in chaos being reduced to a much simpler model where is it down here the isomorphic spa the complexity here has been reduced a ton at the cost of having to understand this part the reality is that I already understood this part because I'm a backend Dev I know how these parts work but if you are a web dev that has mostly ignored the relationship betw... [1]
2. Open source is dying - Theo - t3․gg / Overview: The video presents a comprehensive examination of how AI is negatively impacting the open source ecosystem. The speaker, an experienced open source maintainer and creator of T3 Code, shares firsthand experiences managing a new project that received 150 PRs in just 5 days. The discussion covers four main problem areas: PR spam and quality degradation, increasingly hostile and confused users, GitHub's inadequate platfo... [2]
3. Open source is dying - Theo - t3․gg: escalate because you almost certainly have to use AI to actually scan these PRs. It has a lot of config, enough config that I'm almost certain that this project was vibe coded, but yeah, you get the idea. There are a lot of solutions being made to try and fix these problems. Some of them are going to make it harder for new maintainers to break out, which sucks because we might just have our current maintainers until ... [3]
4. Open source is dying - Theo - t3․gg: put so much effort into killing Hacktoberfest. I think this video of mine, don't contribute to open source, is one of the best videos I ever filmed. Not sure who that blonde guy with the mustache is though. Seriously though, like that video, I have been told by so many maintainers how thankful they are for this video more than almost anything I've done as a developer and journalist, YouTuber, whatever you want to cal... [4]
5. It’s time to embrace the AI - Theo - t3․gg: editor. If he could automate more of that work and his editing tools made those parts easier, he could spend more time on the fun things like the fancy start of the videos. He could work with more people and get more done. Automating the frustrating parts of his job might mean he can take someone else's job, but more importantly, it means he can spend time on the parts he finds fun that also help make my videos bette... [5]
6. Vibe coding is already dead - Theo - t3․gg: get this for a long ass time, but this has become a huge part of my life as a content creator. People seem to think that if you have a video that performs surprisingly well as a YouTuber, Instagram, whatever you're on, that what you should do next is the same topic again. It makes a lot of sense. If I mostly talk about I don't know React and I do one video about spelt instead and that spelt video does really well. Ob... [6]
7. I hate that this is still happening - Theo - t3․gg: use to make them is very different from the tech I started with. The best thing to make your first video with is the things you already have. You shouldn't buy a bunch of new stuff to inspire you to make the first video. You should do it despite not having the right equipment. And once you get good at it, you'll figure out what your equipment can and can't do and make changes based on what you know. And this is the r... [7]
8. I'm so f***ing tired of Obsidian. - Theo - t3․gg: Transcript: This video is going to be a little bit different. If you didn't already know this, I run most of my channel through Notion. Everything from our content calendar and when videos come out to my list of topics that I intend to cover to our research to our assignments to our brands to the sponsors, like everything about what makes a specific video a specific video is managed through Notion. Normally, this isn... [8]
9. It's not just you (Claude did get dumber) - Theo - t3․gg: small percentage to some. They can be sued if they lie. So, they aren't lying here. Again, they use whatever language makes it sound as not bad as possible because Anthropic is not interested in transparency. This first issue since it was a small percentage they said that and then this issue as well as the opus issue that we discussed earlier were not small issues as such they were not called that and also here with.... [9]
10. I hate that this is still happening - Theo - t3․gg: up here. Lionus doesn't like open source because he's a god dev. He likes open source because he had to go through this whole process himself as a bad dev becoming a good dev over time. And during his process getting there, he grew a fondness to open source because the closed source systems he was using caused him a lot of problems. He also earned his right to be an [ __ ] because of all of the problems he encountere... [10]
11. A breakdown of style solutions for 2025 - Theo - t3․gg: And I don't fathom how anyone can see it differently. It's just hard for me to comprehend. So, it's happened. Chad Cienne is the perfect thing in the middle here. It is the thing I wanted when I filmed my last video. Most of that video was me just complaining that the solutions in the middle were bad because they didn't take advantage of all the awesome technologies in the other circles. Now we have something in the.... [11]
12. Open source is dying - Theo - t3․gg: to hire because this just makes my life easier. If you see an issue that's really stale that has already been fixed, comment saying, "Hey, are you sure you're on the latest version? I think this PR fixed it. It doesn't happen for me anymore on the latest." These types of things are so goddamn helpful. And once you've done that a bit on the issue side, you can start doing the same on the PR side. And here, Ben Bandit ... [12]

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

### q075 FAIL

- Prompt: Find the section where the speaker explains the core idea.
- Class: `timestamp_navigation`
- Status: `Completed`
- Score: `2`
- Sources: `8`
- Failure: `no_sources`
- Source videos: Are juniors screwed? (Getting a job in a post-AI world) | ChatGPT Atlas Drove Me Insane (it's not just Chrome) | Cursor, Claude Code and Codex all have a BIG problem | Delete your CLAUDE.md (and your AGENT.md too) | The Truth About React Native | The actual reason you can't get a job | gpt-5.4 is really, really good
- Notes: timestamp-oriented answer did not surface timestamp information or a timing caveat

#### Answer

Retrieved evidence for: Find the section where the speaker explains the core idea.

The answer model is unavailable, so this fallback lists the highest-ranked saved excerpts instead of synthesizing beyond them.

1. The actual reason you can't get a job - Theo - t3․gg: just by being there, being accessible, being real, and talking about the things you actually give a [ __ ] about, it's unbelievable. There is no growth hack that is more powerful than talking about the [ __ ] you care about with others who also care about the thing. That will always lead you to success faster than anything else. And anyone trying to say otherwise is trying to sell you some [ __ ] And honestly, this a... [1]
2. The Truth About React Native - Theo - t3․gg / Key Points: ntire applications. The project started as a way for teams to embed UI components into existing apps without requiring dedicated mobile engineers for every feature. At Facebook, this enables "vertical slicing" where product teams (like ads, feed, messages) own their entire stack across platforms, rather than having separate frontend and backend teams. This architectural approach means that finding native code in an a... [2]
3. Cursor, Claude Code and Codex all have a BIG problem - Theo - t3․gg: time. Codebase quality peaks at approximately 6 months; after that, bad patterns spread exponentially while good patterns spread linearly, making early code quality critical for long-term maintainability. The speaker advocates for "sledgehammer development" - aggressively deleting and rewriting problematic code sections rather than trying to fix them incrementally, which is now economically viable with modern AI too.... [3]
4. Delete your CLAUDE.md (and your AGENT.md too) - Theo - t3․gg: ease in task success and only marginally improving performance (4% average) when written by developers. Context files increase agent exploration, testing, and reasoning, resulting in over 20% higher costs; the speaker's own test showed a 25% time penalty (1m11s vs 1m29s) when using a claude.md file. Most information developers put in these files (architecture overviews, command lists, dependencies) is already discove... [4]
5. Are juniors screwed? (Getting a job in a post-AI world) - Theo - t3․gg: s, contributing to issues, and helping others in Discords/GitHub is a powerful way to stand out and get hired Overview This video breaks down the current state of the software engineering job market, which the speaker describes as "weird" with conflicting signals about unemployment and hiring difficulties. The content is structured into three main sections: companies' failures in hiring processes, experienced develop... [5]
6. ChatGPT Atlas Drove Me Insane (it's not just Chrome) - Theo - t3․gg: cross-platform support (especially Windows) extremely difficult. Input events are translated through a complex multi-stage pipeline (NS Event → Web Input Event → potentially re-synthesized NS Event), which the speaker finds horrifying to maintain. Atlas uses Chromium's Mojo IPC system with custom Swift and TypeScript bindings to communicate between the separate processes. The browser handles agent mode by compositing... [6]
7. gpt-5.4 is really, really good - Theo - t3․gg: weakness compared to competitors like Opus and Gemini, requiring extensive prompt engineering to achieve acceptable results. Benchmark performance is strong on SWE-Bench Pro (57.7%) and other tests, though the speaker's private SkateBench V2 shows Gemini 3.1 Pro Preview leading at 97% vs GPT 5.4 High at 82%. 5.4 Pro and X-High variants often underperform compared to standard 5.4 High in practical use, despite higher.... [7]
8. The Truth About React Native - Theo - t3․gg: gned to integrate into existing native apps, not necessarily replace entire apps; companies like Facebook, Microsoft, Amazon, and Sony use it for specific features or products while maintaining native code elsewhere. Approximately 25% of top 100 apps across major App Store categories use React Native, and there are roughly 10x more React Native job listings than Swift UI or Jetpack Compose positions. Meta Quest's sys... [8]

### q076 FAIL

- Prompt: Find the section where the speaker gives an example.
- Class: `timestamp_navigation`
- Status: `Completed`
- Score: `2`
- Sources: `7`
- Failure: `no_sources`
- Source videos: Are juniors screwed? (Getting a job in a post-AI world) | ChatGPT Atlas Drove Me Insane (it's not just Chrome) | Cursor, Claude Code and Codex all have a BIG problem | Delete your CLAUDE.md (and your AGENT.md too) | The Truth About React Native | gpt-5.4 is really, really good
- Notes: timestamp-oriented answer did not surface timestamp information or a timing caveat

#### Answer

Retrieved evidence for: Find the section where the speaker gives an example.

The answer model is unavailable, so this fallback lists the highest-ranked saved excerpts instead of synthesizing beyond them.

1. The Truth About React Native - Theo - t3․gg / Key Points: ntire applications. The project started as a way for teams to embed UI components into existing apps without requiring dedicated mobile engineers for every feature. At Facebook, this enables "vertical slicing" where product teams (like ads, feed, messages) own their entire stack across platforms, rather than having separate frontend and backend teams. This architectural approach means that finding native code in an a... [1]
2. Cursor, Claude Code and Codex all have a BIG problem - Theo - t3․gg: time. Codebase quality peaks at approximately 6 months; after that, bad patterns spread exponentially while good patterns spread linearly, making early code quality critical for long-term maintainability. The speaker advocates for "sledgehammer development" - aggressively deleting and rewriting problematic code sections rather than trying to fix them incrementally, which is now economically viable with modern AI too.... [2]
3. Delete your CLAUDE.md (and your AGENT.md too) - Theo - t3․gg: ease in task success and only marginally improving performance (4% average) when written by developers. Context files increase agent exploration, testing, and reasoning, resulting in over 20% higher costs; the speaker's own test showed a 25% time penalty (1m11s vs 1m29s) when using a claude.md file. Most information developers put in these files (architecture overviews, command lists, dependencies) is already discove... [3]
4. Are juniors screwed? (Getting a job in a post-AI world) - Theo - t3․gg: s, contributing to issues, and helping others in Discords/GitHub is a powerful way to stand out and get hired Overview This video breaks down the current state of the software engineering job market, which the speaker describes as "weird" with conflicting signals about unemployment and hiring difficulties. The content is structured into three main sections: companies' failures in hiring processes, experienced develop... [4]
5. ChatGPT Atlas Drove Me Insane (it's not just Chrome) - Theo - t3․gg: cross-platform support (especially Windows) extremely difficult. Input events are translated through a complex multi-stage pipeline (NS Event → Web Input Event → potentially re-synthesized NS Event), which the speaker finds horrifying to maintain. Atlas uses Chromium's Mojo IPC system with custom Swift and TypeScript bindings to communicate between the separate processes. The browser handles agent mode by compositing... [5]
6. gpt-5.4 is really, really good - Theo - t3․gg: weakness compared to competitors like Opus and Gemini, requiring extensive prompt engineering to achieve acceptable results. Benchmark performance is strong on SWE-Bench Pro (57.7%) and other tests, though the speaker's private SkateBench V2 shows Gemini 3.1 Pro Preview leading at 97% vs GPT 5.4 High at 82%. 5.4 Pro and X-High variants often underperform compared to standard 5.4 High in practical use, despite higher.... [6]
7. The Truth About React Native - Theo - t3․gg: gned to integrate into existing native apps, not necessarily replace entire apps; companies like Facebook, Microsoft, Amazon, and Sony use it for specific features or products while maintaining native code elsewhere. Approximately 25% of top 100 apps across major App Store categories use React Native, and there are roughly 10x more React Native job listings than Swift UI or Jetpack Compose positions. Meta Quest's sys... [7]

### q077 FAIL

- Prompt: Find the section where the speaker changes direction.
- Class: `timestamp_navigation`
- Status: `Completed`
- Score: `2`
- Sources: `8`
- Failure: `no_sources`
- Source videos: Are juniors screwed? (Getting a job in a post-AI world) | ChatGPT Atlas Drove Me Insane (it's not just Chrome) | Cursor, Claude Code and Codex all have a BIG problem | Delete your CLAUDE.md (and your AGENT.md too) | How I cracked an impossible DEF CON challenge | The Truth About React Native | gpt-5.4 is really, really good
- Notes: timestamp-oriented answer did not surface timestamp information or a timing caveat

#### Answer

Retrieved evidence for: Find the section where the speaker changes direction.

The answer model is unavailable, so this fallback lists the highest-ranked saved excerpts instead of synthesizing beyond them.

1. How I cracked an impossible DEF CON challenge - Theo - t3․gg: are a fellow music nerd you know that 44 means the beat is not everything here so in this first one the beat would be this note this a then we have a rest it's an eighth rest so that this would be offbeat and nothing then we have this F which would be on beat then a rest then we have this E flat then we have this a but this a wouldn't be on beat because it's 1 2 3 4 and there's a gap between each one when you're play... [1]
2. The Truth About React Native - Theo - t3․gg / Key Points: ntire applications. The project started as a way for teams to embed UI components into existing apps without requiring dedicated mobile engineers for every feature. At Facebook, this enables "vertical slicing" where product teams (like ads, feed, messages) own their entire stack across platforms, rather than having separate frontend and backend teams. This architectural approach means that finding native code in an a... [2]
3. Cursor, Claude Code and Codex all have a BIG problem - Theo - t3․gg: time. Codebase quality peaks at approximately 6 months; after that, bad patterns spread exponentially while good patterns spread linearly, making early code quality critical for long-term maintainability. The speaker advocates for "sledgehammer development" - aggressively deleting and rewriting problematic code sections rather than trying to fix them incrementally, which is now economically viable with modern AI too.... [3]
4. Delete your CLAUDE.md (and your AGENT.md too) - Theo - t3․gg: ease in task success and only marginally improving performance (4% average) when written by developers. Context files increase agent exploration, testing, and reasoning, resulting in over 20% higher costs; the speaker's own test showed a 25% time penalty (1m11s vs 1m29s) when using a claude.md file. Most information developers put in these files (architecture overviews, command lists, dependencies) is already discove... [4]
5. Are juniors screwed? (Getting a job in a post-AI world) - Theo - t3․gg: s, contributing to issues, and helping others in Discords/GitHub is a powerful way to stand out and get hired Overview This video breaks down the current state of the software engineering job market, which the speaker describes as "weird" with conflicting signals about unemployment and hiring difficulties. The content is structured into three main sections: companies' failures in hiring processes, experienced develop... [5]
6. ChatGPT Atlas Drove Me Insane (it's not just Chrome) - Theo - t3․gg: cross-platform support (especially Windows) extremely difficult. Input events are translated through a complex multi-stage pipeline (NS Event → Web Input Event → potentially re-synthesized NS Event), which the speaker finds horrifying to maintain. Atlas uses Chromium's Mojo IPC system with custom Swift and TypeScript bindings to communicate between the separate processes. The browser handles agent mode by compositing... [6]
7. gpt-5.4 is really, really good - Theo - t3․gg: weakness compared to competitors like Opus and Gemini, requiring extensive prompt engineering to achieve acceptable results. Benchmark performance is strong on SWE-Bench Pro (57.7%) and other tests, though the speaker's private SkateBench V2 shows Gemini 3.1 Pro Preview leading at 97% vs GPT 5.4 High at 82%. 5.4 Pro and X-High variants often underperform compared to standard 5.4 High in practical use, despite higher.... [7]
8. The Truth About React Native - Theo - t3․gg: gned to integrate into existing native apps, not necessarily replace entire apps; companies like Facebook, Microsoft, Amazon, and Sony use it for specific features or products while maintaining native code elsewhere. Approximately 25% of top 100 apps across major App Store categories use React Native, and there are roughly 10x more React Native job listings than Swift UI or Jetpack Compose positions. Meta Quest's sys... [8]

### q078 FAIL

- Prompt: Find the section where the speaker lists tradeoffs.
- Class: `timestamp_navigation`
- Status: `Completed`
- Score: `2`
- Sources: `8`
- Failure: `no_sources`
- Source videos: Are juniors screwed? (Getting a job in a post-AI world) | ChatGPT Atlas Drove Me Insane (it's not just Chrome) | Cursor, Claude Code and Codex all have a BIG problem | Delete your CLAUDE.md (and your AGENT.md too) | Hacking LightHouse Scores | The Truth About React Native | gpt-5.4 is really, really good
- Notes: timestamp-oriented answer did not surface timestamp information or a timing caveat

#### Answer

Retrieved evidence for: Find the section where the speaker lists tradeoffs.

The answer model is unavailable, so this fallback lists the highest-ranked saved excerpts instead of synthesizing beyond them.

1. Hacking LightHouse Scores - Theo - t3․gg: TL;DR Lighthouse scores can be hacked to achieve perfect 100 scores through techniques like deferring content loading, delaying layout shifts, and manipulating LCP elements—often making sites objectively worse for users. Field data from real users (via tools like Sentry) matters far more than lab-based Lighthouse scores; a site with poor Lighthouse scores can still provide an excellent user experience. Lighthouse sco... [1]
2. The Truth About React Native - Theo - t3․gg / Key Points: ntire applications. The project started as a way for teams to embed UI components into existing apps without requiring dedicated mobile engineers for every feature. At Facebook, this enables "vertical slicing" where product teams (like ads, feed, messages) own their entire stack across platforms, rather than having separate frontend and backend teams. This architectural approach means that finding native code in an a... [2]
3. Cursor, Claude Code and Codex all have a BIG problem - Theo - t3․gg: time. Codebase quality peaks at approximately 6 months; after that, bad patterns spread exponentially while good patterns spread linearly, making early code quality critical for long-term maintainability. The speaker advocates for "sledgehammer development" - aggressively deleting and rewriting problematic code sections rather than trying to fix them incrementally, which is now economically viable with modern AI too.... [3]
4. Delete your CLAUDE.md (and your AGENT.md too) - Theo - t3․gg: ease in task success and only marginally improving performance (4% average) when written by developers. Context files increase agent exploration, testing, and reasoning, resulting in over 20% higher costs; the speaker's own test showed a 25% time penalty (1m11s vs 1m29s) when using a claude.md file. Most information developers put in these files (architecture overviews, command lists, dependencies) is already discove... [4]
5. Are juniors screwed? (Getting a job in a post-AI world) - Theo - t3․gg: s, contributing to issues, and helping others in Discords/GitHub is a powerful way to stand out and get hired Overview This video breaks down the current state of the software engineering job market, which the speaker describes as "weird" with conflicting signals about unemployment and hiring difficulties. The content is structured into three main sections: companies' failures in hiring processes, experienced develop... [5]
6. ChatGPT Atlas Drove Me Insane (it's not just Chrome) - Theo - t3․gg: cross-platform support (especially Windows) extremely difficult. Input events are translated through a complex multi-stage pipeline (NS Event → Web Input Event → potentially re-synthesized NS Event), which the speaker finds horrifying to maintain. Atlas uses Chromium's Mojo IPC system with custom Swift and TypeScript bindings to communicate between the separate processes. The browser handles agent mode by compositing... [6]
7. gpt-5.4 is really, really good - Theo - t3․gg: weakness compared to competitors like Opus and Gemini, requiring extensive prompt engineering to achieve acceptable results. Benchmark performance is strong on SWE-Bench Pro (57.7%) and other tests, though the speaker's private SkateBench V2 shows Gemini 3.1 Pro Preview leading at 97% vs GPT 5.4 High at 82%. 5.4 Pro and X-High variants often underperform compared to standard 5.4 High in practical use, despite higher.... [7]
8. The Truth About React Native - Theo - t3․gg: gned to integrate into existing native apps, not necessarily replace entire apps; companies like Facebook, Microsoft, Amazon, and Sony use it for specific features or products while maintaining native code elsewhere. Approximately 25% of top 100 apps across major App Store categories use React Native, and there are roughly 10x more React Native job listings than Swift UI or Jetpack Compose positions. Meta Quest's sys... [8]

### q079 FAIL

- Prompt: Find the section where the speaker talks about implementation details.
- Class: `timestamp_navigation`
- Status: `Completed`
- Score: `2`
- Sources: `7`
- Failure: `no_sources`
- Source videos: Are juniors screwed? (Getting a job in a post-AI world) | ChatGPT Atlas Drove Me Insane (it's not just Chrome) | Cursor, Claude Code and Codex all have a BIG problem | Delete your CLAUDE.md (and your AGENT.md too) | The Truth About React Native | gpt-5.4 is really, really good
- Notes: timestamp-oriented answer did not surface timestamp information or a timing caveat

#### Answer

Retrieved evidence for: Find the section where the speaker talks about implementation details.

The answer model is unavailable, so this fallback lists the highest-ranked saved excerpts instead of synthesizing beyond them.

1. The Truth About React Native - Theo - t3․gg / Key Points: ntire applications. The project started as a way for teams to embed UI components into existing apps without requiring dedicated mobile engineers for every feature. At Facebook, this enables "vertical slicing" where product teams (like ads, feed, messages) own their entire stack across platforms, rather than having separate frontend and backend teams. This architectural approach means that finding native code in an a... [1]
2. Cursor, Claude Code and Codex all have a BIG problem - Theo - t3․gg: time. Codebase quality peaks at approximately 6 months; after that, bad patterns spread exponentially while good patterns spread linearly, making early code quality critical for long-term maintainability. The speaker advocates for "sledgehammer development" - aggressively deleting and rewriting problematic code sections rather than trying to fix them incrementally, which is now economically viable with modern AI too.... [2]
3. Delete your CLAUDE.md (and your AGENT.md too) - Theo - t3․gg: ease in task success and only marginally improving performance (4% average) when written by developers. Context files increase agent exploration, testing, and reasoning, resulting in over 20% higher costs; the speaker's own test showed a 25% time penalty (1m11s vs 1m29s) when using a claude.md file. Most information developers put in these files (architecture overviews, command lists, dependencies) is already discove... [3]
4. Are juniors screwed? (Getting a job in a post-AI world) - Theo - t3․gg: s, contributing to issues, and helping others in Discords/GitHub is a powerful way to stand out and get hired Overview This video breaks down the current state of the software engineering job market, which the speaker describes as "weird" with conflicting signals about unemployment and hiring difficulties. The content is structured into three main sections: companies' failures in hiring processes, experienced develop... [4]
5. ChatGPT Atlas Drove Me Insane (it's not just Chrome) - Theo - t3․gg: cross-platform support (especially Windows) extremely difficult. Input events are translated through a complex multi-stage pipeline (NS Event → Web Input Event → potentially re-synthesized NS Event), which the speaker finds horrifying to maintain. Atlas uses Chromium's Mojo IPC system with custom Swift and TypeScript bindings to communicate between the separate processes. The browser handles agent mode by compositing... [5]
6. gpt-5.4 is really, really good - Theo - t3․gg: weakness compared to competitors like Opus and Gemini, requiring extensive prompt engineering to achieve acceptable results. Benchmark performance is strong on SWE-Bench Pro (57.7%) and other tests, though the speaker's private SkateBench V2 shows Gemini 3.1 Pro Preview leading at 97% vs GPT 5.4 High at 82%. 5.4 Pro and X-High variants often underperform compared to standard 5.4 High in practical use, despite higher.... [6]
7. The Truth About React Native - Theo - t3․gg: gned to integrate into existing native apps, not necessarily replace entire apps; companies like Facebook, Microsoft, Amazon, and Sony use it for specific features or products while maintaining native code elsewhere. Approximately 25% of top 100 apps across major App Store categories use React Native, and there are roughly 10x more React Native job listings than Swift UI or Jetpack Compose positions. Meta Quest's sys... [7]

### q080 FAIL

- Prompt: Find the section where the speaker talks about results or outcomes.
- Class: `timestamp_navigation`
- Status: `Completed`
- Score: `2`
- Sources: `8`
- Failure: `no_sources`
- Source videos: Are juniors screwed? (Getting a job in a post-AI world) | ChatGPT Atlas Drove Me Insane (it's not just Chrome) | Cursor, Claude Code and Codex all have a BIG problem | Delete your CLAUDE.md (and your AGENT.md too) | I can't believe he was right. | The Truth About React Native | gpt-5.4 is really, really good
- Notes: timestamp-oriented answer did not surface timestamp information or a timing caveat

#### Answer

Retrieved evidence for: Find the section where the speaker talks about results or outcomes.

The answer model is unavailable, so this fallback lists the highest-ranked saved excerpts instead of synthesizing beyond them.

1. I can't believe he was right. - Theo - t3․gg: as I do today, even if my relationship with it is very different than it was a year ago. And I recommend that you reflect yourself and give these things a try. Let me know what y'all think and how you're using these tools today. [1]
2. The Truth About React Native - Theo - t3․gg / Key Points: ntire applications. The project started as a way for teams to embed UI components into existing apps without requiring dedicated mobile engineers for every feature. At Facebook, this enables "vertical slicing" where product teams (like ads, feed, messages) own their entire stack across platforms, rather than having separate frontend and backend teams. This architectural approach means that finding native code in an a... [2]
3. Cursor, Claude Code and Codex all have a BIG problem - Theo - t3․gg: time. Codebase quality peaks at approximately 6 months; after that, bad patterns spread exponentially while good patterns spread linearly, making early code quality critical for long-term maintainability. The speaker advocates for "sledgehammer development" - aggressively deleting and rewriting problematic code sections rather than trying to fix them incrementally, which is now economically viable with modern AI too.... [3]
4. Delete your CLAUDE.md (and your AGENT.md too) - Theo - t3․gg: ease in task success and only marginally improving performance (4% average) when written by developers. Context files increase agent exploration, testing, and reasoning, resulting in over 20% higher costs; the speaker's own test showed a 25% time penalty (1m11s vs 1m29s) when using a claude.md file. Most information developers put in these files (architecture overviews, command lists, dependencies) is already discove... [4]
5. Are juniors screwed? (Getting a job in a post-AI world) - Theo - t3․gg: s, contributing to issues, and helping others in Discords/GitHub is a powerful way to stand out and get hired Overview This video breaks down the current state of the software engineering job market, which the speaker describes as "weird" with conflicting signals about unemployment and hiring difficulties. The content is structured into three main sections: companies' failures in hiring processes, experienced develop... [5]
6. ChatGPT Atlas Drove Me Insane (it's not just Chrome) - Theo - t3․gg: cross-platform support (especially Windows) extremely difficult. Input events are translated through a complex multi-stage pipeline (NS Event → Web Input Event → potentially re-synthesized NS Event), which the speaker finds horrifying to maintain. Atlas uses Chromium's Mojo IPC system with custom Swift and TypeScript bindings to communicate between the separate processes. The browser handles agent mode by compositing... [6]
7. gpt-5.4 is really, really good - Theo - t3․gg: weakness compared to competitors like Opus and Gemini, requiring extensive prompt engineering to achieve acceptable results. Benchmark performance is strong on SWE-Bench Pro (57.7%) and other tests, though the speaker's private SkateBench V2 shows Gemini 3.1 Pro Preview leading at 97% vs GPT 5.4 High at 82%. 5.4 Pro and X-High variants often underperform compared to standard 5.4 High in practical use, despite higher.... [7]
8. The Truth About React Native - Theo - t3․gg: gned to integrate into existing native apps, not necessarily replace entire apps; companies like Facebook, Microsoft, Amazon, and Sony use it for specific features or products while maintaining native code elsewhere. Approximately 25% of top 100 apps across major App Store categories use React Native, and there are roughly 10x more React Native job listings than Swift UI or Jetpack Compose positions. Meta Quest's sys... [8]

