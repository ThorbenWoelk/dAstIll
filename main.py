#!/usr/bin/env python3
import argparse
import re
import shutil
import signal
import sys
import time
from pathlib import Path

from dotenv import load_dotenv

# Load .env file early to ensure environment variables are available
load_dotenv()

from config.channel_config import ChannelConfigManager  # noqa: E402
from config.config import Config  # noqa: E402
from src.claude_integration import ClaudeCodeIntegration  # noqa: E402
from src.monitoring_service import ChannelMonitoringService  # noqa: E402
from src.ollama_processor import OllamaTranscriptProcessor  # noqa: E402
from src.rss_monitor import RSSChannelMonitor  # noqa: E402
from src.transcript_loader import RateLimitError, YouTubeTranscriptLoader  # noqa: E402


def check_disk_space(path: str, required_mb: int = 100) -> bool:
    """Check if there's enough disk space for downloads."""
    try:
        _, _, free_bytes = shutil.disk_usage(path)
        free_mb = free_bytes / (1024 * 1024)
        return free_mb >= required_mb
    except Exception as e:
        # If we can't check disk space, warn but allow operation to continue
        print(
            f"⚠️ Warning: Could not check disk space ({str(e)}). Proceeding with download."
        )
        print("   💡 Monitor disk usage manually to avoid out-of-space errors.")
        return True


def validate_handle_format(handle: str) -> tuple[bool, str]:
    """Validate YouTube channel handle format.

    Args:
        handle: Channel handle to validate

    Returns:
        Tuple of (is_valid, error_message)
    """
    import string

    if not handle or not isinstance(handle, str):
        return False, "Handle cannot be empty"

    clean_handle = handle.strip().lstrip("@")
    allowed_chars = string.ascii_letters + string.digits + "_-"

    if not clean_handle:
        return False, "Handle cannot be empty after removing @ symbol"

    if not all(c in allowed_chars for c in clean_handle):
        return (
            False,
            "Handles can only contain letters, numbers, underscores, and dashes",
        )

    return True, ""


def validate_channel_id(channel_id: str) -> bool:
    """Validate YouTube channel ID format."""
    if not channel_id:
        return False

    # Check for Windows reserved names (case-insensitive)
    reserved_names = {
        "CON",
        "PRN",
        "AUX",
        "NUL",
        "COM1",
        "COM2",
        "COM3",
        "COM4",
        "COM5",
        "COM6",
        "COM7",
        "COM8",
        "COM9",
        "LPT1",
        "LPT2",
        "LPT3",
        "LPT4",
        "LPT5",
        "LPT6",
        "LPT7",
        "LPT8",
        "LPT9",
    }
    if channel_id.upper() in reserved_names:
        return False

    # YouTube channel IDs start with UC and are 24 characters long
    if len(channel_id) == 24 and channel_id.startswith("UC"):
        # Standard channel ID format - only alphanumeric, underscore, hyphen
        return re.match(r"^UC[a-zA-Z0-9_-]{22}$", channel_id) is not None

    # Legacy username or custom channel name - check for UC prefix conflicts first
    if channel_id.startswith("UC"):
        # If it starts with UC but isn't 24 chars, reject it to avoid confusion
        return False

    # Single character names (just alphanumeric)
    if len(channel_id) == 1:
        return channel_id.isalnum()

    # Multi-character legacy names (3-30 chars for security, but not 24 chars to avoid UC confusion)
    if 3 <= len(channel_id) <= 30 and len(channel_id) != 24:
        # Legacy username or custom channel name - strict for file system safety
        # Only allow alphanumeric and underscore - no periods to prevent hidden files
        if re.match(r"^[a-zA-Z0-9][a-zA-Z0-9_]*[a-zA-Z0-9]$", channel_id):
            return True

    return False


