# dAstIll - YouTube Transcript Loader & Channel Monitor

A Python command-line tool for downloading, organizing, and managing YouTube video transcripts with automatic channel monitoring capabilities. dAstIll uses a stateless file-based architecture and RSS feeds to provide a complete transcript management solution without requiring API keys.

## Features

### Core Transcript Processing
- **Smart Deduplication**: File-based status tracking prevents redundant downloads
- **Markdown Storage**: Saves transcripts as organized markdown files with metadata
- **Channel Organization**: Automatic file organization by YouTube channel
- **Multiple Output Formats**: Raw transcript, cleaned text, and custom file outputs
- **Language Support**: Multi-language transcript preference with fallback to auto-generated
- **Stateless Architecture**: File system as single source of truth, no JSON databases

### Automatic Channel Monitoring
- **RSS-Based Monitoring**: Monitor YouTube channels without API keys or quotas
- **Automatic Processing**: Auto-download and process new videos as they're published
- **Configurable Workflows**: Set per-channel auto-download and auto-process preferences
- **Background Monitoring**: Continuous monitoring with configurable check intervals
- **Zero Cost**: Completely free using YouTube's public RSS feeds

## Installation

### Prerequisites
- Python 3.13+
- uv package manager

### Setup
```bash
git clone <repository-url>
cd dAstIll
uv sync
```

## Usage

### Quick Start: Manual Transcript Download
```bash
# Download a transcript (saves as markdown automatically)
uv run python main.py https://www.youtube.com/watch?v=VIDEO_ID

# Or use the explicit download command
uv run python main.py download https://www.youtube.com/watch?v=VIDEO_ID --channel "Channel Name"
```

### Channel Monitoring Setup
```bash
# Add channels to monitor (channel ID required - see section below)
uv run python main.py channel add "Tina Huang" "@TinaHuang1" UC2UXDak6o7rBm23k3Vv5dww --auto-download --auto-process
uv run python main.py channel add "Tech Channel" "@techhandle" UCTechChannelID123 --languages en de

# Enable global monitoring and set check interval
uv run python main.py settings enable
uv run python main.py settings interval 300  # Check every 5 minutes

# Start monitoring (runs continuously until stopped)
uv run python main.py monitor start

# Check monitoring status
uv run python main.py monitor status
```

### Finding YouTube Channel IDs

Since automatic channel ID resolution is unreliable, you need to provide channel IDs manually:

1. **Via RSS Feed**: Visit `https://www.youtube.com/feeds/videos.xml?channel_id=CHANNEL_ID` and try different channel IDs from the channel's page source
2. **Via URL**: Check the channel's page source for `channel_id` or `externalId` fields
3. **Via Third-party Tools**: Use online YouTube channel ID lookup tools

Example for finding @TinaHuang1's channel ID:
- Visit https://www.youtube.com/@TinaHuang1
- Test RSS feed: https://www.youtube.com/feeds/videos.xml?channel_id=UC2UXDak6o7rBm23k3Vv5dww
- If the RSS feed works, the channel ID is valid

### Download Options
```bash
# Specify language preferences and channel
uv run python main.py download <url> -l de en es --channel "Channel Name"

# Force re-download even if already processed  
uv run python main.py download <url> --force

# Get raw transcript without cleaning
uv run python main.py download <url> --raw

# Save to custom file (in addition to markdown storage)
uv run python main.py download <url> -o transcript.txt

# Disable automatic markdown storage
uv run python main.py download <url> --no-markdown
```

### Video Management Commands
```bash
# List all processed videos with statistics
uv run python main.py list --stats

# Show videos by status
uv run python main.py queue --status downloaded
uv run python main.py queue --status processed

# Get detailed info for a specific video
uv run python main.py info VIDEO_ID

# Process videos (move from downloaded to channel folders)
uv run python main.py process VIDEO_ID1 VIDEO_ID2 --channel "Channel Name"

# Add videos to download queue
uv run python main.py add VIDEO_ID1 VIDEO_ID2 --channel "Channel Name"

# Remove a video from tracking
uv run python main.py remove VIDEO_ID --delete-file
```

### Channel Management Commands
```bash
# List configured channels
uv run python main.py channel list --enabled-only

# Enable/disable specific channels
uv run python main.py channel toggle "@channelhandle" --disable

# Remove a channel from monitoring
uv run python main.py channel remove "@channelhandle"

# Test monitoring configuration
uv run python main.py monitor test

# Manual check (one-time)
uv run python main.py monitor check
```

## File Organization

dAstIll uses a stateless file-based architecture with four status levels:

