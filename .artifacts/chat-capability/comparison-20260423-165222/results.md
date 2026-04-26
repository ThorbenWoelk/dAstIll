# Chat Capability Sweep Results

- Generated: `2026-04-23T14:53:26.934557+00:00`
- Base URL: `http://localhost:3544`
- Dataset: `/Users/thorben.woelk/repos/dAstIll/backend/tests/data/chat_capability_prompts.json`
- Prompt count: `11`

## Summary

- Passed prompts: `11/11`
- Answerability pass: `11/11`
- Grounding pass: `11/11`
- Shape pass: `11/11`
- Average score: `3.00`

## Capability Classes

- `comparison`: passed `11/11`, avg score `3.00`, failures `-`

## Failures By Class


## Prompt Results

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

### q011 PASS

- Prompt: Which channels talk about the same subjects most often?
- Class: `comparison`
- Status: `Completed`
- Score: `3`
- Sources: `12`
- Failure: `-`
- Source videos: AI chat apps are driving me insane | Going Back To Next | How I code with AI right now | It’s actually over now | PewDiePie is right about AI | Vibe coding is already dead | What happened to me? | Why Github Actually Won | Why I moved away from SQL

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
12. Why Github Actually Won - Theo - t3․gg: accountants or CEOs trying to optimize for revenue rather than the developer experience that we cared so much about. In the end, we won because the open source community started to converge on distributed version control and we were the only ones in the hosting space that truly cared about how developers worked at all. The only ones who questioned this approached it from first principles tried to make it better holis... [12]

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

1. Open source is dying - Theo - t3․gg / TL;DR: AI is causing significant damage to open source through PR spam, decreased contribution quality, and financial threats to maintainers' traditional revenue streams Maintainers are experiencing burnout from dealing with low-quality AI-generated contributions and increasingly entitled/toxic users who have unrealistic expectations GitHub has failed to provide adequate moderation tools, forcing maintainers to build their ... [1]
2. My favorite browser is (kind of) dead - Theo - t3․gg: open- Source freely available and still being meaningfully maintained but they're nowhere near as ready as Arc is and I'm not ready to leave it behind yet and honestly the way I'm feeling now is that I'm more invested in the success of Ark than they are and that shouldn't be the case I really hope I'm wrong here I do genuinely hope so and if I have a good conversation with Josh I will certainly do a follow-up video b... [2]
3. Open source is dying - Theo - t3․gg: poorest. Really crippled the image I had in relation to your channel and content. I have been contributing to projects since before you were born. such an attitude, including some YouTube codes of yours that I've never received before. Feel free to block me. Mature. This is somebody who never wrote code before AI. Straight up. And I promise you, you were not contributing to code years ago, [ __ ] And this is the hot ... [3]
4. Which browser should you use right now? - Theo - t3․gg / Key Points: no customization Not recommended:** Safari - crashes websites, bad developer experience Orion - broken Chrome extension support, closed source Firefox - privacy promise abandoned, gradient issues, poor performance Brave - causes website issues, aggressive crypto promotion, bad UX Dia - doesn't work, terrible vertical real estate Ladybird - not meant to be used Personal Context and Philosophy The speaker previously re... [4]
5. Open source is dying - Theo - t3․gg: Transcript: Open source is incredibly important to me. I can say confidently I would not be here today if it wasn't for open- source software. It's a huge part of how I started my career, got into YouTube, and made all of this happen. Life without open source is genuinely hard for me to imagine, which is why I'm really scared right now. We're finally at the point where AI is having a real impact on open source. And i... [5]
6. Corepack is dead, and I'm scared - Theo - t3․gg / TL;DR: by default, which backfired and led to removal discussions instead. Corepack allowed developers to specify and auto-install the correct package manager version per project, improving reproducibility and easing open-source contributions. The Node Package Maintenance working group formalized a roadmap that includes revising the downloads page, separating Corepack documentation, and removing it from distribution. Key ma... [6]
7. Open source is dead now? - Theo - t3․gg / Full transcript: If you've been paying attention to my content recently, you know that I've become a much stronger advocate of open source. Not that I wasn't before, but I think now more than ever, it's really important that we're open sourcing our software, that we're supporting open source communities, and that we're building in a way where things can build on top of each other. I am really scared of a future where we stop open sou... [7]
8. Did Anthropic just kill Figma? - Theo - t3․gg / Full summary: At a glance Anthropic launched "Claude Design," a new product for designing user interfaces, which the reviewer finds genuinely exciting and potentially threatening to Figma. The reviewer tested Claude Design by creating a marketing site prototype for "T3 Code," finding the initial output workable but requiring significant iterative feedback to fix word wrap, layout, and logo issues. Claude Design includes useful col... [8]
9. Did Claude really get dumber again? - Theo - t3․gg / Full summary: At a glance Claude models (Opus 4.6, 4.7, Sonnet 4.6) are experiencing widespread, measurable performance regressions, not just user perception. Regressions stem from multiple layers: the Claude Code harness, API changes, tokenization updates, compute routing, and thinking redaction—not just the base model itself. Claude Code's harness is poorly engineered, wasting tokens and making the model perform significantly wo... [9]
10. This model is kind of a disaster. - Theo - t3․gg / Full summary: At a glance Anthropic's new Opus 4.7 model is described as a "disaster" that regresses in consistency and quality despite showing occasional impressive peaks. Aggressive safety guardrails and system prompts inadvertently lobotomize the model, causing it to flag benign tasks (like cryptography puzzles or personal website updates) as security threats and hard-lock chats. The creator argues that perceived model regressi... [10]
11. Claude's new Cursor killer just dropped - Theo - t3․gg / Full summary: At a glance Anthropic released a new Claude Code desktop app, integrating Claude Chat, Co-work, and Code into a single application, replacing the CLI. The reviewer finds the new desktop app severely flawed, citing numerous UX bugs, missing basic features, and poor performance, arguing it barely improves upon the "trash" CLI. Compared to alternatives like Codex and the reviewer's own project (T3 Code), the Claude app ... [11]
12. A letter to tech CEOs - Theo - t3․gg / Full summary: At a glance The author argues that despite increased risks (cloning, self-hosting, security vulnerabilities), businesses must open-source their software to survive the AI-driven future. Historically, giant SaaS companies (like Salesforce) won by building massive feature moats, making it impossible for competitors to satisfy every customer's bespoke needs. Plugin systems fail as a solution to the feature gap because t... [12]