def main():
    parser = argparse.ArgumentParser(description="dAstIll - YouTube Transcript Loader")

    # Add subcommands
    subparsers = parser.add_subparsers(dest="command", help="Available commands")

    # Download command (default behavior)
    download_parser = subparsers.add_parser(
        "download", help="Download transcript for a video"
    )
    download_parser.add_argument("url", help="YouTube video URL or ID")
    download_parser.add_argument(
        "-l",
        "--languages",
        nargs="+",
        help="Preferred languages for transcript (default: from config)",
    )
    download_parser.add_argument(
        "-o", "--output", help="Output file path to save transcript"
    )
    download_parser.add_argument(
        "--raw",
        action="store_true",
        help="Output raw transcript instead of cleaned version",
    )
    download_parser.add_argument(
        "--force",
        action="store_true",
        help="Force download even if video already processed",
    )
    download_parser.add_argument(
        "--no-markdown", action="store_true", help="Disable markdown storage"
    )
    download_parser.add_argument(
        "--channel",
        default="unknown",
        help="Channel name for organizing processed files (default: unknown)",
    )

    # List command
    list_parser = subparsers.add_parser("list", help="List processed videos")
    list_parser.add_argument("--stats", action="store_true", help="Show statistics")

    # Info command
    info_parser = subparsers.add_parser("info", help="Show info for a specific video")
    info_parser.add_argument("video_id", help="Video ID to get info for")

    # Remove command
    remove_parser = subparsers.add_parser("remove", help="Remove a video from tracking")
    remove_parser.add_argument("video_id", help="Video ID to remove")
    remove_parser.add_argument(
        "--delete-file", action="store_true", help="Also delete the transcript file"
    )

    # Config command
    subparsers.add_parser("config", help="Show current configuration")

    # AI processing status command
    subparsers.add_parser("ai-status", help="Check Claude Code integration status")

    # Ollama processing command
    ollama_parser = subparsers.add_parser(
        "ollama", help="Process transcripts with local Ollama models"
    )
    ollama_subparsers = ollama_parser.add_subparsers(
        dest="ollama_action", help="Ollama actions"
    )

    # Ollama process
    process_ollama_parser = ollama_subparsers.add_parser(
        "process", help="Process downloaded transcripts with Ollama"
    )
    process_ollama_parser.add_argument(
        "--model", default="qwen3:8b", help="Ollama model to use (default: qwen3:8b)"
    )
    process_ollama_parser.add_argument(
        "--directory", help="Directory to process (defaults to downloaded folder)"
    )

    # Ollama status
    ollama_subparsers.add_parser("status", help="Check Ollama availability and models")

    # AI workflow command
    ai_workflow_parser = subparsers.add_parser(
        "ai-workflow", help="AI workflow automation"
    )
    ai_workflow_subparsers = ai_workflow_parser.add_subparsers(
        dest="workflow_action", help="AI workflow actions"
    )

    # AI workflow start
    ai_workflow_subparsers.add_parser(
        "start", help="Start the full AI workflow (monitoring + processing)"
    )

    # AI workflow process
    ai_workflow_subparsers.add_parser(
        "process", help="Only process downloaded transcripts with Claude Code"
    )

    # AI workflow status
    ai_workflow_subparsers.add_parser(
        "status", help="Check the status of Docker container and Claude Code"
    )

    # AI workflow stop
    ai_workflow_subparsers.add_parser("stop", help="Stop the Docker container")

    # Add command - Add video IDs to be downloaded later
    add_parser = subparsers.add_parser(
        "add", help="Add video IDs to be downloaded later"
    )
    add_parser.add_argument("video_ids", nargs="+", help="Video IDs or URLs to add")
    add_parser.add_argument("--title", help="Optional title for the video")
    add_parser.add_argument(
        "--channel",
        default="unknown",
        help="Channel name for organizing processed files (default: unknown)",
    )

    # Status command - Update video status
    status_parser = subparsers.add_parser("status", help="Update video status")
    status_parser.add_argument("video_id", help="Video ID to update")
    status_parser.add_argument(
        "new_status",
        choices=["to_be_downloaded", "downloaded", "processed"],
        help="New status for the video",
    )

    # Process command - Move videos from downloaded to processed
    process_parser = subparsers.add_parser("process", help="Mark videos as processed")
    process_parser.add_argument(
        "video_ids",
        nargs="*",
        help="Video IDs to mark as processed (if empty, processes all downloaded videos)",
    )
    process_parser.add_argument(
        "--channel", help="Override channel name for processed files"
    )
    process_parser.add_argument(
        "--with-ai",
        action="store_true",
        help="Process transcripts with Claude Code AI before organizing",
    )

    # Queue command - Show videos in different statuses
    queue_parser = subparsers.add_parser("queue", help="Show videos by status")
    queue_parser.add_argument(
        "--status",
        choices=["to_be_downloaded", "downloaded", "processed"],
        help="Filter by specific status",
    )

    # Monitor command - Channel monitoring functionality
    monitor_parser = subparsers.add_parser(
        "monitor", help="Channel monitoring commands"
    )
    monitor_subparsers = monitor_parser.add_subparsers(
        dest="monitor_action", help="Monitor actions"
    )

    # Monitor start
    monitor_subparsers.add_parser("start", help="Start monitoring channels")

    # Monitor stop
    monitor_subparsers.add_parser("stop", help="Stop monitoring (if running)")

    # Monitor status
    monitor_subparsers.add_parser("status", help="Show monitoring status")

    # Monitor test
    monitor_subparsers.add_parser("test", help="Test monitoring configuration")

    # Monitor check
    monitor_subparsers.add_parser("check", help="Manually check all channels once")

    # Channel management commands
    channel_parser = subparsers.add_parser(
        "channel", help="Channel management commands"
    )
    channel_subparsers = channel_parser.add_subparsers(
        dest="channel_action", help="Channel actions"
    )

    # Add channel
    add_channel_parser = channel_subparsers.add_parser(
        "add", help="Add a channel to monitor"
    )
    add_channel_parser.add_argument("name", help="Channel display name")
    add_channel_parser.add_argument("handle", help="Channel handle (e.g., @username)")
    add_channel_parser.add_argument(
        "channel_id",
        nargs="?",
        help="YouTube channel ID (optional - will be resolved from handle if not provided)",
    )
    add_channel_parser.add_argument(
        "--languages",
        nargs="+",
        default=["en"],
        help="Preferred transcript languages (default: en)",
    )
    add_channel_parser.add_argument(
        "--auto-download",
        action="store_true",
        default=True,
        help="Automatically download new videos (default: True)",
    )
    add_channel_parser.add_argument(
        "--auto-process",
        action="store_true",
        default=False,
        help="Automatically process to final location (default: False)",
    )

    # Subscribe to channel
    subscribe_parser = channel_subparsers.add_parser(
        "subscribe", help="Subscribe to a channel and download recent videos"
    )
    subscribe_parser.add_argument("name", help="Channel display name")
    subscribe_parser.add_argument("handle", help="Channel handle (e.g., @username)")
    subscribe_parser.add_argument(
        "channel_id",
        nargs="?",
        help="YouTube channel ID (optional - will be resolved from handle if not provided)",
    )
    subscribe_parser.add_argument(
        "--languages",
        nargs="+",
        default=["en"],
        help="Preferred transcript languages (default: en)",
    )
    subscribe_parser.add_argument(
        "--auto-download",
        action="store_true",
        default=True,
        help="Automatically download new videos (default: True)",
    )
    subscribe_parser.add_argument(
        "--auto-process",
        action="store_true",
        default=False,
        help="Automatically process to final location (default: False)",
    )
    subscribe_parser.add_argument(
        "--recent-count",
        type=int,
        default=15,
        help="Number of recent videos to download (default: 15, max: 15 due to RSS feed limit)",
    )

    # List channels
    list_channels_parser = channel_subparsers.add_parser(
        "list", help="List configured channels"
    )
    list_channels_parser.add_argument(
        "--enabled-only", action="store_true", help="Show only enabled channels"
    )

    # Remove channel
    remove_channel_parser = channel_subparsers.add_parser(
        "remove", help="Remove a channel"
    )
    remove_channel_parser.add_argument("handle", help="Channel handle to remove")

    # Enable/disable channel
    toggle_channel_parser = channel_subparsers.add_parser(
        "toggle", help="Enable or disable a channel"
    )
    toggle_channel_parser.add_argument("handle", help="Channel handle")
    toggle_channel_parser.add_argument(
        "--enable", action="store_true", help="Enable the channel"
    )
    toggle_channel_parser.add_argument(
        "--disable", action="store_true", help="Disable the channel"
    )

    # Settings command
    settings_parser = subparsers.add_parser("settings", help="Monitoring settings")
    settings_subparsers = settings_parser.add_subparsers(
        dest="settings_action", help="Settings actions"
    )

    # Enable/disable global monitoring
    settings_subparsers.add_parser("enable", help="Enable global monitoring")
    settings_subparsers.add_parser("disable", help="Disable global monitoring")

    # Set check interval
    interval_parser = settings_subparsers.add_parser(
        "interval", help="Set check interval"
    )
    interval_parser.add_argument(
        "seconds", type=int, help="Check interval in seconds (minimum: 60)"
    )

    # Set max videos limit
    limit_parser = settings_subparsers.add_parser(
        "limit", help="Set maximum videos per check"
    )
    limit_parser.add_argument(
        "count",
        type=str,
        help="Maximum videos per check (number or 'unlimited' for no limit)",
    )

    args = parser.parse_args()

    # Default to monitoring mode if no command provided
    if args.command is None:
        print("🚀 Starting dAstIll monitoring service...", flush=True)
        print("Use 'python main.py --help' to see available CLI commands", flush=True)

        # Start monitoring service directly
        global monitoring_service
        monitoring_service = ChannelMonitoringService()

        # Setup signal handlers for graceful shutdown
        signal.signal(signal.SIGINT, signal_handler)
        signal.signal(signal.SIGTERM, signal_handler)

        if monitoring_service.start_monitoring():
            print("✅ Monitoring started. Press Ctrl+C to stop.", flush=True)
            try:
                # Keep the main thread alive
                while monitoring_service.running:
                    time.sleep(1)
            except KeyboardInterrupt:
                print("\n🔄 Stopping monitoring...")
                monitoring_service.stop_monitoring()
        else:
            print("❌ Failed to start monitoring")
            sys.exit(1)
        return

    loader = YouTubeTranscriptLoader()

    try:
        if args.command == "download":
            handle_download(loader, args)
        elif args.command == "list":
            handle_list(loader, args)
        elif args.command == "info":
            handle_info(loader, args)
        elif args.command == "remove":
            handle_remove(loader, args)
        elif args.command == "config":
            handle_config(loader, args)
        elif args.command == "ai-status":
            handle_ai_status(args)
        elif args.command == "ollama":
            handle_ollama(loader, args)
        elif args.command == "add":
            handle_add(loader, args)
        elif args.command == "status":
            handle_status(loader, args)
        elif args.command == "process":
            handle_process(loader, args)
        elif args.command == "queue":
            handle_queue(loader, args)
        elif args.command == "monitor":
            handle_monitor(args)
        elif args.command == "channel":
            handle_channel(args)
        elif args.command == "settings":
            handle_settings(args)
        elif args.command == "ai-workflow":
            handle_ai_workflow(args)

    except Exception as e:
        print(f"Error: {str(e)}", file=sys.stderr)
        sys.exit(1)


