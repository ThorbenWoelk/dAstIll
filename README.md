# dAstIll - YouTube Transcript Loader

A Python app and command-line tool for downloading, organizing, and managing YouTube video transcripts. Subscribe to recent videos of your favorite channels. No API keys needed. 

## Features

### Core Transcript Processing
- **Markdown Storage**: Saves transcripts as organized markdown files with metadata
- **Channel Organization**: Automatic file organization by YouTube channel
- **Language Support**: Multi-language transcript preference with fallback to auto-generated

### Automatic Channel Monitoring
- **RSS-Based Monitoring**: Monitor YouTube channels without API keys or quotas
- **Subscription**: Auto-download and process new videos as they're published
- **Zero Cost**: Completely free using YouTube's public RSS feeds

### AI-Powered Processing (CH-141 Feature)
- **Claude Code Integration**: Automated transcript enhancement using Claude Code
- **Educational Summarization**: Transform raw transcripts into structured learning content
- **Bash Workflow**: Simple script orchestration for Docker + Claude Code automation

## Quick Start (Docker)

For the complete automated workflow with AI processing:

```bash
# 1. Clone and setup
git clone <repository-url>
cd dAstIll
uv sync

# 2. Subscribe to channels (resolves channel IDs automatically)
uv run python main.py channel subscribe "Tina Huang" "@TinaHuang1"
uv run python main.py channel subscribe "HealthyGamerGG" "@HealthyGamerGG"

# 3. Create data directory for Docker storage
mkdir -p ./data

# 4. Start the full AI workflow (Docker + Claude Code automation)
uv run python main.py ai-workflow start
```

The AI workflow will now:
- Monitor channels and download new videos automatically 
- Process transcripts with Claude Code AI enhancement
- Organize enhanced content by channel
- Run 24/7 with comprehensive error handling

**Note**: By default, Docker stores transcripts in `./data/`. To use a custom location, modify the volume mount in `docker-compose.yml` and update `config.json` accordingly.

## AI Workflow Automation (CH-141) ✅

**Fully automated YouTube transcript monitoring, AI processing, and organization.**

### Prerequisites
- Docker running locally
- Claude Code CLI installed and authenticated (`claude setup-token`)

### Quick Start - Complete Automation
```bash
# 1. Setup channels (one-time)
uv run python main.py channel subscribe "3Blue1Brown" "@3blue1brown"
uv run python main.py channel subscribe "Tina Huang" "@TinaHuang1"

# 2. Create data directory for Docker storage
mkdir -p ./data

# 3. Start the full AI workflow (100% automated)
uv run python main.py ai-workflow start
```

### Core Commands
```bash
# Check status of Docker container and Claude Code
uv run python main.py ai-workflow status

# Process only downloaded transcripts with Claude Code (automated)
uv run python main.py ai-workflow process

# Stop the Docker container
uv run python main.py ai-workflow stop
```

### How It Works - Three Automated Steps

#### Step 1: Docker Monitoring (Automatic)
- Monitors subscribed YouTube channels via RSS feeds
- Downloads new transcript files to `/downloaded/` directory
- Runs 24/7 with automatic rate limit handling
- No user interaction required

#### Step 2: Claude AI Processing (Fully Automated)
- Uses Claude Code CLI in non-interactive mode (`--print --dangerously-skip-permissions`)
- Launches `transcript-education-curator` agent automatically
- Processes all `.md` files in downloaded directory
- 10-minute timeout with comprehensive error handling
- No prompts or user interaction required

#### Step 3: File Organization (Automatic)
- Automatically organizes AI-enhanced transcripts by channel
- Moves files from `/downloaded/` to channel-specific folders
- Maintains clean directory structure
- Updates file tracking automatically

### AI Enhancement Features
The transcript-education-curator agent transforms raw transcripts into:
- **Structured Educational Summaries**: Well-organized content with clear sections
- **Key Concepts**: Main topics and important ideas extracted and highlighted
- **Actionable Takeaways**: Practical insights and lessons you can apply
- **Technical Details**: Code examples, formulas, and technical explanations preserved
- **References**: Resources and links mentioned in the video compiled
- **Comprehensive Learning Resource**: Ready for future reference and AI processing

### Processing Flow (100% Automated)
```
Channel RSS → Docker Monitor → downloaded/ → Claude AI → [channel-name]/
```

### Automation Benefits
- **Zero Manual Intervention**: Set up once, runs forever
- **Intelligent Processing**: Claude Code automatically enhances content quality
- **Clean Organization**: Files automatically sorted by channel
- **Reliable Operation**: Error handling, timeouts, and logging built-in
- **Actually Works**: Unlike SDK approaches, leverages Claude Code's native CLI interface

### Quick Start: Manual Transcript Download
```bash
# Download a transcript (saves as markdown automatically)
uv run python main.py download https://www.youtube.com/watch?v=VIDEO_ID

# Specify channel name for organization
uv run python main.py download https://www.youtube.com/watch?v=VIDEO_ID --channel "Channel Name"
```

### Channel ID Resolution

dAstIll automatically resolves YouTube channel IDs from handles (e.g., @channelname). If automatic resolution fails, you can manually provide the channel ID:

