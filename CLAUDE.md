# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

dAstIll is a Python command-line tool for downloading and processing YouTube video transcripts using the `youtube-transcript-api` library. The project uses `uv` as its package manager.

## Development Commands

### Setup and Dependencies
```bash
# Install dependencies using uv
uv sync

# Run the application
uv run python main.py <youtube-url>

# Run with specific options
uv run python main.py <youtube-url> -l en es -o output.txt --raw
```

### Running the Application
```bash
# Basic usage
./main.py <youtube-url-or-id>

# With language preference
./main.py <youtube-url> -l de en

# Save to file
./main.py <youtube-url> -o transcript.txt

# Get raw transcript (without cleaning)
./main.py <youtube-url> --raw
```

## Architecture

### Core Components

1. **main.py**: Entry point that handles CLI argument parsing and orchestrates the transcript loading process.

2. **src/dastill/youtube_loader.py**: Core module containing `YouTubeTranscriptLoader` class that:
   - Extracts video IDs from various YouTube URL formats
   - Fetches transcripts using youtube-transcript-api
   - Processes transcripts with cleaning (removes timestamps, music symbols, excessive whitespace)
   - Handles multiple language preferences with fallback to auto-generated transcripts
   - Provides both raw and cleaned transcript outputs

### Key Design Patterns

- The loader separates concerns between URL parsing, API interaction, and text processing
- Transcript data is returned as a dictionary containing metadata (video_id, language, is_generated) along with raw, formatted, and cleaned text versions
- Error handling wraps API calls to provide meaningful error messages