def handle_download(loader, args):
    print(f"Loading transcript for: {args.url}")

    save_markdown = not args.no_markdown
    transcript_data = loader.load_transcript(
        args.url,
        args.languages,
        force=args.force,
        save_markdown=save_markdown,
        channel=args.channel,
    )

    if transcript_data.get("already_exists"):
        print("✓ Video already processed!")
        print(f"Video ID: {transcript_data.get('video_id', 'N/A')}")
        print(f"Language: {transcript_data.get('language', 'N/A')}")
        print(f"Auto-generated: {transcript_data.get('is_generated', 'N/A')}")
        print(f"File: {transcript_data.get('file_path', 'N/A')}")
        print(f"Status: {transcript_data.get('status', 'N/A')}")
        print(f"Channel: {transcript_data.get('channel', 'N/A')}")
        return

    print("✓ Transcript loaded successfully!")
    print(f"Video ID: {transcript_data.get('video_id', 'N/A')}")
    print(f"Language: {transcript_data.get('language', 'N/A')}")
    print(f"Auto-generated: {transcript_data.get('is_generated', 'N/A')}")

    if transcript_data.get("file_path"):
        print(f"Saved to: {transcript_data['file_path']}")

    if args.output:
        loader.save_transcript(transcript_data, args.output)
        print(f"Also saved to: {args.output}")

    if not args.output and not transcript_data.get("file_path"):
        print("\nTranscript:")
        print("-" * 50)
        if args.raw:
            print(transcript_data["formatted_text"])
        else:
            print(transcript_data["cleaned_text"])