1. Visit the channel page on YouTube (e.g., https://www.youtube.com/@channelname)
2. View page source (right-click → View Page Source)
3. Search for "channelId" or "externalId"
4. The ID will be a string starting with 'UC' (e.g., UCX6OQ3DkcsbYNE6H8uQQuVA)

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
uv run python main.py process                                # Process all downloaded videos
uv run python main.py process VIDEO_ID1 VIDEO_ID2            # Process specific videos
uv run python main.py process --channel "Channel Name"       # Process all with channel override

# Add videos to download queue
uv run python main.py add VIDEO_ID1 VIDEO_ID2 --channel "Channel Name"

# Remove a video from tracking
uv run python main.py remove VIDEO_ID --delete-file
```

### Channel Management Commands
```bash
# List configured channels
uv run python main.py channel list --enabled-only

# Add a channel to monitoring (without downloading videos)
# Channel ID is optional - will be automatically resolved from handle
uv run python main.py channel add "Channel Name" "@channelhandle"

# Subscribe to a channel (adds channel and downloads recent videos)
# Channel ID is optional - will be automatically resolved from handle
uv run python main.py channel subscribe "Channel Name" "@channelhandle" --recent-count 15

# Or provide channel ID directly if resolution fails
uv run python main.py channel subscribe "Channel Name" "@channelhandle" CHANNEL_ID --recent-count 15

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

### Recommended Docker Workflow

**Complete Setup Process:**

1. **Subscribe to channels** using CLI commands:
   ```bash
   uv run python main.py channel subscribe "Channel Name" "@handle" CHANNEL_ID --recent-count 15
   ```

2. **Deploy the monitoring service** with Docker Compose:
   ```bash
   docker-compose up -d
   ```

3. **Service automatically**:
   - Starts monitoring immediately on container startup
   - Checks subscribed channels every 2 minutes for new videos
   - Downloads new video transcripts automatically
   - Handles rate limits with 3-hour sleep periods
   - Restarts automatically if the container fails

4. **Monitor the service**:
   ```bash
   # View logs
   docker-compose logs -f dastill-monitor
   
   # Check status
   docker-compose exec dastill-monitor uv run python main.py monitor status
   
   # Stop service
   docker-compose down
   ```

### Manual Processing Workflow
1. **Download transcripts** using dAstIll for single videos or manual processing
2. **Process with AI tools** like Claude Code for summarization and analysis
3. **Manage your library** using dAstIll's comprehensive management commands

### Automated Monitoring Workflow
1. **Configure channels** you want to monitor
2. **Enable monitoring** and set your preferred check interval
3. **Start monitoring** - dAstIll automatically processes new videos
4. **Use AI tools** to process the organized transcript library

### Rate Limit Handling
When the YouTube API rate limit is reached:
- The service automatically detects rate limit errors
- Sleeps for 3 hours before resuming
- Logs the event clearly in Docker logs
- Continues monitoring after the sleep period

### Example Processing Workflow
```bash
# Setup automated monitoring (provide channel ID manually)
uv run python main.py channel add "AI Research Channel" "@ai-research" UCAIResearchChannelID123 --auto-download --auto-process
uv run python main.py settings enable && uv run python main.py monitor start

# Monitor automatically downloads and organizes new videos
# Later, use Claude Code to process the organized transcripts
claude-code "Please summarize all new transcripts in the 'ai research channel' folder"
```

## Docker Deployment

dAstIll can be deployed as a long-running Docker service for continuous channel monitoring:

### Quick Start with Docker Compose

```bash
# Build and start the monitoring service
docker-compose up -d

# View logs
docker-compose logs -f dastill-monitor

# Stop the service
docker-compose down
```

### Docker Configuration

The service includes:
- **Automatic monitoring**: Continuously checks subscribed channels for new videos
- **Volume mounts**: Persistent storage for videos and configuration
- **Health checks**: Ensures service reliability
- **Restart policy**: Automatically recovers from failures
- **Rate limit handling**: Automatically sleeps for 3 hours when hitting YouTube API limits

```yaml
services:
  dastill-monitor:
    build: .
    volumes:
      - ./data:/data              # Video storage and configuration
    environment:
      - DASTILL_BASE_PATH=/data
      - DASTILL_CONFIG_DIR=/data/config
    restart: unless-stopped
```

### Running CLI Commands in Docker

```bash
# Subscribe to a channel using Docker (requires cli profile)
docker-compose --profile cli run --rm dastill-cli channel subscribe "Tech Channel" "@techchannel" UC123456789

# Check monitoring status
docker-compose --profile cli run --rm dastill-cli monitor status
```

## Channel Subscriptions

The subscription feature allows you to quickly onboard new channels by downloading their recent videos:

### Subscribe to a Channel

```bash
# Subscribe and download the latest 15 videos (default and RSS feed maximum)
uv run python main.py channel subscribe "Channel Name" "@handle" CHANNEL_ID

# Subscribe with custom video count (max 15 due to RSS feed limit)
uv run python main.py channel subscribe "Channel Name" "@handle" CHANNEL_ID --recent-count 15

# Subscribe with auto-processing enabled
uv run python main.py channel subscribe "Channel Name" "@handle" CHANNEL_ID --auto-process
```

### Subscription Process

1. **Adds the channel** to your monitoring configuration
2. **Downloads recent videos** from the RSS feed (up to 20)
3. **Sets up monitoring** for future videos
4. **Updates tracking** to prevent re-downloading

### Finding Channel IDs

To subscribe to a channel, you need its YouTube channel ID:
1. Visit the channel's YouTube page
2. View page source (Ctrl+U / Cmd+U)
3. Search for "channelId" or "UC" followed by alphanumeric characters
4. The channel ID typically starts with "UC" and is 24 characters long

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