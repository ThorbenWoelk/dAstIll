# dAstIll - YouTube Transcript Manager

A Python CLI tool for downloading, organizing, and managing YouTube video transcripts with automated channel monitoring and AI processing. No API keys required.

## Features

- **Automated Channel Monitoring**: Subscribe to channels and auto-download new videos via RSS feeds
- **AI-Powered Processing**: Claude Code and Ollama integration for educational transcript enhancement  
- **Channel Organization**: Automatic file organization by YouTube channel in markdown format
- **Docker Deployment**: 24/7 monitoring with comprehensive error handling
- **Zero Cost**: Completely free using YouTube's public RSS feeds

## Quick Start

### Automated Workflow (Recommended)

```bash
# 1. Setup
git clone <repository-url> && cd dAstIll && uv sync
mkdir -p ./data

# 2. Subscribe to channels
uv run python main.py channel subscribe "3Blue1Brown" "@3blue1brown"
uv run python main.py channel subscribe "Tina Huang" "@TinaHuang1"

# 3. Start automated AI workflow
uv run python main.py ai-workflow start
```

This starts a fully automated process that:
- Monitors channels for new videos
- Downloads transcripts automatically
- Processes with Claude Code AI
- Organizes content by channel

### AI Workflow Commands

```bash
# Check status
uv run python main.py ai-workflow status

# Process downloaded transcripts
uv run python main.py ai-workflow process

# Stop workflow
uv run python main.py ai-workflow stop
```

### Manual Download

```bash
# Single video
uv run python main.py download https://www.youtube.com/watch?v=VIDEO_ID

# With channel organization
uv run python main.py download <url> --channel "Channel Name"
```

## Channel Management

```bash
# Subscribe to channel (auto-downloads recent videos)
uv run python main.py channel subscribe "Channel Name" "@handle"

# List channels
uv run python main.py channel list

# Add channel without downloading
uv run python main.py channel add "Channel Name" "@handle"

# Remove channel
uv run python main.py channel remove "@handle"
```

## Video Management

```bash
# List videos
uv run python main.py list --stats

# Process downloaded videos
uv run python main.py process

# Get video info
uv run python main.py info VIDEO_ID
```

## Download Options

```bash
# Language preferences
uv run python main.py download <url> -l en de --channel "Channel"

# Force re-download
uv run python main.py download <url> --force

# Raw transcript
uv run python main.py download <url> --raw
```

## File Organization

Files are organized in a stateless four-status system:

```
/base_path/
├── to_be_downloaded/    # Queued videos (empty placeholders)
├── downloaded/          # Downloaded transcripts awaiting processing  
├── [channel-name]/      # Processed transcripts by channel
└── unknown/            # Processed videos with unknown channel

~/.dastill/
├── config.json         # Application configuration
└── channels.json       # Channel monitoring settings
```

Each markdown file includes video metadata, processing timestamps, and cleaned transcript content.

## Configuration

Configuration uses a hierarchical system with environment variables taking precedence:

### Primary Configuration (.env file)

The storage path is centrally configured in the `.env` file:

```bash
# Copy .env.example to .env and customize
cp .env.example .env

# Edit .env file to set your storage path
DASTILL_BASE_PATH="/path/to/your/transcript/storage"
```

### Secondary Configuration (~/.dastill/config.json)

Additional settings can be configured in the JSON file:

```json
{
  "storage": {
    "base_path": "~/Documents/youtube-transcripts",  # Overridden by DASTILL_BASE_PATH
    "markdown_format": true
  },
  "transcript": {
    "default_languages": ["en"],
    "include_metadata": true,
    "clean_transcript": true
  }
}
```

**Note**: The `DASTILL_BASE_PATH` environment variable takes precedence over the JSON configuration for storage path, ensuring consistent paths across Docker and local execution.

## Docker Deployment

For 24/7 monitoring:

```bash
# Start monitoring service
docker-compose up -d

# View logs  
docker-compose logs -f dastill-monitor

# Stop service
docker-compose down
```

## Ollama Integration

Local AI processing alternative using Ollama:

```bash
# Install Ollama and pull model
ollama pull llama3.2

# Check status
uv run python main.py ollama status

# Process transcripts
uv run python main.py ollama process /path/to/transcripts --model llama3.2
```

## Dependencies

- `youtube-transcript-api>=1.2.1` - YouTube transcript fetching
- `python-dotenv>=1.0.0` - Environment configuration
- `requests>=2.31.0` - HTTP requests for RSS feeds

Requires Python 3.13+ and uv package manager.