def handle_list(loader, args):
    if args.stats:
        stats = loader.get_stats()
        print("Statistics:")
        print(f"Total videos: {stats['total']}")
        print(f"To be downloaded: {stats['to_be_downloaded']}")
        print(f"Downloaded: {stats['downloaded']}")
        print(f"Processed: {stats['processed']}")
        print(f"\nChannels: {stats['channels']}")
    else:
        videos = loader.list_processed_videos()
        if not videos:
            print("No videos found.")
            return

        print(f"All videos ({len(videos)}):")
        print("-" * 80)
        for video in videos:
            print(f"ID: {video['video_id']}")
            print(f"Status: {video['status']}")
            print(f"Channel: {video['channel']}")
            if video.get("file_path"):
                print(f"File: {video['file_path']}")
            print("-" * 40)


def handle_info(loader, args):
    info = loader.get_video_info(args.video_id)
    if not info:
        print(f"No information found for video ID: {args.video_id}")
        return

    print("Video Information:")
    print(f"ID: {info.get('video_id', 'N/A')}")
    print(f"Status: {info.get('status', 'unknown')}")
    print(f"Channel: {info.get('channel', 'unknown')}")
    print(f"File: {info.get('file_path', 'N/A')}")

    # Note: In stateless mode, language/generation info is only available
    # when reading from actual transcript files, not from file system metadata
    print(f"Language: {info.get('language', 'Check file for details')}")
    print(f"Auto-generated: {info.get('is_generated', 'Check file for details')}")

    metadata = info.get("metadata", {})
    if metadata:
        print(f"Languages requested: {metadata.get('languages_requested', 'N/A')}")
        print(f"File size: {metadata.get('file_size', 'N/A')} bytes")


def handle_remove(loader, args):
    result = loader.remove_video(args.video_id, delete_file=args.delete_file)

    if result["found"]:
        print(
            f"✓ Video {args.video_id} found (status: {result.get('previous_status', 'unknown')})"
        )

        if args.delete_file:
            if result["file_deleted"]:
                print("✓ Associated file deleted successfully")
            elif result["error"]:
                print(f"⚠ Warning: {result['error']}")
            else:
                print("⚠ File deletion was requested but not performed")
        else:
            print("Note: File was not deleted (use --delete-file to remove file)")
    else:
        print(f"Video {args.video_id} not found")


def handle_config(loader, args):
    config = loader.config.config
    print("Current Configuration:")
    print("=" * 50)

    def print_dict(d, indent=0):
        for key, value in d.items():
            if isinstance(value, dict):
                print("  " * indent + f"{key}:")
                print_dict(value, indent + 1)
            else:
                print("  " * indent + f"{key}: {value}")

    print_dict(config)


def handle_ai_status(args):
    """Handle AI processing status command."""
    claude_integration = ClaudeCodeIntegration()
    status = claude_integration.get_status()

    print("Claude Code Integration Status:")
    print("=" * 50)

    if status["available"]:
        print("✅ Claude Code CLI: Available")
        print(f"   Path: {status['claude_path']}")

        if status["authenticated"]:
            print("✅ Authentication: Successful")
        else:
            print("❌ Authentication: Failed")

        print(f"   Status: {status['message']}")
    else:
        print("❌ Claude Code CLI: Not Available")
        print(f"   Message: {status['message']}")
        print("\nTo enable AI processing:")
        print("1. Install Claude Code: Visit https://claude.ai/code")
        print("2. Run: claude setup-token")
        print("3. Verify with: dastill ai-status")


def handle_add(loader, args):
    added_count = 0

    for video_input in args.video_ids:
        # Extract video ID from URL if needed
        video_id = (
            loader._extract_video_id(video_input)
            if "youtube.com" in video_input or "youtu.be" in video_input
            else video_input
        )

        if loader.add_to_be_downloaded(video_id, args.channel):
            print(f"✓ Added {video_id} to download queue (channel: {args.channel})")
            added_count += 1
        else:
            print(f"⚠ Video {video_id} already in system")

    print(f"\nAdded {added_count} new video(s) to download queue")


def handle_status(loader, args):
    video_id = (
        loader._extract_video_id(args.video_id)
        if "youtube.com" in args.video_id or "youtu.be" in args.video_id
        else args.video_id
    )

    current_status, file_path = loader.manager.get_video_status(video_id)

    if current_status == "not_downloaded":
        print(f"Video {video_id} not found")
        return

    print(f"Current status of {video_id}: {current_status}")
    print(
        "Note: In stateless mode, status changes are done by moving files between folders:"
    )
    print("  - to_be_downloaded: /to_be_downloaded/")
    print("  - downloaded: /downloaded/")
    print("  - processed: /[channel-name]/")
    print("Use the 'process' command to move from downloaded to processed.")

    if file_path:
        print(f"Current file: {file_path}")


