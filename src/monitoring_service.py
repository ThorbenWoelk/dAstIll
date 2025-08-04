"""YouTube channel monitoring service using RSS feeds."""

import threading
import time
from collections.abc import Callable
from datetime import datetime
from typing import Any

from config.channel_config import ChannelConfig, ChannelConfigManager

from .rss_monitor import RSSChannelMonitor, VideoInfo
from .transcript_loader import YouTubeTranscriptLoader


class ChannelMonitoringService:
    """Service for monitoring YouTube channels and automatically processing new videos."""

    def __init__(self, config_dir: str | None = None, base_path: str | None = None):
        self.config_manager = ChannelConfigManager(config_dir)
        self.rss_monitor = RSSChannelMonitor()

        # Initialize transcript loader for processing new videos
        self.transcript_loader = YouTubeTranscriptLoader()

        # Monitoring state
        self.running = False
        self.monitor_thread = None

        # Event callbacks
        self.on_new_video: Callable[[VideoInfo, ChannelConfig], None] | None = None
        self.on_video_processed: (
            Callable[[VideoInfo, ChannelConfig, dict], None] | None
        ) = None
        self.on_error: Callable[[str, Exception], None] | None = None
        self.on_status: Callable[[str], None] | None = None

    def test_configuration(self) -> bool:
        """Test monitoring configuration and RSS connectivity."""
        self._log_status("🧪 Testing monitoring configuration...")

        enabled_channels = self.config_manager.get_enabled_channels()
        if not enabled_channels:
            self._log_status("⚠️ No enabled channels to test")
            return True

        all_good = True

        for channel in enabled_channels:
            self._log_status(f"Testing {channel.name} ({channel.handle})...")

            if not channel.channel_id:
                self._log_status("   ❌ Missing channel ID - please add manually")
                all_good = False
                continue

            # Test RSS feed
            if self.rss_monitor.test_rss_feed(channel.channel_id):
                self._log_status("   ✅ RSS feed accessible")

                # Test video fetching
                videos = self.rss_monitor.get_latest_videos(channel.channel_id, limit=1)
                if videos:
                    video = videos[0]
                    self._log_status(f"   ✅ Latest video: {video.title}")
                else:
                    self._log_status("   ⚠️ No videos found in RSS feed")
            else:
                self._log_status("   ❌ RSS feed not accessible")
                all_good = False

        return all_good

    def start_monitoring(self) -> bool:
        """Start the monitoring service."""
        if self.running:
            self._log_status("Monitoring already running")
            return False

        if not self.config_manager.global_config.enabled:
            self._log_status("Global monitoring is disabled")
            return False

        enabled_channels = self.config_manager.get_enabled_channels()
        if not enabled_channels:
            self._log_status("No enabled channels to monitor")
            return False

        # Verify all enabled channels have channel IDs (now required manually)
        missing_ids = [ch for ch in enabled_channels if not ch.channel_id]
        if missing_ids:
            handles = [ch.handle for ch in missing_ids]
            self._log_error(f"Missing channel IDs for: {handles}", None)
            self._log_error(
                "Please provide channel IDs manually using the 'channel add' command",
                None,
            )
            return False

        self.running = True
        self.monitor_thread = threading.Thread(
            target=self._monitoring_loop, daemon=True
        )
        self.monitor_thread.start()

        self._log_status(
            f"✅ Started RSS monitoring for {len(enabled_channels)} channels"
        )
        for ch in enabled_channels:
            self._log_status(f"   - {ch.name} ({ch.handle})")
        self._log_status(
            f"Check interval: {self.config_manager.global_config.check_interval} seconds"
        )

        return True

    def stop_monitoring(self):
        """Stop the monitoring service."""
        if not self.running:
            return

        self.running = False
        if self.monitor_thread:
            self.monitor_thread.join(timeout=10)
        self._log_status("⏹️ Monitoring stopped")

    def _monitoring_loop(self):
        """Main monitoring loop that runs in a separate thread."""
        self._log_status("🔄 Monitoring loop started")

        while self.running:
            try:
                self._check_all_channels()
                time.sleep(self.config_manager.global_config.check_interval)
            except Exception as e:
                self._log_error("Error in monitoring loop", e)
                time.sleep(60)  # Wait a minute before retrying on error

    def _check_all_channels(self):
        """Check all enabled channels for new videos."""
        enabled_channels = self.config_manager.get_enabled_channels()

        self._log_status(
            f"🔍 Checking {len(enabled_channels)} channels for new videos..."
        )

        for channel in enabled_channels:
            try:
                self._check_channel(channel)
            except Exception as e:
                self._log_error(f"Error checking channel {channel.handle}", e)

    def _check_channel(self, channel: ChannelConfig):
        """Check a single channel for new videos."""
        if not channel.channel_id:
            return

        # Get latest videos from RSS
        max_videos = self.config_manager.global_config.max_videos_per_check
        videos = self.rss_monitor.get_latest_videos(
            channel.channel_id, limit=max_videos
        )

        if not videos:
            return

        # Determine which videos are new
        new_videos = []

        if channel.last_video_id is None:
            # First time checking this channel - only process the latest video
            new_videos = [videos[0]]
            self._log_status(f"🆕 First check for {channel.name} - found latest video")
        else:
            # Find videos newer than last processed video
            for video in videos:
                if video.video_id == channel.last_video_id:
                    break
                new_videos.append(video)

        if not new_videos:
            return

        # Process new videos (oldest first to maintain chronological order)
        for video in reversed(new_videos):
            self._log_status(f"🎥 New video detected: {video.title} ({channel.name})")

            # Update last video ID immediately to avoid reprocessing
            self.config_manager.update_last_video_id(channel.handle, video.video_id)

            # Call callback for new video detection
            if self.on_new_video:
                self.on_new_video(video, channel)

            # Automatically process if enabled
            if channel.monitoring.auto_download:
                self._process_new_video(video, channel)

    def _process_new_video(self, video: VideoInfo, channel: ChannelConfig):
        """Process a new video by downloading its transcript."""
        try:
            self._log_status(f"📝 Processing transcript: {video.title}")

            # Use existing transcript loader to process the video
            transcript_data = self.transcript_loader.load_transcript(
                video.url,
                languages=channel.monitoring.languages,
                force=False,  # Don't reprocess if already exists
                save_markdown=True,
                channel=channel.name,
            )

            if transcript_data.get("already_exists"):
                self._log_status(f"✓ Video already processed: {video.title}")
            else:
                self._log_status(f"✅ Transcript downloaded: {video.title}")

                # Auto-process to final location if enabled
                if channel.monitoring.auto_process:
                    try:
                        success, result = self.transcript_loader.process_video(
                            video.video_id, channel.name
                        )
                        if success:
                            self._log_status(f"✅ Video auto-processed: {result}")
                        else:
                            self._log_status(f"⚠️ Auto-process failed: {result}")
                    except Exception as e:
                        self._log_error(f"Auto-process error for {video.video_id}", e)

            # Call callback for processed video
            if self.on_video_processed:
                self.on_video_processed(video, channel, transcript_data)

        except Exception as e:
            self._log_error(f"Error processing video {video.title}", e)

    def check_now(self) -> dict[str, Any]:
        """Perform an immediate check of all channels (manual trigger)."""
        if self.running:
            self._log_status("Manual check requested while monitoring is running")
            return {"status": "running", "message": "Monitoring already active"}

        try:
            self._log_status("🔄 Manual channel check started")
            enabled_channels = self.config_manager.get_enabled_channels()

            if not enabled_channels:
                return {
                    "status": "no_channels",
                    "message": "No enabled channels configured",
                }

            results = {}
            for channel in enabled_channels:
                try:
                    self._check_channel(channel)
                    results[channel.handle] = {"status": "success"}
                except Exception as e:
                    results[channel.handle] = {"status": "error", "error": str(e)}

            self._log_status("✅ Manual check completed")
            return {"status": "success", "results": results}

        except Exception as e:
            self._log_error("Manual check failed", e)
            return {"status": "error", "error": str(e)}

    def _log_status(self, message: str):
        """Log status message with timestamp."""
        timestamp = datetime.now().strftime("%Y-%m-%d %H:%M:%S")
        formatted_message = f"[{timestamp}] {message}"
        print(formatted_message)

        if self.on_status:
            self.on_status(formatted_message)

    def _log_error(self, message: str, exception: Exception | None):
        """Log error message with timestamp."""
        timestamp = datetime.now().strftime("%Y-%m-%d %H:%M:%S")
        if exception:
            formatted_message = f"[{timestamp}] ❌ {message}: {exception}"
        else:
            formatted_message = f"[{timestamp}] ❌ {message}"

        print(formatted_message)

        if self.on_error:
            self.on_error(formatted_message, exception)

    def get_monitoring_status(self) -> dict[str, Any]:
        """Get current monitoring status and statistics."""
        stats = self.config_manager.get_stats()

        return {
            "monitoring_active": self.running,
            "global_enabled": self.config_manager.global_config.enabled,
            "check_interval": self.config_manager.global_config.check_interval,
            "stats": stats,
            "channels": [
                {
                    "name": ch.name,
                    "handle": ch.handle,
                    "enabled": ch.monitoring.enabled,
                    "has_channel_id": bool(ch.channel_id),
                    "last_video_id": ch.last_video_id,
                    "auto_download": ch.monitoring.auto_download,
                    "auto_process": ch.monitoring.auto_process,
                }
                for ch in self.config_manager.list_channels()
            ],
        }