### q028 PASS

- Prompt: Where do different videos in my library disagree on this topic?
- Class: `comparison`
- Status: `Completed`
- Score: `3`
- Sources: `12`
- Failure: `-`
- Source videos: Gemini 3 Pro is the best model ever made | I hate that this is still happening | JavaScript Frameworks in 2025 | Okay, I'm a bit scared now... | Opus 4.6 Is The Best Coding Model Ever Made* | The "Wrong Way" To Use React | The case against toasts | The code editor wars continue... | WWDC was weird. | We need to talk about the Claude Code rate limits | You’re all wrong | Zod finally has competition (...created by Zod?)

#### Answer

Retrieved evidence for: Where do different videos in my library disagree on this topic?

The answer model is unavailable, so this fallback lists the highest-ranked saved excerpts instead of synthesizing beyond them.

Comparison frame: both the listed excerpts and their source videos are relevant candidates, while the exact similarities, differences, or counterarguments should be checked against the cited text below.

1. WWDC was weird. - Theo - t3․gg: that showed incorrect icon sizing and alignment as supposed proof of iOS fidelity. **Speaker's critique of Flutter**: The speaker identifies as "Flutter's number one hater who uses accessibility as their main argument," expressing frustration that Flutter's attempt to show improved iOS styling had obvious errors like wrong icon sizes and misalignment. Developer Tools and Open-Source Initiatives **Swift-based contain.... [1]
2. I hate that this is still happening - Theo - t3․gg: hear the speaker say seconds later, "By the way, don't do this." That must hurt. That must genuinely suck. And I feel bad for the devs who are learning, that don't know any better, who trusted this resource with 7 million subs and almost million plays to be a good thing to follow along with when it isn't. That hurts a lot. And I am not one of the ones who's going to go after the devs for doing this. I will say they s... [2]
3. We need to talk about the Claude Code rate limits - Theo - t3․gg / Overview: This video examines Anthropic's controversial rate limit changes for Claude Code subscribers, which restrict usage during peak weekday morning hours. The speaker analyzes the underlying GPU compute crisis driving this decision, exploring how Anthropic's slow infrastructure investments and explosive customer growth have created fierce internal competition for resources between research, product, and user teams. The vi... [3]
4. Opus 4.6 Is The Best Coding Model Ever Made* - Theo - t3․gg: tokens — 2-4x more expensive than GPT 5/5.1, roughly 2x more than GPT 5.2/5.2 Codex. New features include team orchestration with parallel agents in Claude Code and API "effort levels" for reasoning intensity. Downsides noted: the model feels slower (5-10 minutes vs 1-2 minutes for tasks), less pleasant to interact with (more templated responses), and still makes "dumb" mistakes like reporting placeholder credentials... [4]
5. Zod finally has competition (...created by Zod?) - Theo - t3․gg: ion standards. Colin (Zod's creator) worked with creators of Valibot and Arktype to develop the "Standard Schema" spec—a common interface for multiple validation libraries. Standard Schema allows framework and library authors to support multiple validators (Zod, Valibot, Arktype) without writing separate adapters for each. The spec is designed for library/framework authors, not end users; it enables ecosystem-wide in... [5]
6. You’re all wrong - Theo - t3․gg: our two groups. Sky is blue, sky is gray. We split this. Sky is blue. This group they read about blue skies. This group reads about gray skies and then groups three and four we swap. What do you think happens if you ask each of these people before and after reading how strongly do they feel about this belief? So I am six out of 10 sure the sky is blue. You have this person they say this and then you give them an arti... [6]
7. Okay, I'm a bit scared now... - Theo - t3․gg / Key Points: also produced a correct answer (139 and ending in 662). This success rate deeply concerns the creator about the future viability of programming competitions. **Potential Training Data Concern**: The creator raises the possibility that solutions might have been trained on existing publicly available Advent of Code solutions, since participants typically open-source their solutions after competitions end. The creator p... [7]
8. JavaScript Frameworks in 2025 - Theo - t3․gg: side complexity while ignoring server-side tradeoffs; similarly, the shift from SPA to isomorphic models exposes frontend devs to complexity they previously ignored. React Compiler and Svelte 5 Runes represent opposing compiler philosophies—React Compiler auto-optimizes by adding memoization, while Svelte trades minimal syntax for more expressive reactivity—ironically both frameworks have traded their original philos... [8]
9. The "Wrong Way" To Use React - Theo - t3․gg / Overview: ed as "Shinobi" or linked in description) about data collocation in React components, triggered by recent React 19 suspense drama. The creator explains the fundamental conflict between React's component model (where components should be self-contained with their own state and data) and the performance problems this creates when components fetch their own data. The video includes extensive discussion of a Twitter thre... [9]
10. Gemini 3 Pro is the best model ever made - Theo - t3․gg / Key Points: anything seen before but gets stuck more frequently and requires active supervision. Google Scale Claims vs Reality While Google's CEO claimed they're "shipping Gemini at the scale of Google," the reviewer noted users are hitting rate limits that don't reflect Google-scale infrastructure capacity. This disconnect between marketing language and actual availability was highlighted as worth watching. [10]
11. The code editor wars continue... - Theo - t3․gg: about code you're watching all these videos you know what you're doing it's a cool place to be I think that's all I got on this one I'm excited to see where all the stuff goes until next time peace nerds [11]
12. The case against toasts - Theo - t3․gg: What's worse than a toast? No feedback at all. So, you don't have time to design or build a better feedback mechanism. I guess a toast is better than nothing. Very fair. This was awesome, Max. Thanks for inspiring me to go on this rant and also inspiring Agore to go spin his blog back up so he can write about this, too. Always surprised how trillion dollar companies don't care as much about basic US x things like AWS... [12]