def handle_process(loader, args):
    processed_count = 0
    claude_integration = None

    # Initialize Claude Code integration if --with-ai flag is provided
    with_ai_flag = getattr(args, "with_ai", False)
    if with_ai_flag:
        claude_integration = ClaudeCodeIntegration()
        if not claude_integration.is_available():
            print(
                "❌ Claude Code integration not available. Use 'dastill ai-status' to check."
            )
            print("❌ --with-ai flag requires Claude Code to be available.")
            print("💡 Either install Claude Code SDK or run without --with-ai flag.")
            return
        else:
            is_auth, auth_msg = claude_integration.check_authentication()
            if not is_auth:
                print(f"❌ Claude Code authentication failed: {auth_msg}")
                print("❌ --with-ai flag requires authenticated Claude Code session.")
                print(
                    "💡 Run this command from within Claude Code or remove --with-ai flag."
                )
                return
            else:
                print("✅ Claude Code integration ready. Processing with AI...")

    # If no video IDs provided, process all downloaded videos
    if not args.video_ids:
        downloaded_videos = loader.manager.list_videos_by_status("downloaded")
        if not downloaded_videos:
            print("No videos found in downloaded folder.")
            return

        print(f"Processing all {len(downloaded_videos)} downloaded videos...")
        video_ids_to_process = [video["video_id"] for video in downloaded_videos]
    else:
        video_ids_to_process = args.video_ids

    for video_input in video_ids_to_process:
        video_id = (
            loader._extract_video_id(video_input)
            if "youtube.com" in video_input or "youtu.be" in video_input
            else video_input
        )

        try:
            # AI processing step if enabled
            if claude_integration:
                # Get the file path for the downloaded video
                status, file_path = loader.manager.get_video_status(video_id)
                if status == "downloaded" and file_path:
                    print(f"🤖 Processing {video_id} with AI...")
                    ai_success, ai_message, ai_content = (
                        claude_integration.process_transcript(Path(file_path))
                    )

                    if ai_success and ai_content:
                        # Update the file with AI-processed content
                        if claude_integration.update_transcript_file(
                            Path(file_path), ai_content
                        ):
                            print(f"✓ AI processing complete for {video_id}")
                        else:
                            print(
                                f"⚠ Failed to update file with AI content for {video_id}"
                            )
                    else:
                        print(f"⚠ AI processing failed for {video_id}: {ai_message}")
                else:
                    print(f"⚠ Cannot find downloaded file for {video_id}")

            # Regular processing step
            success, result = loader.process_video(video_id, args.channel)
            if success:
                print(f"✓ Processed {video_id}")
                print(f"  File moved to: {result}")
                processed_count += 1
            else:
                print(f"⚠ {result}")
        except Exception as e:
            print(f"Error processing {video_id}: {str(e)}")

    print(f"\nProcessed {processed_count} video(s)")


def handle_queue(loader, args):
    if args.status:
        videos = loader.manager.list_videos_by_status(args.status)
        print(f"Videos with status '{args.status}' ({len(videos)}):")
    else:
        # Show all statuses
        stats = loader.get_stats()
        print("Video Queue Overview:")
        print(f"Total videos: {stats['total']}")
        print(f"  to_be_downloaded: {stats['to_be_downloaded']}")
        print(f"  downloaded: {stats['downloaded']}")
        print(f"  processed: {stats['processed']}")
        print(f"\nChannels: {stats['channels']}")
        print("\nAll videos:")
        videos = loader.list_processed_videos()

    if not videos:
        print("No videos found.")
        return

    print("-" * 80)
    for video in videos:
        print(f"ID: {video['video_id']}")
        print(f"Status: {video['status']}")
        print(f"Channel: {video['channel']}")
        if video.get("file_path"):
            print(f"File: {video['file_path']}")
        print("-" * 40)


# Global monitoring service instance for signal handling
monitoring_service = None


def signal_handler(signum, frame):
    """Handle shutdown signals for monitoring service."""
    global monitoring_service
    if monitoring_service and monitoring_service.running:
        print(f"\n🔄 Received signal {signum}, stopping monitoring...")
        monitoring_service.stop_monitoring()
    sys.exit(0)


def handle_monitor(args):
    """Handle monitor subcommands."""
    global monitoring_service

    if args.monitor_action is None:
        print(
            "Error: No monitor action specified. Use 'monitor start', 'monitor status', etc."
        )
        return

    monitoring_service = ChannelMonitoringService()

    if args.monitor_action == "start":
        # Setup signal handlers for graceful shutdown
        signal.signal(signal.SIGINT, signal_handler)
        signal.signal(signal.SIGTERM, signal_handler)

        print("🚀 Starting monitoring service...")
        if monitoring_service.start_monitoring():
            print("✅ Monitoring started. Press Ctrl+C to stop.")
            try:
                # Keep the main thread alive
                while monitoring_service.running:
                    time.sleep(1)
            except KeyboardInterrupt:
                print("\n🔄 Stopping monitoring...")
                monitoring_service.stop_monitoring()
        else:
            print("❌ Failed to start monitoring")

    elif args.monitor_action == "stop":
        print("⚠️ This command stops any monitoring process that might be running.")
        print(
            "Note: If monitoring is running in another terminal, you'll need to stop it there."
        )

    elif args.monitor_action == "status":
        status = monitoring_service.get_monitoring_status()
        print("📊 Monitoring Status:")
        print("=" * 60)
        print(f"Global monitoring enabled: {status['global_enabled']}")
        print(f"Currently active: {status['monitoring_active']}")
        print(f"Check interval: {status['check_interval']} seconds")

        stats = status["stats"]
        print("\nChannel statistics:")
        print(f"  Total channels: {stats['total_channels']}")
        print(f"  Enabled channels: {stats['enabled_channels']}")
        print(f"  Channels with IDs: {stats['channels_with_ids']}")
        print(f"  Channels with last video: {stats['channels_with_last_video']}")

        if status["channels"]:
            print("\nConfigured channels:")
            for ch in status["channels"]:
                status_icons = []
                if ch["enabled"]:
                    status_icons.append("✅")
                else:
                    status_icons.append("❌")
                if ch["has_channel_id"]:
                    status_icons.append("🆔")
                if ch["auto_download"]:
                    status_icons.append("⬇️")
                if ch["auto_process"]:
                    status_icons.append("⚡")

                print(f"  {ch['name']} ({ch['handle']}) {' '.join(status_icons)}")

    elif args.monitor_action == "test":
        print("🧪 Testing monitoring configuration...")
        if monitoring_service.test_configuration():
            print("✅ All tests passed")
        else:
            print("❌ Some tests failed")

    elif args.monitor_action == "check":
        print("🔍 Manually checking all channels...")
        result = monitoring_service.check_now()
        if result["status"] == "success":
            print("✅ Manual check completed")
        else:
            print(f"❌ Manual check failed: {result.get('message', 'Unknown error')}")


