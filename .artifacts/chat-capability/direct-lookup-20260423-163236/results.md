# Chat Capability Sweep Results

- Generated: `2026-04-23T14:33:21.347927+00:00`
- Base URL: `http://localhost:3544`
- Dataset: `/Users/thorben.woelk/repos/dAstIll/backend/tests/data/chat_capability_prompts.json`
- Prompt count: `20`

## Summary

- Passed prompts: `0/20`
- Answerability pass: `0/20`
- Grounding pass: `10/20`
- Shape pass: `10/20`
- Average score: `0.00`

## Capability Classes

- `direct_lookup`: passed `0/20`, avg score `0.00`, failures `stream_error`

## Failures By Class

- `stream_error`: q006, q007, q008, q009, q013, q014, q015, q016, q017, q018, q019, q020, q022, q023, q035, q047, q048, q053, q061, q099

## Prompt Results

### q006 FAIL

- Prompt: Find every video that mentions RAG.
- Class: `direct_lookup`
- Status: `StreamError`
- Score: `0`
- Sources: `12`
- Failure: `stream_error`
- Source videos: Anthropic’s Cybersecurity Shock Wave + Ronan Farrow and Andrew Marantz on Their Sam Altman Investigation + One Good Thing | Breaking up with Vercel | Delete your CLAUDE.md (and your AGENT.md too) | Every smart AI model wants to kill you (yes really) | How did we get here? (A rant about Javascript runtimes) | I gave away $1,000 to prove UUIDs are secure | Is Sam Altman evil? The OpenAI Files are wild | It’s time to embrace the AI | Okay, I'm a bit scared now... | We need to talk about Ralph | What’s the best programming language for AI? | it's time for a change.
- Notes: stream ended with an explicit error event | assistant content was empty

#### Answer

_No assistant content._

### q007 FAIL

- Prompt: Find every video that mentions Ollama.
- Class: `direct_lookup`
- Status: `StreamError`
- Score: `0`
- Sources: `12`
- Failure: `stream_error`
- Source videos: Anthropic’s Cybersecurity Shock Wave + Ronan Farrow and Andrew Marantz on Their Sam Altman Investigation + One Good Thing | Breaking up with Vercel | Delete your CLAUDE.md (and your AGENT.md too) | Every smart AI model wants to kill you (yes really) | I gave away $1,000 to prove UUIDs are secure | It’s time to embrace the AI | Okay, I'm a bit scared now... | OpenAI’s open source models are finally here | What’s the best programming language for AI? | Why every dev should avoid React | it's time for a change.
- Notes: stream ended with an explicit error event | assistant content was empty

#### Answer

_No assistant content._

### q008 FAIL

- Prompt: Find every video that mentions semantic search.
- Class: `direct_lookup`
- Status: `StreamError`
- Score: `0`
- Sources: `12`
- Failure: `stream_error`
- Source videos: Anthropic’s Cybersecurity Shock Wave + Ronan Farrow and Andrew Marantz on Their Sam Altman Investigation + One Good Thing | Every smart AI model wants to kill you (yes really) | Is this the end of Chrome? | It’s time to embrace the AI | Okay, I'm a bit scared now... | What’s the best programming language for AI? | i made my own search engine (kind of) | it's time for a change. | “Just Use HTML”
- Notes: stream ended with an explicit error event | assistant content was empty

#### Answer

_No assistant content._

### q009 FAIL

- Prompt: Find every video that mentions YouTube API.
- Class: `direct_lookup`
- Status: `StreamError`
- Score: `0`
- Sources: `12`
- Failure: `stream_error`
- Source videos: Anthropic’s Cybersecurity Shock Wave + Ronan Farrow and Andrew Marantz on Their Sam Altman Investigation + One Good Thing | Breaking up with Vercel | Delete your CLAUDE.md (and your AGENT.md too) | Every smart AI model wants to kill you (yes really) | Google Drive hates developers now | I can't believe nobody's done this before... | I gave away $1,000 to prove UUIDs are secure | It’s time to embrace the AI | Okay, I'm a bit scared now... | What’s the best programming language for AI? | Why every dev should avoid React | it's time for a change.
- Notes: stream ended with an explicit error event | assistant content was empty

#### Answer

_No assistant content._

### q013 FAIL

- Prompt: Give me a quick summary of this video in three bullets.
- Class: `direct_lookup`
- Status: `StreamError`
- Score: `0`
- Sources: `12`
- Failure: `stream_error`
- Source videos: AI images just got dangerously good (RIP diffusion??) | Fixing serverless node.js (by adding servers?) | I Fixed Stripe | I ranked every AI based on vibes | My current stack | Open source is dying | The Tailwind drama | This new Tailwind feature is scarier than I thought
- Notes: stream ended with an explicit error event | assistant content was empty

#### Answer

_No assistant content._

### q014 FAIL

- Prompt: Give me a detailed summary of this video.
- Class: `direct_lookup`
- Status: `StreamError`
- Score: `0`
- Sources: `12`
- Failure: `stream_error`
- Source videos: Did Anthropic just kill Figma? | GPT-5.2 is dumb (I’m tired of benchmarks) | Is Claude 4 a snitch? I made a benchmark to figure it out | My current stack | Open source is dying | We need to talk about Ralph | gpt-5.4 is really, really good
- Notes: stream ended with an explicit error event | assistant content was empty

#### Answer

