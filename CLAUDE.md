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

#### Download Command (Default)
```bash
# Basic usage - downloads and saves as markdown
./main.py <youtube-url-or-id>
./main.py download <youtube-url-or-id>

# With language preference
./main.py <youtube-url> -l de en

# Save to custom file (in addition to markdown storage)
./main.py <youtube-url> -o transcript.txt

# Get raw transcript (without cleaning)
./main.py <youtube-url> --raw

# Force re-download even if already processed
./main.py <youtube-url> --force

# Disable markdown storage
./main.py <youtube-url> --no-markdown
```

#### Management Commands
```bash
# List all processed videos
./main.py list

# Show statistics
./main.py list --stats

# Get info for specific video
./main.py info <video-id>

# Remove video from tracking
./main.py remove <video-id>

# Remove and delete file
./main.py remove <video-id> --delete-file

# Show current configuration
./main.py config
```

## Architecture

### Core Components

1. **main.py**: Enhanced CLI entry point with subcommands for downloading, listing, managing videos, and configuration.

2. **src/dastill/youtube_loader.py**: Core module containing `YouTubeTranscriptLoader` class that:
   - Extracts video IDs from various YouTube URL formats
   - Fetches transcripts using youtube-transcript-api
   - Processes transcripts with cleaning (removes timestamps, music symbols, excessive whitespace)
   - Handles multiple language preferences with fallback to auto-generated transcripts
   - Integrates with video tracking and markdown storage systems
   - Provides both raw and cleaned transcript outputs

3. **src/dastill/config.py**: Configuration management system that:
   - Stores user preferences in `~/.dastill/config.json`
   - Manages storage paths, language preferences, and markdown formatting options
   - Creates sensible defaults on first run (transcripts stored in `~/dAstIll-transcripts/` by default)

4. **src/dastill/video_tracker.py**: Video tracking database that:
   - Maintains a JSON database of processed videos to prevent redundant downloads
   - Tracks metadata like language, generation status, file paths, and processing dates
   - Provides statistics and management functions

5. **src/dastill/markdown_storage.py**: Markdown file storage system that:
   - Saves transcripts as formatted markdown files with metadata in `~/dAstIll-transcripts/` by default
   - Organizes files by date (optional) 
   - Sanitizes filenames and handles file naming conflicts
   - Includes video information, URLs, and processing timestamps

### Key Design Patterns

- **Separation of Concerns**: Each module handles a specific aspect (API, storage, tracking, config)
- **Configuration-Driven**: User preferences control behavior without code changes
- **Deduplication**: Video tracking prevents redundant API calls and storage
- **Flexible Storage**: Multiple output formats supported (raw text, markdown, custom files)
- **Backward Compatibility**: Legacy command-line usage still works
- **Extensible**: Easy to add new storage formats or metadata fields

### Data Flow

1. CLI parses command and arguments
2. YouTubeTranscriptLoader checks if video already processed (unless `--force`)
3. If not processed, fetches transcript via API
4. Cleans and formats transcript text
5. Saves to markdown storage (unless `--no-markdown`)
6. Updates video tracking database
7. Optionally saves to custom output file