def handle_channel(args):
    """Handle channel management subcommands."""
    if args.channel_action is None:
        print(
            "Error: No channel action specified. Use 'channel add', 'channel list', etc."
        )
        return

    config_manager = ChannelConfigManager()

    if args.channel_action == "add":
        # Resolve channel ID if not provided
        channel_id = args.channel_id
        if not channel_id:
            print(f"🔍 Resolving channel ID for handle: {args.handle}")
            monitor = RSSChannelMonitor()

            # Validate handle format first
            is_valid, error_message = validate_handle_format(args.handle)
            if not is_valid:
                print(f"❌ Invalid handle format: {args.handle}")
                print(error_message)
                print("Example: @channelname or channelname")
                return

            try:
                channel_id = monitor.resolve_channel_id_from_handle(args.handle)
                if not channel_id:
                    print(f"❌ Could not resolve channel ID for handle: {args.handle}")
                    print("\nPossible reasons:")
                    print("  • The channel handle might be incorrect")
                    print("  • The channel might not exist")
                    print("  • YouTube might be blocking automated requests")
                    print("\nYou can manually find the channel ID by:")
                    print("  1. Visit the channel page on YouTube")
                    print("  2. View page source (right-click → View Page Source)")
                    print('  3. Search for "channelId" or "externalId"')
                    print("  4. The ID will be a string starting with 'UC'")
                    print(
                        f'\nThen run: channel add "{args.name}" "{args.handle}" CHANNEL_ID'
                    )
                    return
                print(f"✅ Resolved channel ID: {channel_id}")
            except Exception as e:
                print(f"❌ Error resolving channel ID: {str(e)}")
                print("Please check your internet connection and try again.")
                return

        # Validate channel ID format
        if not validate_channel_id(channel_id):
            print(f"❌ Invalid channel ID format: {channel_id}")
            print(
                "Channel ID should be 24 characters starting with 'UC' or a valid username"
            )
            return

        success = config_manager.add_channel(
            name=args.name,
            handle=args.handle,
            channel_id=channel_id,
            languages=args.languages,
            auto_download=args.auto_download,
            auto_process=args.auto_process,
        )

        if success:
            print(f"✅ Added channel: {args.name} ({args.handle})")
            print(f"   Languages: {', '.join(args.languages)}")
            print(f"   Auto-download: {args.auto_download}")
            print(f"   Auto-process: {args.auto_process}")
        else:
            print(f"❌ Channel {args.handle} already exists")

    elif args.channel_action == "list":
        channels = config_manager.list_channels(enabled_only=args.enabled_only)
        if not channels:
            filter_text = " (enabled only)" if args.enabled_only else ""
            print(f"No channels configured{filter_text}.")
            return

        filter_text = " (enabled only)" if args.enabled_only else ""
        print(f"Configured channels{filter_text} ({len(channels)} total):")
        print("=" * 80)

        for i, ch in enumerate(channels, 1):
            status = "✅ Enabled" if ch.monitoring.enabled else "❌ Disabled"
            print(f"{i}. {ch.name} ({ch.handle}) - {status}")
            print(f"   Channel ID: {ch.channel_id or 'Not resolved'}")
            print(f"   Languages: {', '.join(ch.monitoring.languages)}")
            print(f"   Auto-download: {ch.monitoring.auto_download}")
            print(f"   Auto-process: {ch.monitoring.auto_process}")
            print(f"   Last video: {ch.last_video_id or 'None'}")
            print()

    elif args.channel_action == "remove":
        if config_manager.remove_channel(args.handle):
            print(f"✅ Removed channel: {args.handle}")
        else:
            print(f"❌ Channel {args.handle} not found")

    elif args.channel_action == "toggle":
        if args.enable and args.disable:
            print("❌ Cannot specify both --enable and --disable")
            return
        elif args.enable:
            enabled = True
        elif args.disable:
            enabled = False
        else:
            # Toggle current state
            channel = config_manager.get_channel(args.handle)
            if not channel:
                print(f"❌ Channel {args.handle} not found")
                return
            enabled = not channel.monitoring.enabled

        if config_manager.enable_channel(args.handle, enabled):
            status = "enabled" if enabled else "disabled"
            print(f"✅ Channel {args.handle} {status}")
        else:
            print(f"❌ Channel {args.handle} not found")

    elif args.channel_action == "subscribe":
        # Resolve channel ID if not provided
        channel_id = args.channel_id
        if not channel_id:
            print(f"🔍 Resolving channel ID for handle: {args.handle}")
            monitor = RSSChannelMonitor()

            # Validate handle format first
            is_valid, error_message = validate_handle_format(args.handle)
            if not is_valid:
                print(f"❌ Invalid handle format: {args.handle}")
                print(error_message)
                print("Example: @channelname or channelname")
                return

            try:
                channel_id = monitor.resolve_channel_id_from_handle(args.handle)
                if not channel_id:
                    print(f"❌ Could not resolve channel ID for handle: {args.handle}")
                    print("\nPossible reasons:")
                    print("  • The channel handle might be incorrect")
                    print("  • The channel might not exist")
                    print("  • YouTube might be blocking automated requests")
                    print("\nYou can manually find the channel ID by:")
                    print("  1. Visit the channel page on YouTube")
                    print("  2. View page source (right-click → View Page Source)")
                    print('  3. Search for "channelId" or "externalId"')
                    print("  4. The ID will be a string starting with 'UC'")
                    print(
                        f'\nThen run: channel subscribe "{args.name}" "{args.handle}" CHANNEL_ID'
                    )
                    return
                print(f"✅ Resolved channel ID: {channel_id}")
            except Exception as e:
                print(f"❌ Error resolving channel ID: {str(e)}")
                print("Please check your internet connection and try again.")
                return

        # Validate channel ID format
        if not validate_channel_id(channel_id):
            print(f"❌ Invalid channel ID format: {channel_id}")
            print(
                "Channel ID should be 24 characters starting with 'UC' or a valid username"
            )
            return

        # First add the channel
        success = config_manager.add_channel(
            name=args.name,
            handle=args.handle,
            channel_id=channel_id,
            languages=args.languages,
            auto_download=args.auto_download,
            auto_process=args.auto_process,
        )

        if not success:
            print(f"❌ Channel {args.handle} already exists")
            return

        print(f"✅ Added channel: {args.name} ({args.handle})")
        print(f"   Languages: {', '.join(args.languages)}")
        print(f"   Auto-download: {args.auto_download}")
        print(f"   Auto-process: {args.auto_process}")

        # Download recent videos (RSS feeds are limited to ~20 videos)
        config = Config()
        max_videos = config.get("monitoring.max_recent_videos", 15)
        recent_count = min(args.recent_count, max_videos)  # Respect config limit
        print(f"\n📥 Downloading recent {recent_count} videos...")

        # Check available disk space (estimate ~2MB per transcript)
        estimated_space_mb = recent_count * 2
        config_storage = config.get("storage.base_path", ".")
        if not check_disk_space(config_storage, estimated_space_mb):
            print(f"⚠️ Insufficient disk space for {recent_count} videos")
            print(f"   Estimated space needed: {estimated_space_mb}MB")
            print("   Please free up disk space and try again")
            return

        # Use RSS monitor to get recent videos
        rss_monitor = RSSChannelMonitor()

        videos = rss_monitor.get_latest_videos(args.channel_id, limit=recent_count)

        if not videos:
            print("⚠️ Could not fetch recent videos from RSS feed")
            print("   This could be due to:")
            print("   - Invalid channel ID")
            print("   - Network connectivity issues")
            print("   - Temporary YouTube API limitations")
            print(
                "   The RSS monitor already includes retry logic with exponential backoff"
            )
            return

        # Download each video's transcript
        loader = YouTubeTranscriptLoader()
        downloaded_count = 0
        successful_videos = []
        last_attempted_video = None
        rate_limit_hit = False

        for i, video in enumerate(videos, 1):
            print(f"\n[{i}/{len(videos)}] Processing: {video.title}")
            last_attempted_video = video  # Track the last video we attempted to process

            try:
                result = loader.load_transcript(
                    video.url,
                    languages=args.languages,
                    force=False,
                    save_markdown=True,
                    channel=args.name,
                )

                if result.get("already_exists"):
                    print("   ✓ Already downloaded")
                    successful_videos.append(video)
                else:
                    print("   ✅ Downloaded successfully")
                    downloaded_count += 1
                    successful_videos.append(video)

                    # Auto-process if enabled
                    if args.auto_process:
                        success, process_result = loader.process_video(
                            video.video_id, args.name
                        )
                        if success:
                            print(f"   ⚡ Auto-processed to: {process_result}")

            except RateLimitError as e:
                print(f"   ⚠️ Rate limit hit: {str(e)}")
                print("   ⏸️ Stopping here to avoid further rate limiting.")
                print(
                    "   💡 The monitoring service will handle rate limits by sleeping 3 hours."
                )
                rate_limit_hit = True
                break
            except Exception as e:
                print(f"   ❌ Error: {str(e)}")
                # Continue processing other videos even if one fails

        # Update last_video_id based on processing results
        if successful_videos and not rate_limit_hit:
            # All videos processed successfully - update to most recent successful
            last_successful_video = successful_videos[0]
            config_manager.update_last_video_id(
                args.handle, last_successful_video.video_id
            )
        elif successful_videos and rate_limit_hit:
            # Rate limit hit - update to last attempted video to ensure retry of failed video
            if last_attempted_video:
                config_manager.update_last_video_id(
                    args.handle, last_attempted_video.video_id
                )
                print(f"   📌 Will retry from: {last_attempted_video.title}")
        elif rate_limit_hit and videos:
            # Rate limit hit immediately - don't update last_video_id
            print(
                "   ⚠️ No videos processed due to immediate rate limit - will retry next time"
            )

        print(f"\n✅ Subscription complete! Downloaded {downloaded_count} new videos.")
        print("🔄 Future videos will be monitored automatically.")


