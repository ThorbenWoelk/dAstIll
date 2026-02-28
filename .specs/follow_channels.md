Now that https://github.com/steipete/summarize exist, create an app with Rust backend and Svelte frontend to download and categorize youtube video transcripts from a list of specified channels. 

# Sub to YouTube channels
The user can add channels to follow by handle, channel ID, or a channel URL. 

The app identifies video URLs (most recent to older) and lazy loads transcripts. User gets shown more and more video transcripts. 

The app handles rate limits gracefully (summarize already does this probably). 

# AI summarization
- if summarize can work with ollama, do it. 
- otherwise use ollama integration from rig crate to format and summarize Youtube transcripts

# Transcripts + summary
- user can read both of them and conveniently switch between them. they're showing in well-formated markdown. 
- user can edit both (-> db update). 

# Development rules for technical implementation
- don't overengineer. *summarize* basically does it all already. we only need convenience around it

# Test
Test by using "Florentin Streams" and https://www.youtube.com/@NateBJones as default channels. 
Test-strategy: Backend to frontend, inner to outer. Unit test to ui-testing. 
Tests should serve documentation purpose. 

# Design concept 

A beautiful 2026 frontend. Mobile first. Following:
[design image](Gemini_Generated_Image_kxlxdekxlxdekxlx.png)

# gitignore
Proper setup.