### q029 PASS

- Prompt: Which videos are most aligned with each other?
- Class: `comparison`
- Status: `Completed`
- Score: `3`
- Sources: `10`
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
7. Figma filed for their IPO (and revealed EVERYTHING) - Theo - t3․gg: don't use any Adobe software for any of our stuff for any of my businesses. I do not like Adobe. All my thumbnails are done in Affinity Photo. All our videos are edited in Final Cut. All our graphics is done in other Affinity software or even in Figma. We avoid Adobe to the best of our ability because I do not like them. Know that as I say, this sucks. The fact that Figma couldn't exit that way is not good. It is unf... [7]
8. TypeScript just changed forever - Theo - t3․gg: JavaScript could scale to companies in codebases the size of places like Microsoft when Microsoft tried to write JavaScript code they ran into the absolute that was trying to keep it working when lots of devs are contributing to lots of files and lots of places typescript was written by unders to solve this problem and despite solving it really well it introduced a new problem which is when we have these giant code..... [8]
9. Predicting OpenAI's future via their acquisitions - Theo - t3․gg: TL;DR OpenAI is on an unusual acquisition spree for a startup, targeting companies like Windsurf (failed), IO/Jony Ive, Statsig, and the Alex Xcode agent team. The speaker argues OpenAI is buying pre-aligned, proven teams to solve the difficult problem of staffing new product verticals without the risks of traditional hiring. Acquiring founder-led teams provides OpenAI with "product leads" (visionaries), management,.... [9]
10. JavaScript Frameworks in 2025 - Theo - t3․gg: made here that compilation and bundling are the core of how modern JS apps are created but they're also where the complexity tends to come in JS tooling I'm sure Carson over in the htx world is laughing at us benefits are Ms though types lenting tree shaking code splitting minification isomorphism macros dsls monolithic authoring and distributed deployment if you don't think webd is way better than it used to be then... [10]