def handle_settings(args):
    """Handle monitoring settings subcommands."""
    if args.settings_action is None:
        print(
            "Error: No settings action specified. Use 'settings enable', 'settings interval', etc."
        )
        return

    config_manager = ChannelConfigManager()

    if args.settings_action == "enable":
        config_manager.set_global_monitoring(True)
        print("✅ Global monitoring enabled")

    elif args.settings_action == "disable":
        config_manager.set_global_monitoring(False)
        print("✅ Global monitoring disabled")

    elif args.settings_action == "interval":
        if args.seconds < 60:
            print("❌ Minimum check interval is 60 seconds")
            return

        config_manager.set_check_interval(args.seconds)
        print(f"✅ Check interval set to {args.seconds} seconds")

    elif args.settings_action == "limit":
        if args.count.lower() == "unlimited":
            config_manager.global_config.max_videos_per_check = None
            config_manager.save_configuration()
            print("✅ Video processing limit set to unlimited")
            print(
                "⚠️  Warning: Unlimited processing may consume more memory and API quota"
            )
        else:
            try:
                count = int(args.count)
                if count < 1:
                    print("❌ Video limit must be at least 1")
                    return
                config_manager.global_config.max_videos_per_check = count
                config_manager.save_configuration()
                print(f"✅ Video processing limit set to {count} videos per check")
            except ValueError:
                print("❌ Invalid limit value. Use a number or 'unlimited'")
                return