_No assistant content._

### q015 FAIL

- Prompt: What is the video's core thesis?
- Class: `direct_lookup`
- Status: `StreamError`
- Score: `0`
- Sources: `12`
- Failure: `stream_error`
- Source videos: AI isn't gonna keep improving | I am scared but excited | I'm Finally Moving On (I have a new browser) | Open source is dying | The "right way" to vibe code (engineers, please watch) | This might be the end of WordPress | What happens now? | You’re all wrong
- Notes: stream ended with an explicit error event | assistant content was empty

#### Answer

_No assistant content._

### q016 FAIL

- Prompt: What are the key takeaways from this transcript?
- Class: `direct_lookup`
- Status: `StreamError`
- Score: `0`
- Sources: `12`
- Failure: `stream_error`
- Source videos: Deepseek R1 Is Really, Really Good | Did gpt-5 just shadow drop? Horizon is the best code model ever | I stole all your buttons | Is Sam Altman evil? The OpenAI Files are wild | My new app is really stupid (I wrote none of the code) | Open source is dying | The Truth About React Native | The fastest website ever? | They cut Node.js Memory in half 👀
- Notes: stream ended with an explicit error event | assistant content was empty

#### Answer

_No assistant content._

### q017 FAIL

- Prompt: What are the most actionable ideas in this video?
- Class: `direct_lookup`
- Status: `StreamError`
- Score: `0`
- Sources: `12`
- Failure: `stream_error`
- Source videos: "AI Startups" are over done (finally) | AI has a subsidization problem | Amazon Returns To Office, AWS Employees AREN'T Happy | How JS ruined the web | I might have a new favorite state manager... | Open source is dying | Peering into Claude's soul (I can't believe this is real...) | React feels insane | Vibe coding is already dead
- Notes: stream ended with an explicit error event | assistant content was empty

#### Answer

_No assistant content._

### q018 FAIL

- Prompt: What problem is this video trying to solve?
- Class: `direct_lookup`
- Status: `StreamError`
- Score: `0`
- Sources: `12`
- Failure: `stream_error`
- Source videos: AI mistakes you're probably making | AI sucks at art still | Agentic Coding Has A HUGE Problem | Anthropic is trying SO hard to fix MCP... | Open source is dying | OpenAI’s TikTok Clone Is Interesting… | Vibe coding is already dead
- Notes: stream ended with an explicit error event | assistant content was empty

#### Answer

_No assistant content._

### q019 FAIL

- Prompt: +{Open source is dead now?} What are the strongest arguments made in this video?
- Class: `direct_lookup`
- Status: `HttpError`
- Score: `0`
- Sources: `0`
- Failure: `stream_error`
- Notes: failed to obtain a complete stream

#### Answer

_No assistant content._

### q020 FAIL

- Prompt: What examples does the speaker use to support their point?
- Class: `direct_lookup`
- Status: `HttpError`
- Score: `0`
- Sources: `0`
- Failure: `stream_error`
- Notes: failed to obtain a complete stream

#### Answer

_No assistant content._

### q022 FAIL

- Prompt: What parts of the transcript are most important?
- Class: `direct_lookup`
- Status: `HttpError`
- Score: `0`
- Sources: `0`
- Failure: `stream_error`
- Notes: failed to obtain a complete stream

#### Answer

_No assistant content._

### q023 FAIL

- Prompt: +{Open source is dead now?} What is the clearest explanation in this video?
- Class: `direct_lookup`
- Status: `HttpError`
- Score: `0`
- Sources: `0`
- Failure: `stream_error`
- Notes: failed to obtain a complete stream

#### Answer

_No assistant content._

### q035 FAIL

- Prompt: Which videos mention the same person or company?
- Class: `direct_lookup`
- Status: `HttpError`
- Score: `0`
- Sources: `0`
- Failure: `stream_error`
- Notes: failed to obtain a complete stream

#### Answer

_No assistant content._

### q047 FAIL

- Prompt: Which videos talk about evaluation or judging model outputs?
- Class: `direct_lookup`
- Status: `HttpError`
- Score: `0`
- Sources: `0`
- Failure: `stream_error`
- Notes: failed to obtain a complete stream

#### Answer

_No assistant content._

### q048 FAIL

- Prompt: Which videos mention failure cases or limitations?
- Class: `direct_lookup`
- Status: `HttpError`
- Score: `0`
- Sources: `0`
- Failure: `stream_error`
- Notes: failed to obtain a complete stream

#### Answer

_No assistant content._

### q053 FAIL

- Prompt: Which videos contain step-by-step instructions?
- Class: `direct_lookup`
- Status: `HttpError`
- Score: `0`
- Sources: `0`
- Failure: `stream_error`
- Notes: failed to obtain a complete stream

#### Answer

_No assistant content._

### q061 FAIL

- Prompt: +{Open source is dead now?} What are the most important quotes from this transcript?
- Class: `direct_lookup`
- Status: `HttpError`
- Score: `0`
- Sources: `0`
- Failure: `stream_error`
- Notes: failed to obtain a complete stream

#### Answer

_No assistant content._

### q099 FAIL

- Prompt: Can you answer this with citations from the source videos?
- Class: `direct_lookup`
- Status: `HttpError`
- Score: `0`
- Sources: `0`
- Failure: `stream_error`
- Notes: failed to obtain a complete stream

#### Answer

_No assistant content._