### q030 PASS

- Prompt: Which videos offer the strongest counterargument?
- Class: `comparison`
- Status: `Completed`
- Score: `3`
- Sources: `8`
- Failure: `-`
- Source videos: Gemini 3 Pro is the best model ever made | JavaScript Frameworks in 2025 | Okay, I'm a bit scared now... | Opus 4.6 Is The Best Coding Model Ever Made* | WWDC was weird. | We need to talk about the Claude Code rate limits | Why I moved away from SQL | You’re all wrong

#### Answer

Retrieved evidence for: Which videos offer the strongest counterargument?

The answer model is unavailable, so this fallback lists the highest-ranked saved excerpts instead of synthesizing beyond them.

Comparison frame: both the listed excerpts and their source videos are relevant candidates, while the exact similarities, differences, or counterarguments should be checked against the cited text below.

1. WWDC was weird. - Theo - t3․gg: that showed incorrect icon sizing and alignment as supposed proof of iOS fidelity. **Speaker's critique of Flutter**: The speaker identifies as "Flutter's number one hater who uses accessibility as their main argument," expressing frustration that Flutter's attempt to show improved iOS styling had obvious errors like wrong icon sizes and misalignment. Developer Tools and Open-Source Initiatives **Swift-based contain.... [1]
2. You’re all wrong - Theo - t3․gg: our two groups. Sky is blue, sky is gray. We split this. Sky is blue. This group they read about blue skies. This group reads about gray skies and then groups three and four we swap. What do you think happens if you ask each of these people before and after reading how strongly do they feel about this belief? So I am six out of 10 sure the sky is blue. You have this person they say this and then you give them an arti... [2]
3. Opus 4.6 Is The Best Coding Model Ever Made* - Theo - t3․gg: tokens — 2-4x more expensive than GPT 5/5.1, roughly 2x more than GPT 5.2/5.2 Codex. New features include team orchestration with parallel agents in Claude Code and API "effort levels" for reasoning intensity. Downsides noted: the model feels slower (5-10 minutes vs 1-2 minutes for tasks), less pleasant to interact with (more templated responses), and still makes "dumb" mistakes like reporting placeholder credentials... [3]
4. Okay, I'm a bit scared now... - Theo - t3․gg / Key Points: also produced a correct answer (139 and ending in 662). This success rate deeply concerns the creator about the future viability of programming competitions. **Potential Training Data Concern**: The creator raises the possibility that solutions might have been trained on existing publicly available Advent of Code solutions, since participants typically open-source their solutions after competitions end. The creator p... [4]
5. We need to talk about the Claude Code rate limits - Theo - t3․gg / Overview: This video examines Anthropic's controversial rate limit changes for Claude Code subscribers, which restrict usage during peak weekday morning hours. The speaker analyzes the underlying GPU compute crisis driving this decision, exploring how Anthropic's slow infrastructure investments and explosive customer growth have created fierce internal competition for resources between research, product, and user teams. The vi... [5]
6. JavaScript Frameworks in 2025 - Theo - t3․gg: side complexity while ignoring server-side tradeoffs; similarly, the shift from SPA to isomorphic models exposes frontend devs to complexity they previously ignored. React Compiler and Svelte 5 Runes represent opposing compiler philosophies—React Compiler auto-optimizes by adding memoization, while Svelte trades minimal syntax for more expressive reactivity—ironically both frameworks have traded their original philos... [6]
7. Gemini 3 Pro is the best model ever made - Theo - t3․gg / Key Points: anything seen before but gets stuck more frequently and requires active supervision. Google Scale Claims vs Reality While Google's CEO claimed they're "shipping Gemini at the scale of Google," the reviewer noted users are hitting rate limits that don't reflect Google-scale infrastructure capacity. This disconnect between marketing language and actual availability was highlighted as worth watching. [7]
8. Why I moved away from SQL - Theo - t3․gg: plication development. Convex's approach enables better AI coding experiences because infrastructure is expressed purely in TypeScript/JavaScript rather than SQL or configuration files that LLMs struggle with. Limitations exist: Convex works best for TypeScript-only applications; if you need separate backends in Go/Rust, CLI tools, or multiple teams accessing the same database, it's not ideal. The lock-in concern has... [8]

