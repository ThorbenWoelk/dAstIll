# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

dAstIll is a Python command-line tool for downloading and processing YouTube video transcripts using the `youtube-transcript-api` library. The project uses `uv` as its package manager and operates as a stateless application using the file system as the single source of truth for video status.

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

# With channel specification for organized processing
./main.py <youtube-url> --channel "tina huang"

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

#### Video Status Management Commands
```bash
# Add videos to download queue
./main.py add <video-id1> <video-id2> --channel "tina huang"

# Show video queue and status overview
./main.py queue

# Show videos by specific status
./main.py queue --status downloaded
./main.py queue --status processed
./main.py queue --status to_be_downloaded

# Update video status manually
./main.py status <video-id> processed

# Process videos (move from downloaded to processed with channel organization)
./main.py process <video-id1> <video-id2>
./main.py process <video-id> --channel "tina huang"
```

## Architecture

### Core Components

1. **main.py**: Enhanced CLI entry point with subcommands for downloading, listing, managing videos, and configuration.

2. **src/dastill/transcript_loader.py**: Core module containing `YouTubeTranscriptLoader` class that:
   - Extracts video IDs from various YouTube URL formats
   - Fetches transcripts using youtube-transcript-api
   - Processes transcripts with cleaning (removes timestamps, music symbols, excessive whitespace)
   - Handles multiple language preferences with fallback to auto-generated transcripts
   - Uses file system for status tracking (no JSON database)
   - Provides both raw and cleaned transcript outputs

3. **src/dastill/config.py**: Configuration management system that:
   - Stores user preferences in `~/.dastill/config.json`
   - Manages storage base path and formatting options
   - Creates sensible defaults on first run

4. **src/dastill/file_manager.py**: Video file management system (`VideoFileManager` class) that:
   - Uses file system as single source of truth for video status
   - Manages four status levels based on file location:
     - `not_downloaded`: No file exists
     - `to_be_downloaded`: Empty placeholder in `/to_be_downloaded/`
     - `downloaded`: Transcript content in `/downloaded/`
     - `processed`: Final location in `/[channel-name]/` folders
   - Provides status detection, file movement, and statistics

5. **src/dastill/transcript_formatter.py**: Markdown formatting utilities (`TranscriptFormatter` class) that:
   - Formats transcript data as markdown with metadata
   - Handles filename sanitization and generation with security validation
   - Supports channel-specific file organization

### Key Design Patterns

- **Stateless Architecture**: File system is the single source of truth, no JSON database
- **Separation of Concerns**: Each module handles a specific aspect (API, storage, management, config)
- **Configuration-Driven**: User preferences control behavior without code changes
- **File-Based Status**: Video status determined by file location and existence
- **Channel Organization**: Processed files automatically organized by channel
- **Backward Compatibility**: Legacy command-line usage still works
- **No Data Inconsistency**: Eliminates JSON-file system sync issues

### Data Flow

#### Download Flow
1. CLI parses command and arguments (including optional `--channel`)
2. YouTubeTranscriptLoader checks file system for existing video (unless `--force`)
3. If not processed, fetches transcript via API
4. Cleans and formats transcript text
5. Saves to `/downloaded/` folder with channel info in filename
6. Optionally saves to custom output file

#### Processing Flow
1. User runs `process` command with video IDs
2. System moves files from `/downloaded/` to `/[channel-name]/` folder
3. File location change automatically updates status to `processed`
4. Files end up in channel-specific folders for easy Obsidian organization

#### Four-Status System (Stateless)
- **not_downloaded**: No file exists anywhere
- **to_be_downloaded**: Empty placeholder file exists in `/to_be_downloaded/`
- **downloaded**: Transcript content exists in `/downloaded/`, ready for AI processing  
- **processed**: File exists in `/[channel-name]/` folder (e.g., `/tina huang/`)

#### Directory Structure
```
/base_path/
├── to_be_downloaded/          # Empty placeholder files
├── downloaded/                # Downloaded transcripts awaiting processing
├── tina huang/               # Processed Tina Huang videos
├── unknown/                  # Processed videos with unknown channel
└── [other channels]/         # Other channel-specific folders
```