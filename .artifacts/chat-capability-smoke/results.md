# Chat Capability Sweep Results

- Generated: `2026-03-27T14:08:16.886715+00:00`
- Base URL: `http://localhost:3544`
- Dataset: `/Users/thorben.woelk/repos/dAstIll/backend/tests/data/chat_capability_prompts.json`
- Prompt count: `3`

## Summary

- Passed prompts: `3/3`
- Answerability pass: `3/3`
- Grounding pass: `3/3`
- Shape pass: `3/3`
- Average score: `3.00`

## Capability Classes

- `topic_aggregation`: passed `1/1`, avg score `3.00`, failures `-`
- `highlight_lookup`: passed `1/1`, avg score `3.00`, failures `-`
- `timestamp_navigation`: passed `1/1`, avg score `3.00`, failures `-`

## Failures By Class


## Prompt Results

### q001 PASS

- Prompt: What topics come up most across my library?
- Class: `topic_aggregation`
- Status: `Completed`
- Score: `3`
- Sources: `12`
- Failure: `-`
- Source videos: AI Slop & Meta #lastweektonight | AI sucks at art still | Microsoft and OpenAI are breaking up? | My 10-Year-Old Vibe Codes. She Also Does Math by Hand. Why That's the Only Strategy That Works. | My Take on the Hard AI Questions: Jobs, Water, Artificial Romance, School Cheating & More | The end of the Clawdbot saga | This VS Code theme is threatening people? | Vercel Finally Caught Up | What happened to me? | Why Microsoft deleted this extension from MILLIONS of computers
- Tools: Library search (search_library)

#### Answer

Based on the provided excerpts, here are the recurring topics across your library:

**AI-Related Content**
- AI's impact on art, music, and creative work[8][10]
- AI in education and concerns about cheating[7][10]
- AI as augmentation vs. automation[10]
- AI-generated content flooding platforms (Meta's algorithm changes)[12]
- Practical frameworks for discussing AI with skeptics[10]

**Content Creation & Authenticity**
- Choosing video topics based on personal excitement rather than algorithm optimization or money[2][9]
- Balancing audience interests with personal passion[9]
- Avoiding the trap of repeating successful topics just because they performed well[1]

**Open Source & Developer Ecosystems**
- VS Code extension drama around the "Material Theme" extension[1][4]
- Microsoft's removal of extensions from users' computers[4]
- Open source maintainer behavior and sustainability[1][4]

**Frameworks for Understanding Technology Debates**
- Moral Foundations theory for productive conversations[7][10]
- Creation vs. consumption frameworks[5][8]
- Scarcity vs. abundance thinking[10]

**Microsoft/Big Tech Developments**
- Microsoft and OpenAI relationship[3]
- Platform algorithm changes prioritizing AI-generated content[12]

The evidence doesn't provide a complete quantitative analysis of topic frequency across your full library—these are the themes visible in this sample.

### q062 PASS

- Prompt: What highlights have I saved from this video?
- Class: `highlight_lookup`
- Status: `Completed`
- Score: `3`
- Sources: `5`
- Failure: `-`
- Source videos: Hacking LightHouse Scores | The case against toasts | Twitter & Content Creators #lastweektonight
- Tools: Library search (search_library)

#### Answer

The available evidence doesn't include any information about your personal saved highlights. The tool output shows search results for content mentioning "user saved highlights" in video summaries and transcripts, but these are video content matches—not your actual saved highlights data.

If you're referring to a specific video, please let me know which one, and I can check if any saved highlights data is available for it. Otherwise, I don't have access to your personal highlight/annotation data in the current evidence.

### q074 PASS

- Prompt: Which timestamps matter most in this transcript?
- Class: `timestamp_navigation`
- Status: `Completed`
- Score: `3`
- Sources: `10`
- Failure: `-`
- Source videos: 3 Startups Deep in 30 Days: How Nano Banana Pro Just Triggered a Billion-Dollar Chain Reaction | Don't Fall For the Stock Market Hype. The $7,000 Raise AI Is Giving You (That Nobody Mentions) | It’s actually over now | LeCun Said LLMs Are a Dead End—Then Revealed Meta Fudged Their Benchmarks. Both Matter - Here's Why. | McKinsey Says $1 Trillion In Sales Will Go Through AI Agents. Most Businesses Are Invisible. | Nano Banana Pro closed a gap — and unlocked billions #nanobananapro #ai #futureofwork | OpenAI Is Slowing Hiring. Anthropic's Engineers Stopped Writing Code. Here's Why You Should Care. | The Most Important Algorithm Of All Time | This new API makes your websites way faster | Why Every Cold Application You Send Is a Waste of Time (And What Actually Works)
- Tools: Library search (search_library)

#### Answer

The provided ground-truth excerpts do not contain timestamps or time markers, and none of the excerpts discuss which timestamps are important in a transcript. I'm unable to answer which timestamps matter most because that information is not present in the available evidence.