### q032 PASS

- Prompt: What does the newest video add that older ones did not?
- Class: `comparison`
- Status: `Completed`
- Score: `3`
- Sources: `9`
- Failure: `-`
- Source videos: Everything needs to change | What happened to me? | What happens now? | wtf is Y Combinator doing???

#### Answer

Retrieved evidence for: What does the newest video add that older ones did not?

The answer model is unavailable, so this fallback lists the highest-ranked saved excerpts instead of synthesizing beyond them.

1. wtf is Y Combinator doing??? - Theo - t3․gg / Key Points: VSCode fork with unusual features**: A fork of VSCode now exists that has added TikTok and gambling directly into the editor, a development the video creator finds absurd. **Reaction to the trend**: The creator asks "What the hell is going on???" expressing clear bewilderment at the direction this project has taken. [1]
2. What happened to me? - Theo - t3․gg: is this a thing others care about too? So to take this my skateboard taught me how to code idea. That's 10 out of 10 exciting for me. Like obviously I really want to talk about this. Unique insight also 10 out of 10. These are things I haven't seen others communicate. Obviously nobody could talk about my love of my skateboard the way I can. But do people care? No, the result of this is that this video averages across... [2]
3. What happened to me? - Theo - t3․gg: videos about new models mainly because he didn't have much insight yet. over time covering the models, he got tons of more insight into things like pricing, the capabilities, and he built his own benchmarks. Now, if he posts a video about a new model, it's going to be like 3 days later, and it still performs well because people want to see his take more than they want to see the first video about the thing. This has ... [3]
4. What happened to me? - Theo - t3․gg: of my community, the people hanging out in Twitch chat right now who have been there since day one, watched this whole thing happen, many of which joined me on the same journey. So, knowing all of this, I want to talk a bit about how I choose a topic for a video because another one of the questions I get all of the time is, "What is your dream video that you would do if the algorithm wouldn't kill it?" A question I g... [4]
5. What happened to me? - Theo - t3․gg: There's a comment I've been seeing a lot lately and I wanted to take the time to address it. He's usually in the format of something like, "Man, I missed the old Theo videos. I really liked when Theo would talk about tech and new frameworks and TypeScript, and now all he does is shill AI stuff that he makes money off of." I have a lot of thoughts about this. My first one is that when I look at my channel, sure, there... [5]
6. Everything needs to change - Theo - t3․gg / Key Points: but tools today differ dramatically from even a few weeks ago. This creates a massive opportunity for innovation—going beyond what seems reasonable and trying different approaches. The speaker admits they won't have time to try most new tools but enjoys seeing them. [6]
7. What happens now? - Theo - t3․gg / Overview: This video is a deep dive response to an article by Chris Gregory about how AI tools like Claude Code and Cursor are fundamentally changing software development. The speaker explores the thesis that while code has become cheap to produce, software remains expensive because the real costs — problem understanding, maintenance, distribution, and architecture — haven't changed. The discussion covers the rise of "disposab... [7]
8. wtf is Y Combinator doing??? - Theo - t3․gg / TL;DR: A VSCode fork has been created that adds TikTok and gambling functionality. The video expresses bewilderment at this development, questioning what is happening. The content references Y Combinator in relation to this situation (per the video title). [8]
9. What happened to me? - Theo - t3․gg: result the way I think about things has changed. There are different pieces of how I would rank a video idea. Obviously, there's my excitement level. Like how excited am I about this topic? There is unique insights. This is an important one for me. Like do I have anything unique to add? If somebody else has a video on the topic and said everything I would want to say, I don't need to do the video. I do a video when I... [9]

