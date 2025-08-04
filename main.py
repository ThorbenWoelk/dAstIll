#!/usr/bin/env python3
import argparse
import re
import shutil
import signal
import sys
import time

from config.channel_config import ChannelConfigManager
from config.config import Config
from src.monitoring_service import ChannelMonitoringService
from src.rss_monitor import RSSChannelMonitor
from src.transcript_loader import RateLimitError, YouTubeTranscriptLoader


def check_disk_space(path: str, required_mb: int = 100) -> bool:
    """Check if there's enough disk space for downloads."""
    try:
        _, _, free_bytes = shutil.disk_usage(path)
        free_mb = free_bytes / (1024 * 1024)
        return free_mb >= required_mb
    except Exception:
        # If we can't check disk space, assume it's available
        return True


def validate_channel_id(channel_id: str) -> bool:
    """Validate YouTube channel ID format."""
    if not channel_id:
        return False

    # YouTube channel IDs start with UC and are 24 characters long
    # Or can be legacy usernames/custom names (more flexible)
    if len(channel_id) == 24 and channel_id.startswith("UC"):
        # Standard channel ID format
        return re.match(r"^UC[a-zA-Z0-9_-]{22}$", channel_id) is not None
    elif 1 <= len(channel_id) <= 100:
        # Legacy username or custom channel name
        return re.match(r"^[a-zA-Z0-9_.-]+$", channel_id) is not None

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
        "video_ids", nargs="+", help="Video IDs to mark as processed"
    )
    process_parser.add_argument(
        "--channel", help="Override channel name for processed files"
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
    add_channel_parser.add_argument("channel_id", help="YouTube channel ID (required)")
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
    subscribe_parser.add_argument("channel_id", help="YouTube channel ID (required)")
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
        help="Number of recent videos to download (default: 15, max: 20)",
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

    args = parser.parse_args()

    # Require explicit command
    if args.command is None:
        parser.print_help()
        sys.exit(1)

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

    for video_input in args.video_ids:
        video_id = (
            loader._extract_video_id(video_input)
            if "youtube.com" in video_input or "youtu.be" in video_input
            else video_input
        )

        try:
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
        # Validate channel ID format
        if not validate_channel_id(args.channel_id):
            print(f"❌ Invalid channel ID format: {args.channel_id}")
            print(
                "Channel ID should be 24 characters starting with 'UC' or a valid username"
            )
            return

        success = config_manager.add_channel(
            name=args.name,
            handle=args.handle,
            channel_id=args.channel_id,
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
        # Validate channel ID format
        if not validate_channel_id(args.channel_id):
            print(f"❌ Invalid channel ID format: {args.channel_id}")
            print(
                "Channel ID should be 24 characters starting with 'UC' or a valid username"
            )
            return

        # First add the channel
        success = config_manager.add_channel(
            name=args.name,
            handle=args.handle,
            channel_id=args.channel_id,
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

        # Download recent videos
        config = Config()
        max_videos = config.get("monitoring.max_recent_videos", 20)
        recent_count = min(args.recent_count, max_videos)
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

        for i, video in enumerate(videos, 1):
            print(f"\n[{i}/{len(videos)}] Processing: {video.title}")

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
                else:
                    print("   ✅ Downloaded successfully")
                    downloaded_count += 1

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
                break
            except Exception as e:
                print(f"   ❌ Error: {str(e)}")

        # Update the last video ID to prevent re-downloading on next monitor check
        if videos:
            config_manager.update_last_video_id(args.handle, videos[0].video_id)

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


if __name__ == "__main__":
    main()
