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

[design image](Gemini_Generated_Image_kxlxdekxlxdekxlx.png)

This is a design concept for an application designed to bring clarity and calm to consuming YouTube content via text.

The core philosophy is "Subtraction over Addition." Every element on screen must serve a distinct purpose. The aesthetic is inspired by Zen principles: simplicity, naturalness, and a focus on the essential content (the text).

1. Design Philosophy & Aesthetic
Name Concept: Enso

Visual Language:

Typography Centric: The reading experience is paramount. We will use high-quality, highly readable typography. A humanist serif font for body content (e.g., Merriweather or Lora) paired with a clean sans-serif for UI elements (e.g., Inter or Geist Sans).

Color Palette: Muted and natural. No harsh dark modes or blinding whites.
Muted red, Off-white/cream (e.g., #FAF9F6) or a very soft warm gray. Dark charcoal (#333333) instead of pure black, reducing contrast strain.

Motion: Transitions should be soft and fluid (Svelte is excellent for this). Opening a transcript shouldn't be a jarring jump; it should feel like unfolding a document.

2. User Interface Concepts
The app consists of essentially two main views.

View 1: The "River" (Home Feed & Subscriptions)
This is the default screen. It is a unified, lazy-loading stream of content from all subscribed channels, ordered newest to oldest.

The Feed:

An infinite scrolling list of "Video Cards."

Rate Limit State:

If the backend hits a YouTube rate limit, a subtle banner appears at the bottom of the viewport.

Text: "Taking a breath. YouTube is slowing us down. Resuming shortly." (Instead of scary error messages).

View 2: The "Sanctuary" (Reader & Editor)
Clicking a card in the River smoothly transitions to this view. This is a distraction-free reading environment.

UI Elements:

The Navigation Header:

A simple "← Back" arrow or chevron on the left.

The Video Title in the center (truncated if necessary).

The Mode Toggle (The Centerpiece):

Below the header, a prominent but sleek toggle switch designed like a rounded pill.

Options: [ Transcript | Summary ].

The active selection is highlighted with the muted accent color background. Switching is instant.

The Content Area (Markdown Viewer):

Centered layout with a max-width (e.g., 750px) for optimal readability.

Renders Markdown beautifully: clear headings, well-spaced bullet points, and distinct paragraphs.

If the summary hasn't finished generating by Ollama yet, the "Summary" side of the toggle shows a gentle pulsing loader animation.

Editing Experience:

We avoid a separate "Edit Mode" button if possible.

Concept: When the user hovers over the text block, a subtle "Edit" icon (a pencil) appears in the top right margin of the text area.

Clicking it seamlessly replaces the rendered Markdown view with a clean, monospaced textarea containing the raw markdown.

The "Edit" icon changes to a checkmark icon ("Save"). Clicking "Save" commits the changes to the Rust backend via API and re-renders the Markdown view.

3. User Flow Example
User opens the app. They see an empty "River" and the input field at the top.

They paste a YouTube URL: youtube.com/@SomeChannel.

The app acknowledges. A few seconds later, video cards begin to gently populate the feed as the Rust backend fetches the latest lists.

The user sees a recent video, "Understanding Rust Lifetimes." They click it.

The view transitions to the "Sanctuary." The "Summary" toggle is active by default.

They read the AI summary. They spot a typo made by the AI.

They click the edit icon, fix the typo in the raw markdown, and click save.

They toggle to "Transcript" to read the verbatim text to verify a specific quote.

They click the "← Back" arrow to return to the River and browse for more content.


# gitignore
Proper setup.