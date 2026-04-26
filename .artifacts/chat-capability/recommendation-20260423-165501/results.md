# Chat Capability Sweep Results

- Generated: `2026-04-23T14:55:47.700747+00:00`
- Base URL: `http://localhost:3544`
- Dataset: `/Users/thorben.woelk/repos/dAstIll/backend/tests/data/chat_capability_prompts.json`
- Prompt count: `10`

## Summary

- Passed prompts: `9/10`
- Answerability pass: `9/10`
- Grounding pass: `9/10`
- Shape pass: `9/10`
- Average score: `2.70`

## Capability Classes

- `recommendation`: passed `9/10`, avg score `2.70`, failures `generic_answer`

## Failures By Class

- `generic_answer`: q059

## Prompt Results

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
3. So I've had gpt-5 for a bit now... - Theo - t3․gg / Key Points: steering. The knowledge cutoff appears to be recent, and it picks up on patterns very well. It does exactly what system prompts instruct—better than anything else the creator has used. Coding Capabilities The creator built most of Skatebench with GPT-5, and it built all demo components shown in the video "first try with no issues." It demonstrated excellent tool-calling behaviors throughout. When given a complex feat... [3]
4. What happened to me? - Theo - t3․gg: of my community, the people hanging out in Twitch chat right now who have been there since day one, watched this whole thing happen, many of which joined me on the same journey. So, knowing all of this, I want to talk a bit about how I choose a topic for a video because another one of the questions I get all of the time is, "What is your dream video that you would do if the algorithm wouldn't kill it?" A question I g... [4]
5. So I've had gpt-5 for a bit now... - Theo - t3․gg: things too. It's knowledge cutoff seems to be pretty recent and it seems to pick up on patterns really well. It does exactly what the [ __ ] you tell it to through the system prompt. better than like anything else I've ever used. Okay, looks like it's done. Looks like it wasn't a big change. Cool. Let's uh rerun this and see how it does. Also, by the way, looks like Cloud 4 Opus still recommends upload thing the most... [5]
6. We stopped using serverless. The results are insane. - Theo - t3․gg: TL;DR UploadThing V7 delivers up to 5x faster uploads, with benchmarks showing improvements from ~5 seconds to ~1.5 seconds for multiple files and ~4 seconds to ~0.5 seconds for single small files. The architecture shifted away from direct S3 uploads to using a custom ingest server, reducing network hops from 7 to 3 and eliminating the need for polling. Moving away from serverless to running their own infrastructure.... [6]
7. You suck at picking projects - Theo - t3․gg: TL;DR The speaker built a project because they personally needed it, not because of exceptional development skills or product vision. Projects built for personal use have been the speaker's most successful creations, often succeeding immediately. Examples of successful personal projects include Upload Thing, Pick Thing, Quick Pick, work at Twitch, and the YouTube channel itself. Creating things you want to exist and.... [7]
8. Serverless: A Comprehensive Breakdown - Theo - t3․gg: oesn't mean I'm moving all my things off in fact I was a new service that we built for upload thing generally speaking everything I build is still built around serverless paradigms but I haven't taken the time recently to break down why and to really showcase the truth of serverless I've also not been able to do it without a certain sponsor behind me not that they ever had meaningful influence over the things I said.... [8]
9. Defending a disaster (modern frontend development rant) - Theo - t3․gg: or for writing this very excited to read it I am a front-end developer who is Fed Up about front-end development if you write front-end this isn't about you personally okay thank you writing a lot of front end recently I just readed the homepage for upload thing and I'm working on a whole other project it's like 95 plus% client side code so thankful it's about how your choices make me angry okay interesting angle cur... [9]
10. My favorite browser is (kind of) dead - Theo - t3․gg: tead of pretending I can organize all of it instead I sort by recency and when I need to upload something that I just did it's going to be right there generally speaking I think folder systems and file systems are poorly architected and they're like an artifact of a previous way computers work that we just deal with now but downloads being the place where the thing I just did goes and sorting it by recent has been a.... [10]
11. The real reason Tea got hacked (it's NOT vibe coding) - Theo - t3․gg: how bad their security was, but still call it a hack because it was still a hack. It's also worth noting that a lot of services expose URLs publicly. If you have a URL to something, you can probably access it most of the time. This includes, but isn't limited to really big services. Like up until somewhat recently, Google Photos had public URLs for all of the things uploaded. But those URLs are randomly generated an.... [11]
12. Breaking up with Vercel - Theo - t3․gg: the set of sponsors that I recommend that I like the most you'll see those popping up in things like tutorials going forward and I do have one last versell sponsored video that has to come out it's actually a collab video Believe It or Not between versell and one of their competing products fly at iio because in both my head and in theirs they're not really competing I'm really excited for that video we're actually g... [12]

### q012 PASS

- Prompt: What is the best single video to understand this topic?
- Class: `recommendation`
- Status: `Completed`
- Score: `3`
- Sources: `7`
- Failure: `-`
- Source videos: Deepseek R1 Is Really, Really Good | Gemini Flash 3 is my new favorite model (yes really) | How Minecraft AI ACTUALLY works | Is Electron really that bad? | Watch this if you know HTML | What happened to me? | Why Microsoft deleted this extension from MILLIONS of computers

#### Answer

Retrieved evidence for: What is the best single video to understand this topic?

The answer model is unavailable, so this fallback lists the highest-ranked saved excerpts instead of synthesizing beyond them.

1. Watch this if you know HTML - Theo - t3․gg / Overview: This video provides an in-depth technical analysis of the evolution of web application rendering strategies, moving from traditional Multi-Page Apps (MPAs) and Single Page Apps (SPAs) to modern hybrid models. The speaker diagrams the data flow and trade-offs of each approach, highlighting the specific problems each model solves and the new complexities it introduces. Key themes include the tension between server-side... [1]
2. Deepseek R1 Is Really, Really Good - Theo - t3․gg: pixels instead of specifying this pixel is this gray this one right next to it's a slightly different gray gradients are really hard to compress because there's a lot of different colors in the range this means anything that changes quickly or has a range of numbers in a small area especially things like confetti suck to compress and seeing this young lean video at the very least made me feel better about the quality... [2]
3. Why Microsoft deleted this extension from MILLIONS of computers - Theo - t3․gg: things I would have been more than willing to forgive Matia if he had just apologized and stopped during this spiral but as he has continued to be wrong he has continued to deny and get worse and worse doing worse and worse things never once taking the time to admit that he was wrong or apologize to the harm he has caused to hundreds of developers in the open source ecosystem into the millions of users of a theme tha... [3]
4. What happened to me? - Theo - t3․gg: of my community, the people hanging out in Twitch chat right now who have been there since day one, watched this whole thing happen, many of which joined me on the same journey. So, knowing all of this, I want to talk a bit about how I choose a topic for a video because another one of the questions I get all of the time is, "What is your dream video that you would do if the algorithm wouldn't kill it?" A question I g... [4]
5. Is Electron really that bad? - Theo - t3․gg: quality of experience is trash we are perfectly aligned I couldn't agree more but the moment you say wow elect sucks at the end of it you've just lost the plot you're not talking about a thing you understand if you think that's the case if you actually think electron is the reason that Discord on desktop sucks you don't understand electron Discord business incentives or basic software development straight up and that... [5]
6. Gemini Flash 3 is my new favorite model (yes really) - Theo - t3․gg: with Gemini 3 Pro, managing to beat out Flash and 2.5 Pro for previous days. And also managing to beat out Sonet 4.5, which is pretty impressive, too. MMU, it pulled the best score to date. Pretty nuts. Screen understanding is pretty good. It crushes the scores from 2.5, Flash, and Pro, which were in the like single digit to just barely double digit percentages, and now it's pulling a 70. All cool stuff. Video unders... [6]
7. How Minecraft AI ACTUALLY works - Theo - t3․gg: in a text editor notice how few pixels are changing on my screen right now basically none of the pixels on my screen are changing at the moment pretty much zero of them which means it's very easy to encode my video at the moment but if I was to switch here and move my arms around really fast suddenly my CPU is going to spike I just watched it go from 4% CPU utilization to seven just from that like that's the nature..... [7]

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
4. What happens now? - Theo - t3․gg: complicated, then everyone could be a YouTuber. Cuz that's the hard part. Cuz that's the first problem you ran into. The radio thing even happens to an extent here, too. If the airplane radios were easier, everyone could land the plane. No, you [ __ ] can't. Be realistic here. 34 of men answer yes to this question. Fun fact, the majority of men think they can land the plane. I bring this up because of a real conversa... [4]
5. I'm so f***ing tired of Obsidian. - Theo - t3․gg: Transcript: This video is going to be a little bit different. If you didn't already know this, I run most of my channel through Notion. Everything from our content calendar and when videos come out to my list of topics that I intend to cover to our research to our assignments to our brands to the sponsors, like everything about what makes a specific video a specific video is managed through Notion. Normally, this isn... [5]
6. Are juniors screwed? (Getting a job in a post-AI world) - Theo - t3․gg: deas based on things being posted on HN and on places like Simon Willis's blog. and I'm going to compare that against my channel and use an AI agent to compare and contrast and find ideas that I might not have covered yet that could be good topics for my channel. I just build random [ __ ] like this all the time when I have a theory or an idea or some question I want to answer. I love using these tools to build all o... [6]
7. Abort Controller is criminally underrated (every react dev should use this) - Theo - t3․gg: don't sleep on a boort controller this is going to be a fun one I will admit I have slept on a boort controller for far too long I really shouldn't have especially for react devs uh if you're a react Dev you've used to use effect which means you almost certainly should also be using a board controller if you're a JS Dev this will benefit you but if you're a react Dev this is almost an essential watch trust me you wan... [7]
8. Software Sucks Now - Theo - t3․gg: be a bit different than y'all might think. Ghosty is a great one. If you're not familiar, Ghosty is my terminal written in Zigg by the creator of all of the cool Terraform stuff over at Hashi Corp. He left and this has been his new pet project. Another weird one, Lossless Cut. If you're not familiar, it's a huge part of how we do our content on this channel. It's an open source video editing software that is apparent... [8]

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
2. What’s a Hard Fork? - Hard Fork: Podcast ASR smoke transcript. This text came from the local OpenAI-compatible ASR endpoint, not from RSS show notes. [2]
3. What’s a Hard Fork? - Hard Fork / Key Points: Transcript Metadata**: The only content in the transcript is a procedural note indicating it is a "smoke transcript" generated by a local OpenAI-compatible ASR endpoint, explicitly stating it did not come from RSS show notes. No definitions, examples, or explanations of a "hard fork" are present. [3]
4. What’s a Hard Fork? - Hard Fork / Overview: The video is titled "What’s a Hard Fork?", suggesting an educational focus on blockchain or software development concepts. However, the actual transcript provides no information on this subject. It consists entirely of an ASR metadata placeholder stating the text was generated by a local OpenAI-compatible ASR endpoint rather than sourced from RSS show notes. [4]
5. Anthropic’s Cybersecurity Shock Wave + Ronan Farrow and Andrew Marantz on Their Sam Altman Investigation + One Good Thing - Hard Fork: are perfectly content with a free weather app on their phone. That is fine for you. But as somebody who loves cool things, new ideas, people having fun. I just wanted to shout out, act me weather because I think it's a really cool thing. Now, what is the likelihood that this app will be purchased by Apple and then shut down? I mean, if that happens, I hope these guys get paid again because somebody has to move the we... [5]
6. A.I. Backlash Turns Violent + Kara Swisher on Healthmaxxing + The Zuck Bot Is Coming - Hard Fork: At a glance Recent weeks have seen anti-AI sentiment escalate into violence, including a Molotov cocktail attack on Sam Altman's home and a shooting at an Indiana city councilman's house over a data center vote. Public trust in AI and the government's ability to regulate it is plummeting, driven by economic fears, elite-driven deployment, and AI companies actively opposing accountability measures. Data center constru... [6]
7. A.I. Backlash Turns Violent + Kara Swisher on Healthmaxxing + The Zuck Bot Is Coming - Hard Fork: mRNA vaccines and AI looking at gene folding. So there was all this real stuff and all this really ridiculous stuff. Right. And so you said sort of like I'm saying a lot of stuff that seems like obviously wrong, but some stuff that seems actually promising. So I want to spend some time and see if I can sort of separate the wheat from the chaff. Right. And I also need to do the stunts because it's funny, right? Like d... [7]
8. A.I. Backlash Turns Violent + Kara Swisher on Healthmaxxing + The Zuck Bot Is Coming - Hard Fork: your earbuds now. And that was helpful. I would say helpful. I mean, the hyperbaric chamber is fucking ridiculous, although I enjoyed it. Right. It was kind of fun to be in there, although I don't like small spaces, but it was so stupid. It's so stupid to have all these people insist that this is the way to go. And I was like, it's really not. You do know that. What is supposed to be happening to you while you're in ... [8]
9. A.I. Backlash Turns Violent + Kara Swisher on Healthmaxxing + The Zuck Bot Is Coming - Hard Fork: which are very good. And the way they live longer is they don't sit around and measure fucking everything or just tell us the world is going to die. That is a lot to do. Your mental state has a lot to do with your longevity. And the only thing I would give it to the wellness grifters, a lot of them, is this idea of collapsing health span with lifespan. And I think that's true. We live to, I think it's 79 in this coun... [9]
10. Anthropic’s Cybersecurity Shock Wave + Ronan Farrow and Andrew Marantz on Their Sam Altman Investigation + One Good Thing - Hard Fork: build this frontier even though it's dangerous and we're going to guide it to this safer place. But, you know, you did build the thing in the first place. So, I just like reminding people of that tension because it is not actually inevitable that we build these systems and yet we do often act as if that were the case. Yeah. Last thing, a lot of the people I know who are plugged into the cybersecurity world are being ... [10]
11. Anthropic’s Cybersecurity Shock Wave + Ronan Farrow and Andrew Marantz on Their Sam Altman Investigation + One Good Thing - Hard Fork / Key Points: hacking water and energy infrastructure. If a Mythos-quality model fell into their hands, the damage would be severe. **US Government Blocked from Access**: Paradoxically, the US government has spent months trying to kill Anthropic, designating them a supply chain risk and ordering federal agencies to stop using Claude. Thus, the US national security apparatus currently lacks access to this critical defensive technol... [11]
12. Anthropic’s Cybersecurity Shock Wave + Ronan Farrow and Andrew Marantz on Their Sam Altman Investigation + One Good Thing - Hard Fork: the sunlit side of the moon from the side that is dark. I also learned that we don't call it the dark side of the moon. That's not the preferred astronomical term. What do we call it? The far side of the moon. I am obsessed with all of these astronauts that are four of them up there, Victor, Christina, Jeremy, Reed. This is my mountain rushmore. I love these people who I've never met. They are adorable. They are incr... [12]

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

### q059 FAIL

- Prompt: What should I watch if I want the deepest dive?
- Class: `recommendation`
- Status: `Completed`
- Score: `0`
- Sources: `0`
- Failure: `generic_answer`
- Notes: assistant answer was too short for the expected prompt type (75 chars) | no grounding sources were attached | recommendation answer had no supporting sources

#### Answer

I can’t answer that from the currently indexed transcripts and summaries.

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

1. What happened to me? - Theo - t3․gg: of my community, the people hanging out in Twitch chat right now who have been there since day one, watched this whole thing happen, many of which joined me on the same journey. So, knowing all of this, I want to talk a bit about how I choose a topic for a video because another one of the questions I get all of the time is, "What is your dream video that you would do if the algorithm wouldn't kill it?" A question I g... [1]
2. It's finally out!!! (Next.js 15 breakdown) - Theo - t3․gg: using a JS file so if I go to a service like pck thing that was built with creat T3 app we would use JS do types to import at type import next. next config it worked it was fine it wasn't great but now nextjs supports a TS file for the next config and this makes it much easier just have a correctly type next config which is more important because of some of the changes coming here in different ways to add things so n... [2]
3. The drama never ends... - Theo - t3․gg: but it's one I felt I had to. I wanted to do my best to cover this reasonably, and I hope you see that for what it is. Let me know what I did right, and more importantly, what I could do better on. And until next time, peace nerds. [3]
4. The fastest website ever? - Theo - t3․gg: what's cool about the app is that it's mostly just plain old idiomatic next for example just using route based code splitting next font for automatic fonts next image for image optimizations server components to prevent the JS size from increasing all the things you normally expect partial pre-rendering for largely static delivery with server side invoked Dynamic Parts yep all cool stuff but as Malta says they do add... [4]
5. It's finally out!!! (Next.js 15 breakdown) - Theo - t3․gg: without versell or even without serverless ler Rob just did a video showing how to deploy to a VPS with nextjs it was really good but they're also seeing some of the things that they like and expect from next that are harder to do in the environments and they're trying to expose those so it's easier to do stuff like an expire time now being a value in the next config that you can configure or stuff like having better... [5]
6. What happened to me? - Theo - t3․gg: audience didn't like the video, that's why it didn't perform. Oh yeah, maybe I should make a video the audience likes. It really does come down to that. There are layers to this, like is the video clickable? Does it start in a way that's entertaining and interesting enough that you continue to watch from there? All of these pieces are important, but I'm not blocked by the algorithm for making certain content. In fact... [6]
7. The fastest website ever? - Theo - t3․gg: in be really nice they're working on getting all of these snuck in to be actually part of nextjs so if you do want these optimizations by the time you watch this video they might already be in next which is really cool that said they were almost all really easy to implement in your code base with a single 150 line of code file you could get half or more of the things that discussing here which is nuts it's so extensi... [7]
8. What happened to me? - Theo - t3․gg: result the way I think about things has changed. There are different pieces of how I would rank a video idea. Obviously, there's my excitement level. Like how excited am I about this topic? There is unique insights. This is an important one for me. Like do I have anything unique to add? If somebody else has a video on the topic and said everything I would want to say, I don't need to do the video. I do a video when I... [8]
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

1. Need animations? Use this library. - Theo - t3․gg / Overview: This video covers a major announcement in the web animation ecosystem: Framer Motion, the popular React animation library with over 4.5 million weekly npm downloads, has become an independent open-source project simply called "Motion." The creator, Matt, has left the Framer company after six years to maintain the library independently with Framer's blessing. This separation clarifies the confusing relationship betwee... [1]
2. What happened to me? - Theo - t3․gg: of my community, the people hanging out in Twitch chat right now who have been there since day one, watched this whole thing happen, many of which joined me on the same journey. So, knowing all of this, I want to talk a bit about how I choose a topic for a video because another one of the questions I get all of the time is, "What is your dream video that you would do if the algorithm wouldn't kill it?" A question I g... [2]
3. The most important function in my codebase - Theo - t3․gg / Overview: This video explores the critical problem of error handling in TypeScript and presents three progressively sophisticated solutions. The speaker begins by explaining why TypeScript's default `try/catch` pattern fails to provide type safety for errors, then demonstrates a custom wrapper function that forces explicit error handling. The discussion expands to cover `neverthrow`, a library that implements Result types for.... [3]
4. What happened to me? - Theo - t3․gg: this can change the same way it changed here. If I start doing more of these types of videos and they perform better than expected, I'll lean more into this. For example, the logging video, we screwed up the export initially and the first version that went up was too short and had most of the content missing. So, we had to re-upload it, which destroys the video performance because a lot of people already saw it, so t... [4]
5. What happened to me? - Theo - t3․gg: audience didn't like the video, that's why it didn't perform. Oh yeah, maybe I should make a video the audience likes. It really does come down to that. There are layers to this, like is the video clickable? Does it start in a way that's entertaining and interesting enough that you continue to watch from there? All of these pieces are important, but I'm not blocked by the algorithm for making certain content. In fact... [5]
6. Shadcn just changed forever - Theo - t3․gg: TL;DR Shadcn introduced "Shadcn Create," a major new customization system that lets developers build their own themed component library instead of using default styles. The new system is built on Base UI primitives instead of Radix UI, though users can switch between the two. Developers can now customize base component library, preset style, color palette, fonts, border radius, icon sets, and accent styles before gen... [6]
7. Need animations? Use this library. - Theo - t3․gg: TL;DR Framer Motion has been spun out as an independent open-source project called "Motion," separating from the Framer company to serve the broader web development community beyond just React. The new Motion library introduces vanilla JavaScript APIs, making its animation capabilities available to all frameworks (Vue, Svelte, Angular, etc.), not just React. Motion has a new dedicated homepage at motion.dev featuring... [7]
8. Boneless UI - Theo - t3․gg: ment, styling, and markup—to build custom design systems. Native HTML and CSS are advancing (popover, anchor, dialog, view transitions) to handle functionality that previously required JavaScript. Overview The video discusses an article by Adam that categorizes modern UI component libraries into four playful but descriptive categories: headless, boneless, skinless, and lifeless. The speaker clarifies that these are n... [8]
9. The most important function in my codebase - Theo - t3․gg: ing type-safe error handling with TypeScript's type narrowing. Three solutions for typed error handling exist on a spectrum: the custom `try-catch` wrapper (lowest friction, copy-paste solution), `neverthrow` (library-based Result type that integrates with TypeScript), and Effect.ts (a paradigm-shifting approach that's essentially its own language). `neverthrow` uses a Result type pattern where functions always retur... [9]
10. Tailwind V4 is WAY better than I expected - Theo - t3․gg: alues now work without brackets for numeric inputs (e.g., `h-54`), gradients support angles, and new utility variants like `@min`, `@max`, `group-has`, `not`, and descendant selectors have been added. Overview This video provides an extensive, hands-on review of the newly released Tailwind V4 beta, a major version representing a complete rewrite of the framework's engine. The host explores the transition to a Rust-ba... [10]
11. React Doesn't Scale - Theo - t3․gg: TL;DR A viral Reddit post claims React codebases become disorganized messes at scale, with very few senior engineers truly understanding the library; the video analyzes these claims in depth. The presenter argues most React problems stem from developer inexperience and wrong mental models (especially OOP/class-based thinking), not the framework itself. Key React issues discussed: misuse of `useEffect`, `useState`, `u... [11]
12. React 19 is finally out! - Theo - t3․gg: ML-level concerns. The React Compiler is entirely client-focused and eliminates the need for manual memoization, improving client-side performance significantly without requiring server-side patterns. Overview This video covers the official stable release of React 19, detailing the major changes that delayed the release and the new capabilities the framework now offers. The presenter explains the technical resolution... [12]