### q046 PASS

- Prompt: Which videos discuss tradeoffs between speed and accuracy?
- Class: `comparison`
- Status: `Completed`
- Score: `3`
- Sources: `12`
- Failure: `-`
- Source videos: Claude Code has a big problem | Going Back To Next | Hacking LightHouse Scores | JavaScript Frameworks in 2025 | Opus 4.6 Is The Best Coding Model Ever Made* | Skip just dropped - "it's like React, for your Backend" | The fastest website ever? | Vite Raised $4.6 Million To Fix JavaScript

#### Answer

Retrieved evidence for: Which videos discuss tradeoffs between speed and accuracy?

The answer model is unavailable, so this fallback lists the highest-ranked saved excerpts instead of synthesizing beyond them.

1. Going Back To Next - Theo - t3․gg / Overview: ents why he ultimately decided to return to Next.js and JavaScript. Theo provides extensive commentary throughout, offering technical insights, personal experiences from his time at Twitch, and analysis of the tradeoffs between different technology stacks. The discussion covers language ergonomics, error handling, developer experience, context switching costs, and a decision framework for choosing between technology.... [1]
2. The fastest website ever? - Theo - t3․gg: every few months the McMaster website goes viral for being super fast and it really is just navigating the website feels like it's flying and the fact that it's using things that aren't modern webtech is really interesting but is it just vanilla HTML and can we go faster if we Branch away from that we have a lot to dive into here this isn't a simple thing to cover because there's so many misconceptions about what mak... [2]
3. Vite Raised $4.6 Million To Fix JavaScript - Theo - t3․gg: e Remix, Astro, SvelteKit, and others. The speaker argues that VC-backed open source has different risks but isn't inherently worse than corporate-backed or hobbyist models, each having distinct sustainability tradeoffs. Rolldown aims to become the unified bundler for Vite in both dev and prod, potentially enabling fast production builds by end of year. VoidZero plans to offer a separate enterprise-focused toolchain.... [3]
4. The fastest website ever? - Theo - t3․gg / Takeaways: Don't attribute speed to architecture alone**: McMaster's speed comes from deliberate prefetching engineering, not from avoiding frameworks. Their custom JS solution is essentially a custom framework. **PageSpeed scores don't tell the whole story**: A site can feel incredibly fast to users while showing poor metrics; actual user experience should drive optimization decisions. **Prefetch strategically, not comprehensi... [4]
5. The fastest website ever? - Theo - t3․gg: diagnos of performance issues I have a good feeling there too come on Google there we go still not a perfect score in performance but the accessibility is better it is overall good let's take a look at how it actually feels to browse it this is next faster a copyright distinct entity as a demonstration of what a website that vaguely looks and navigates similar to mcmas but is totally not McMaster or even a reference ... [5]
6. Going Back To Next - Theo - t3․gg: asize that "building faster doesn't mean you're building more wrong" - rapid iteration with productive tools can lead to correct solutions more quickly. A major theme throughout is context switching: switching between different languages, ecosystems, and codebases creates significant productivity loss, and full-stack TypeScript frameworks like Next.js minimize this friction. The discussion includes a framework for ch... [6]
7. Opus 4.6 Is The Best Coding Model Ever Made* - Theo - t3․gg: es like reporting placeholder credentials as critical security issues. Anthropic blocked "partial turn prefill" misuse vectors in the API, which has implications for model-swapping and chat history portability between providers. Speaker's overall verdict: roughly a 5-10% improvement in capability with a 3-5% loss in interaction quality, plus speed regression. Overview This video is an in-depth review of Anthropic's n... [7]
8. The fastest website ever? - Theo - t3․gg / Overview: This extensive technical deep-dive analyzes why the McMaster-Carr industrial supply website is renowned for its speed and then explores whether a Next.js implementation could be even faster. The video deconstructs the misconception that McMaster uses simple "vanilla HTML," revealing instead a complex custom JavaScript framework handling prefetching and client-side navigation. It then examines "Next Faster," a demonst... [8]
9. Claude Code has a big problem - Theo - t3․gg: rewrite core primitives if performance becomes an issue—they've since forked Ink and added native components. Alternatives like Codex (Rust-based, uses Ratatouille) and Open Code (uses alt mode) have different tradeoffs: Codex doesn't rewrap text on resize; Open Code has better performance but loses standard terminal behaviors like text selection. The underlying issue is that terminals weren't designed for complex UI... [9]
10. JavaScript Frameworks in 2025 - Theo - t3․gg: S on the server is almost necessary for optimal sites. The complexity debate is often about perspective: GraphQL vs tRPC comparisons misleadingly focus only on client-side complexity while ignoring server-side tradeoffs; similarly, the shift from SPA to isomorphic models exposes frontend devs to complexity they previously ignored. React Compiler and Svelte 5 Runes represent opposing compiler philosophies—React Compil... [10]
11. Hacking LightHouse Scores - Theo - t3․gg: a little bit slow but it's not terribly slow we're right on that 3se second line but we also get content relatively quick a lot of the stuff that they're complaining about is is because we have a video playing on the homepage and this video player is a bunch of JavaScript because hsjs kind of sucks and there isn't a real alternative also if we go to page speeds to test this this will give us an even more neutral take... [11]
12. Skip just dropped - "it's like React, for your Backend" - Theo - t3․gg: TL;DR Skip is a new reactive framework from Meta, created by Christopher Chade (creator of Excalidraw, Prettier, and Recoil), described as "React for the backend." It acts as a layer between backend services/databases and clients, creating a reactive dependency graph where data changes automatically propagate to all derived collections and subscribed clients. The core mental model uses Collections (sources of data) a... [12]