def handle_ollama(loader, args):
    """Handle Ollama processing commands."""
    if args.ollama_action == "status":
        # Check Ollama status
        processor = OllamaTranscriptProcessor()
        available, status_message = processor.check_ollama_availability()

        print("Ollama Local AI Status:")
        print("=" * 50)

        if available:
            print("✅ Ollama: Available")
            print(f"   {status_message}")

            # Show available models using processor method
            models_info = processor.get_available_models()
            if models_info:
                print(f"   Available models: {len(models_info)}")
                for model in models_info[:5]:  # Show first 5 models
                    print(
                        f"      • {model['name']} ({model.get('size', 'unknown size')})"
                    )
                if len(models_info) > 5:
                    print(f"      ... and {len(models_info) - 5} more")
        else:
            print("❌ Ollama: Not Available")
            print(f"   {status_message}")
            print("   💡 Make sure Ollama is installed and running:")
            print("      - Install: https://ollama.ai")
            print("      - Start: ollama serve")
            print("      - Pull model: ollama pull qwen3:8b")

    elif args.ollama_action == "process":
        # Process transcripts with Ollama
        config = loader.config
        base_path = config.get("storage.base_path")

        if args.directory:
            process_dir = Path(args.directory)
        else:
            process_dir = Path(base_path) / "downloaded"

        print(f"Processing transcripts with Ollama model: {args.model}")
        print(f"Directory: {process_dir}")
        print()

        # Check if Ollama is available
        processor = OllamaTranscriptProcessor(
            model_name=args.model,
            template_path=str(Path(base_path) / "TRANSCRIPT_TEMPLATE.md"),
        )

        available, status_message = processor.check_ollama_availability()
        if not available:
            print(f"❌ {status_message}")
            print("Please ensure Ollama is running and the model is available.")
            return

        print(f"✅ {status_message}")
        print()

        # Process transcripts
        success_count, total_count, failed_files = processor.process_directory(
            process_dir
        )

        print()
        print("Processing Summary:")
        print(f"  Successfully processed: {success_count}/{total_count} files")

        if failed_files:
            print(f"  Failed files: {', '.join(failed_files)}")

        if success_count > 0:
            print()
            print("✅ Local AI processing completed successfully!")
            print(
                "💡 Run 'uv run python main.py process' to organize enhanced transcripts"
            )
        else:
            print("ℹ️  No files were processed")

    else:
        print("Error: No Ollama action specified.")
        print("Available actions: status, process")


def handle_ai_workflow(args):
    """Handle AI workflow automation subcommands."""
    import subprocess
    from pathlib import Path

    if args.workflow_action is None:
        print(
            "Error: No workflow action specified. Use 'ai-workflow start', 'ai-workflow status', etc."
        )
        return

    # Get the script path
    script_path = Path(__file__).parent / "scripts" / "ai-workflow.sh"

    if not script_path.exists():
        print(f"❌ AI workflow script not found at: {script_path}")
        print("Please ensure the script is properly installed.")
        return

    try:
        # Execute the bash script with the appropriate action and stream output
        cmd = [str(script_path), args.workflow_action]

        # Use Popen to stream output in real-time
        process = subprocess.Popen(
            cmd,
            stdout=subprocess.PIPE,
            stderr=subprocess.STDOUT,
            universal_newlines=True,
            bufsize=1,
        )

        # Stream output line by line with proper resource cleanup
        try:
            while True:
                output = process.stdout.readline()
                if output == "" and process.poll() is not None:
                    break
                if output:
                    print(output.strip())
        except KeyboardInterrupt:
            print("\n⚠️ Workflow interrupted by user")
            process.terminate()
            try:
                process.wait(timeout=10)
            except subprocess.TimeoutExpired:
                process.kill()
                process.wait()
            sys.exit(1)
        finally:
            # Ensure proper cleanup of resources
            if process.stdout:
                process.stdout.close()
            if process.poll() is None:
                process.terminate()
                try:
                    process.wait(timeout=5)
                except subprocess.TimeoutExpired:
                    process.kill()
                    process.wait()

        # Wait for process to complete and get return code
        return_code = process.poll()

        # Ensure stdout is closed in normal execution path
        if process.stdout:
            process.stdout.close()

        if return_code != 0:
            print(f"\n❌ AI workflow exited with code {return_code}")

        sys.exit(return_code)

    except Exception as e:
        print(f"❌ Error executing AI workflow: {str(e)}")
        sys.exit(1)


if __name__ == "__main__":
    main()