```
/base_path/                        # Configurable storage location
├── to_be_downloaded/              # Empty placeholder files (queued)
├── downloaded/                    # Downloaded transcripts awaiting processing
├── tina huang/                    # Processed Tina Huang videos  
├── unknown/                       # Processed videos with unknown channel
└── [other channels]/              # Other channel-specific folders

~/.dastill/
├── config.json                   # Main application configuration
└── channels.json                  # Channel monitoring configuration
```

### Status Flow
1. **not_downloaded**: No file exists
2. **to_be_downloaded**: Empty placeholder in `/to_be_downloaded/`
3. **downloaded**: Transcript content in `/downloaded/`
4. **processed**: Final location in `/[channel-name]/` folders

Each markdown file includes:
- Video metadata (ID, URL, language, channel info)
- Processing information (timestamp, auto-generated status)
- Full cleaned transcript content
- Proper formatting for easy reading and AI processing

## Configuration

### Main Configuration
Configuration is stored in `~/.dastill/config.json`. Default settings:

```json
{
  "storage": {
    "base_path": "~/Documents/totos-vault/AI Memory/youtube library",
    "markdown_format": true
  },
  "transcript": {
    "default_languages": ["en"],
    "include_metadata": true,
    "clean_transcript": true
  }
}
```

### Channel Monitoring Configuration
Channel monitoring is configured in `~/.dastill/channels.json`:

```json
{
  "monitoring": {
    "enabled": true,
    "check_interval": 300,
    "max_videos_per_check": 5
  },
  "channels": {
    "@TinaHuang1": {
      "name": "Tina Huang",
      "channel_id": "UC2UXDak6o7rBm23k3Vv5dww",
      "last_video_id": "video123",
      "monitoring": {
        "languages": ["en"],
        "enabled": true,
        "auto_download": true,
        "auto_process": false
      }
    }
  }
}
```

## Workflow Integration

### Manual Processing Workflow
1. **Download transcripts** using dAstIll for single videos or manual processing
2. **Process with AI tools** like Claude Code for summarization and analysis
3. **Manage your library** using dAstIll's comprehensive management commands

### Automated Monitoring Workflow
1. **Configure channels** you want to monitor
2. **Enable monitoring** and set your preferred check interval
3. **Start monitoring** - dAstIll automatically processes new videos
4. **Use AI tools** to process the organized transcript library

### Example Processing Workflow
```bash
# Setup automated monitoring (provide channel ID manually)
uv run python main.py channel add "AI Research Channel" "@ai-research" UCAIResearchChannelID123 --auto-download --auto-process
uv run python main.py settings enable && uv run python main.py monitor start

# Monitor automatically downloads and organizes new videos
# Later, use Claude Code to process the organized transcripts
claude-code "Please summarize all new transcripts in the 'ai research channel' folder"
```

## Architecture

### Core Components

- **main.py**: Enhanced CLI with comprehensive subcommands and monitoring features
- **transcript_loader.py**: Core transcript fetching and processing logic using YouTube Transcript API  
- **file_manager.py**: Stateless video file management with four-status system
- **transcript_formatter.py**: Markdown file generation and channel organization
- **config.py**: Configuration management system
- **rss_monitor.py**: RSS-based channel monitoring without API requirements
- **channel_config.py**: Channel monitoring configuration management
- **monitoring_service.py**: Core monitoring orchestration and automation service

### Design Principles

- **Stateless Architecture**: File system as single source of truth, no JSON databases
- **Separation of Concerns**: Each module handles specific functionality
- **Configuration-Driven**: Behavior controlled through config files
- **RSS-Based Monitoring**: Uses free YouTube RSS feeds instead of requiring API keys
- **Event-Driven Processing**: Callback-based monitoring with automatic workflow execution
- **Channel Organization**: Automatic file organization by YouTube channel for easy AI processing

### Monitoring Architecture

The monitoring system uses a three-layer architecture:

1. **RSS Monitor**: Fetches YouTube RSS feeds using manually provided channel IDs
2. **Channel Config Manager**: Manages monitored channels and their settings
3. **Monitoring Service**: Orchestrates automatic video detection and processing

This design ensures reliable, scalable monitoring without API dependencies. Channel IDs must be provided manually due to YouTube's bot detection measures.

## Dependencies

- `youtube-transcript-api>=1.6.2` - YouTube transcript fetching  
- `requests>=2.31.0` - HTTP requests for RSS feeds
- Python 3.13+ standard library modules

Built with Python and uv package manager.

## Contributing

1. Fork the repository
2. Create a feature branch
3. Make your changes following the existing code style
4. Add tests for new functionality
5. Update documentation as needed
6. Submit a pull request

## License

[License information to be added]