### q086 PASS

- Prompt: Which video in my library best challenges this topic?
- Class: `comparison`
- Status: `Completed`
- Score: `3`
- Sources: `12`
- Failure: `-`
- Source videos: Can we put Rust in Angular to make it faster? WASM deep dive | Gemini 3 Pro is the best model ever made | How JS ruined the web | JavaScript Frameworks in 2025 | Okay, I'm a bit scared now... | Opus 4.6 Is The Best Coding Model Ever Made* | Tanner just fixed forms (I'm so hyped) | WWDC was weird. | We need to talk about the Claude Code rate limits | What happened to me? | Why I moved away from SQL | You’re all wrong

#### Answer

Retrieved evidence for: Which video in my library best challenges this topic?

The answer model is unavailable, so this fallback lists the highest-ranked saved excerpts instead of synthesizing beyond them.

Comparison frame: both the listed excerpts and their source videos are relevant candidates, while the exact similarities, differences, or counterarguments should be checked against the cited text below.

1. How JS ruined the web - Theo - t3․gg / Key Points: culture often rewards complexity over actual user benefit—engineers who write 30-page proposals for regressions get promoted while those who simply fix problems don't. This cultural problem existed before modern frameworks; the speaker recounts experiences at Twitch where blog managers couldn't embed simple videos correctly despite using WordPress. The problem isn't the tools but rather the culture that incentivizes.... [1]
2. You’re all wrong - Theo - t3․gg: our two groups. Sky is blue, sky is gray. We split this. Sky is blue. This group they read about blue skies. This group reads about gray skies and then groups three and four we swap. What do you think happens if you ask each of these people before and after reading how strongly do they feel about this belief? So I am six out of 10 sure the sky is blue. You have this person they say this and then you give them an arti... [2]
3. Okay, I'm a bit scared now... - Theo - t3․gg / Key Points: also produced a correct answer (139 and ending in 662). This success rate deeply concerns the creator about the future viability of programming competitions. **Potential Training Data Concern**: The creator raises the possibility that solutions might have been trained on existing publicly available Advent of Code solutions, since participants typically open-source their solutions after competitions end. The creator p... [3]
4. Opus 4.6 Is The Best Coding Model Ever Made* - Theo - t3․gg: tokens — 2-4x more expensive than GPT 5/5.1, roughly 2x more than GPT 5.2/5.2 Codex. New features include team orchestration with parallel agents in Claude Code and API "effort levels" for reasoning intensity. Downsides noted: the model feels slower (5-10 minutes vs 1-2 minutes for tasks), less pleasant to interact with (more templated responses), and still makes "dumb" mistakes like reporting placeholder credentials... [4]
5. WWDC was weird. - Theo - t3․gg: that showed incorrect icon sizing and alignment as supposed proof of iOS fidelity. **Speaker's critique of Flutter**: The speaker identifies as "Flutter's number one hater who uses accessibility as their main argument," expressing frustration that Flutter's attempt to show improved iOS styling had obvious errors like wrong icon sizes and misalignment. Developer Tools and Open-Source Initiatives **Swift-based contain.... [5]
6. We need to talk about the Claude Code rate limits - Theo - t3․gg / Overview: This video examines Anthropic's controversial rate limit changes for Claude Code subscribers, which restrict usage during peak weekday morning hours. The speaker analyzes the underlying GPU compute crisis driving this decision, exploring how Anthropic's slow infrastructure investments and explosive customer growth have created fierce internal competition for resources between research, product, and user teams. The vi... [6]
7. What happened to me? - Theo - t3․gg: of my community, the people hanging out in Twitch chat right now who have been there since day one, watched this whole thing happen, many of which joined me on the same journey. So, knowing all of this, I want to talk a bit about how I choose a topic for a video because another one of the questions I get all of the time is, "What is your dream video that you would do if the algorithm wouldn't kill it?" A question I g... [7]
8. JavaScript Frameworks in 2025 - Theo - t3․gg: side complexity while ignoring server-side tradeoffs; similarly, the shift from SPA to isomorphic models exposes frontend devs to complexity they previously ignored. React Compiler and Svelte 5 Runes represent opposing compiler philosophies—React Compiler auto-optimizes by adding memoization, while Svelte trades minimal syntax for more expressive reactivity—ironically both frameworks have traded their original philos... [8]
9. Gemini 3 Pro is the best model ever made - Theo - t3․gg / Key Points: anything seen before but gets stuck more frequently and requires active supervision. Google Scale Claims vs Reality While Google's CEO claimed they're "shipping Gemini at the scale of Google," the reviewer noted users are hitting rate limits that don't reflect Google-scale infrastructure capacity. This disconnect between marketing language and actual availability was highlighted as worth watching. [9]
10. Tanner just fixed forms (I'm so hyped) - Theo - t3․gg / Key Points: ased on form state), improved debuggability. **Drawbacks of controlled in React Native**: Can cause "sticky keys" problem where typing lags because React takes too long to update. The speaker references a past video with Dan Abramov discussing React Native input issues. The library team acknowledges React Native's unique challenges with controlled inputs in their documentation. Philosophy and API Design Principles **... [10]
11. Can we put Rust in Angular to make it faster? WASM deep dive - Theo - t3․gg: TL;DR The video explores integrating Rust-compiled WebAssembly into Angular applications for performance-critical tasks like heavy data processing, numbers, video encoding, and image editing. WebAssembly is not a replacement for JavaScript frameworks; DOM bindings remain a bottleneck, and binary sizes can be problematic. It excels at input-to-output transformations. The host attempts to replicate an Angular+Rust tuto... [11]
12. Why I moved away from SQL - Theo - t3․gg: plication development. Convex's approach enables better AI coding experiences because infrastructure is expressed purely in TypeScript/JavaScript rather than SQL or configuration files that LLMs struggle with. Limitations exist: Convex works best for TypeScript-only applications; if you need separate backends in Go/Rust, CLI tools, or multiple teams accessing the same database, it's not ideal. The lock-in concern has... [12]

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

