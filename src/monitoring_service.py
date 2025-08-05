"""YouTube channel monitoring service using RSS feeds."""

import threading
from collections.abc import Callable
from datetime import datetime, timedelta
from typing import Any

from config.channel_config import ChannelConfig, ChannelConfigManager

from .rss_monitor import RSSChannelMonitor, VideoInfo
from .transcript_loader import (
    RateLimitError,
    TranscriptUnavailableError,
    YouTubeTranscriptLoader,
)


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
        self.shutdown_event = threading.Event()

        # Rate limit recovery state (thread-safe)
        self.rate_limit_recovery_until = None
        self.incomplete_backfills = set()  # Track channels that need backfill retry
        self._recovery_lock = threading.Lock()  # Protect recovery state

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

    def _generate_startup_report(self) -> None:
        """Generate comprehensive startup status report."""
        self._log_status("=" * 70)
        self._log_status("📊 DASTILL MONITORING SERVICE - STARTUP STATUS REPORT")
        self._log_status("=" * 70)

        # Get all channel configurations
        all_channels = self.config_manager.list_channels()
        enabled_channels = self.config_manager.get_enabled_channels()

        # Channel subscription summary
        self._log_status("\n📺 CHANNEL SUBSCRIPTIONS:")
        self._log_status(f"   Total subscribed: {len(all_channels)}")
        self._log_status(f"   Active monitoring: {len(enabled_channels)}")
        self._log_status(f"   Disabled: {len(all_channels) - len(enabled_channels)}")

        # Get download statistics from file system
        from .file_manager import VideoFileManager

        manager = VideoFileManager(
            self.transcript_loader.config.get("storage.base_path")
        )

        # Count videos in different states
        to_be_downloaded = len(
            list((manager.base_path / "to_be_downloaded").glob("*.placeholder"))
        )
        downloaded = len(list((manager.base_path / "downloaded").glob("*.md")))

        # Count processed videos by channel
        processed_total = 0
        channel_counts = {}
        for channel_dir in manager.base_path.iterdir():
            if channel_dir.is_dir() and channel_dir.name not in [
                "to_be_downloaded",
                "downloaded",
                "unknown",
                "config",
            ]:
                count = len(list(channel_dir.glob("*.md")))
                if count > 0:
                    channel_counts[channel_dir.name] = count
                    processed_total += count

        # Also count unknown folder
        unknown_count = len(list((manager.base_path / "unknown").glob("*.md")))
        if unknown_count > 0:
            channel_counts["unknown"] = unknown_count
            processed_total += unknown_count

        self._log_status("\n📥 DOWNLOAD STATISTICS:")
        self._log_status(f"   Queued for download: {to_be_downloaded}")
        self._log_status(f"   Downloaded (awaiting processing): {downloaded}")
        self._log_status(f"   Total processed: {processed_total}")

        if channel_counts:
            self._log_status("\n   Processed by channel:")
            for channel, count in sorted(
                channel_counts.items(), key=lambda x: x[1], reverse=True
            ):
                self._log_status(f"      • {channel}: {count} videos")

        # Check rate limit status
        if self.rate_limit_recovery_until:
            remaining = (
                self.rate_limit_recovery_until - datetime.now()
            ).total_seconds() / 3600
            if remaining > 0:
                self._log_status("\n⚠️  RATE LIMIT RECOVERY MODE")
                self._log_status(
                    f"   Recovery until: {self.rate_limit_recovery_until.strftime('%Y-%m-%d %H:%M:%S')}"
                )
                self._log_status(f"   Time remaining: {remaining:.1f} hours")
                self._log_status("   (Using browser fallback method)")

        # List active channel details
        self._log_status("\n📡 ACTIVE CHANNEL DETAILS:")
        for i, channel in enumerate(enabled_channels, 1):
            self._log_status(f"\n   {i}. {channel.name} ({channel.handle})")
            self._log_status(f"      Channel ID: {channel.channel_id or 'NOT SET'}")

            # Get channel-specific stats
            channel_dir_name = manager._sanitize_channel_name(channel.name)
            channel_processed = channel_counts.get(channel_dir_name, 0)
            self._log_status(f"      Processed videos: {channel_processed}")

            if channel.last_video_id:
                self._log_status(f"      Last video ID: {channel.last_video_id}")

            # Check RSS feed for available videos
            if channel.channel_id:
                try:
                    recent_videos = self.rss_monitor.get_latest_videos(
                        channel.channel_id
                    )
                    new_videos = 0
                    for video in recent_videos:
                        status, _ = manager.get_video_status(video.video_id)
                        if status == "not_downloaded":
                            new_videos += 1
                    if new_videos > 0:
                        self._log_status(
                            f"      Available to download: {new_videos} new videos"
                        )
                except Exception:
                    pass

            # Show monitoring settings
            if channel.monitoring:
                settings = []
                if channel.monitoring.auto_download:
                    settings.append("auto-download")
                if channel.monitoring.auto_process:
                    settings.append("auto-process")
                if channel.monitoring.languages:
                    settings.append(
                        f"languages: {','.join(channel.monitoring.languages)}"
                    )
                if settings:
                    self._log_status(f"      Settings: {', '.join(settings)}")

        self._log_status("\n" + "=" * 70)
        self._log_status("")

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

        # Generate startup status report (before backfill shows current state)
        self._generate_startup_report()

        # Perform startup backfill for all channels
        self._log_status("🔄 Performing startup backfill for all channels...")
        self._startup_backfill(enabled_channels)

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
        self.shutdown_event.set()  # Signal shutdown to interruptible sleep
        if self.monitor_thread:
            self.monitor_thread.join(timeout=10)
        self._log_status("⏹️ Monitoring stopped")

    def _monitoring_loop(self):
        """Main monitoring loop that runs in a separate thread."""
        self._log_status("🔄 Monitoring loop started")
        check_count = 0

        while self.running:
            try:
                # Check if we've recovered from rate limiting and retry backfills
                self._check_rate_limit_recovery()

                # Normal channel monitoring
                self._check_all_channels()
                check_count += 1

                # Periodic memory cleanup every 10 checks to prevent memory leaks
                if check_count % 10 == 0:
                    self._cleanup_memory()

                # Use interruptible sleep instead of time.sleep
                if self.shutdown_event.wait(
                    self.config_manager.global_config.check_interval
                ):
                    self._log_status("🛑 Shutdown requested during sleep")
                    break
            except Exception as e:
                self._log_error("Error in monitoring loop", e)
                # Use interruptible sleep for error wait as well
                if self.shutdown_event.wait(
                    60
                ):  # Wait a minute before retrying on error
                    self._log_status("🛑 Shutdown requested during error recovery")
                    break

    def _check_rate_limit_recovery(self) -> None:
        """Check if we've recovered from rate limiting and retry incomplete backfills."""
        retry_channels = []

        with self._recovery_lock:
            current_time = datetime.now()

            # If we're still in rate limit recovery period, skip
            if (
                self.rate_limit_recovery_until
                and current_time < self.rate_limit_recovery_until
            ):
                return

            # If we have incomplete backfills and we've recovered, retry them
            if self.incomplete_backfills and self.rate_limit_recovery_until:
                self._log_status(
                    "🔄 Rate limit recovery complete - retrying incomplete backfills..."
                )
                self.rate_limit_recovery_until = None

                # Get channels that need backfill retry (copy the set to avoid modification during iteration)
                incomplete_handles = self.incomplete_backfills.copy()
                for channel in self.config_manager.get_enabled_channels():
                    if channel.handle in incomplete_handles:
                        retry_channels.append(channel)

                if retry_channels:
                    # Clear incomplete backfills before retry to avoid duplicate attempts
                    self.incomplete_backfills.clear()
                    # Retry outside the lock to avoid holding it during long operations

        # Retry backfills outside the lock
        if retry_channels:
            self._retry_backfill(retry_channels)

    def _retry_backfill(self, channels: list[ChannelConfig]):
        """Retry backfill for channels that were interrupted by rate limiting."""
        self._log_status(f"🔄 Retrying backfill for {len(channels)} channels...")

        for channel in channels:
            try:
                self._backfill_channel(channel, is_retry=True)
            except RateLimitError:
                # If we hit rate limit again during retry, mark for next retry
                self._log_status("⏸️ Rate limit hit again during backfill retry")
                self._handle_rate_limit_error()
                with self._recovery_lock:
                    self.incomplete_backfills.add(channel.handle)
                break  # Stop retrying other channels
            except TranscriptUnavailableError:
                # Skip channels with no transcripts available
                self._log_status(
                    f"⏭️ No transcripts available for {channel.name} during retry, continuing..."
                )
                continue
            except Exception as e:
                self._log_error(f"Error during backfill retry for {channel.name}", e)

    def _cleanup_memory(self) -> None:
        """Periodic memory cleanup to prevent leaks in long-running service."""
        try:
            # Light memory cleanup - let HTTP connections be managed properly by requests
            import gc

            gc.collect()
        except Exception as e:
            self._log_error("Error during memory cleanup", e)

    def _check_all_channels(self):
        """Check all enabled channels for new videos."""
        enabled_channels = self.config_manager.get_enabled_channels()

        # During rate limit recovery, still check RSS but don't download transcripts
        in_recovery = (
            self.rate_limit_recovery_until
            and datetime.now() < self.rate_limit_recovery_until
        )

        if in_recovery:
            recovery_time_left = self.rate_limit_recovery_until - datetime.now()
            hours_left = recovery_time_left.total_seconds() / 3600
            self._log_status(
                f"⏸️ In rate limit recovery - {hours_left:.1f}h remaining (RSS only)"
            )
        else:
            self._log_status(
                f"🔍 Checking {len(enabled_channels)} channels for new videos..."
            )

        for channel in enabled_channels:
            try:
                self._check_channel(channel, download_transcripts=not in_recovery)
            except RateLimitError:
                # Rate limit handled in _process_new_video or _handle_rate_limit_error
                # Skip remaining channels and wait for next cycle
                self._log_status("⏭️ Skipping remaining channels due to rate limit")
                break
            except Exception as e:
                self._log_error(f"Error checking channel {channel.handle}", e)

    def _check_channel(self, channel: ChannelConfig, download_transcripts: bool = True):
        """Check a single channel for new videos."""
        if not channel.channel_id:
            return

        # Get latest videos from RSS with configured limit
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
            if download_transcripts:
                self._log_status(
                    f"🎥 New video detected: {video.title} ({channel.name})"
                )
            else:
                self._log_status(
                    f"🎥 New video detected (recovery mode): {video.title} ({channel.name})"
                )

            # Update last video ID immediately to avoid reprocessing
            self.config_manager.update_last_video_id(channel.handle, video.video_id)

            # Call callback for new video detection
            if self.on_new_video:
                self.on_new_video(video, channel)

            # Automatically process if enabled and not in recovery mode
            if channel.monitoring.auto_download and download_transcripts:
                self._process_new_video(video, channel)

    def _startup_backfill(self, channels: list[ChannelConfig]):
        """Download historical videos from RSS feeds on startup."""
        for channel in channels:
            if not channel.channel_id or not channel.monitoring.auto_download:
                continue

            try:
                self._backfill_channel(channel, is_retry=False)
            except RateLimitError:
                # Mark this channel for retry later
                with self._recovery_lock:
                    self.incomplete_backfills.add(channel.handle)
                self._handle_rate_limit_error()
                break  # Stop backfill for remaining channels
            except TranscriptUnavailableError:
                # Skip channels with no transcripts available, but continue with other channels
                self._log_status(
                    f"⏭️ No transcripts available for recent videos in {channel.name}, continuing..."
                )
                continue
            except Exception as e:
                self._log_error(f"Error during backfill for {channel.name}", e)

    def _backfill_channel(self, channel: ChannelConfig, is_retry: bool = False):
        """Download historical videos for a single channel."""
        retry_suffix = " (retry)" if is_retry else ""
        self._log_status(f"🔍 Checking backfill for {channel.name}{retry_suffix}...")

        # Get videos for backfill with enhanced limit for comprehensive coverage
        # Use 2x the normal limit for backfill to get more historical content, but still bounded
        max_videos = self.config_manager.global_config.max_videos_per_check
        backfill_limit = (
            None if max_videos is None else max_videos * 2
        )  # Enhanced but bounded

        # Memory-aware processing: Warn if processing large number of videos
        if backfill_limit is None or (backfill_limit and backfill_limit > 200):
            self._log_status(
                f"⚠️  Processing large video set for {channel.name} - monitoring memory usage"
            )

        videos = self.rss_monitor.get_latest_videos(
            channel.channel_id, limit=backfill_limit
        )

        if not videos:
            self._log_status(f"   No videos found in RSS feed for {channel.name}")
            return

        backfill_count = 0
        processed_videos = []

        # Process videos in chronological order (oldest first)
        for video in reversed(videos):
            try:
                # Check if video already exists
                status, _ = self.transcript_loader.manager.get_video_status(
                    video.video_id
                )
                if status != "not_downloaded":
                    continue  # Skip if already downloaded/processed

                self._log_status(f"📥 Backfilling: {video.title}")

                # Download the transcript
                transcript_data = self.transcript_loader.load_transcript(
                    video.url,
                    languages=channel.monitoring.languages,
                    force=False,
                    save_markdown=True,
                    channel=channel.name,
                )

                if not transcript_data.get("already_exists"):
                    backfill_count += 1
                    processed_videos.append(video)

                    # Auto-process to final location if enabled
                    if channel.monitoring.auto_process:
                        try:
                            success, result = self.transcript_loader.process_video(
                                video.video_id, channel.name
                            )
                            if success:
                                self._log_status(f"   ✅ Auto-processed: {result}")
                        except Exception as e:
                            self._log_error(
                                f"Auto-process error for {video.video_id}", e
                            )

            except RateLimitError as e:
                self._log_error(f"Rate limit hit during backfill for {video.title}", e)
                self._log_status("⏸️ Rate limit detected during backfill...")
                # Mark channel for retry and re-raise to stop backfilling
                with self._recovery_lock:
                    self.incomplete_backfills.add(channel.handle)
                raise
            except TranscriptUnavailableError:
                self._log_status(
                    f"⏭️ No transcript available for {video.title}, skipping..."
                )
                continue  # Continue with next video
            except Exception as e:
                self._log_error(f"Error during backfill for {video.title}", e)
                continue  # Continue with next video

        # Update last_video_id to most recent video if we processed any
        if processed_videos or videos:
            latest_video = videos[0]  # First video is most recent
            self.config_manager.update_last_video_id(
                channel.handle, latest_video.video_id
            )
            self._log_status(f"   Updated last_video_id to: {latest_video.video_id}")

        if backfill_count > 0:
            self._log_status(
                f"✅ Backfilled {backfill_count} videos for {channel.name}"
            )
        else:
            self._log_status(f"   No new videos to backfill for {channel.name}")

    def _handle_rate_limit_error(self) -> None:
        """Handle rate limit error by setting recovery time."""
        try:
            # Get configurable recovery time, default to 3 hours
            rate_config = self.config_manager.global_config.rate_limiting
            recovery_hours = getattr(rate_config, "recovery_hours", 3)
        except (AttributeError, TypeError):
            recovery_hours = 3

        with self._recovery_lock:
            self.rate_limit_recovery_until = datetime.now() + timedelta(
                hours=recovery_hours
            )

        self._log_status(
            f"⏸️ Rate limit recovery set until {self.rate_limit_recovery_until.strftime('%Y-%m-%d %H:%M:%S')}"
        )
        self._log_status(
            f"🔄 Will retry incomplete backfills in {recovery_hours} hours..."
        )

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

        except RateLimitError as e:
            self._log_error(f"Rate limit hit while processing {video.title}", e)
            self._handle_rate_limit_error()
            # Re-raise to let the monitoring loop handle it
            raise
        except TranscriptUnavailableError:
            self._log_status(
                f"⏭️ No transcript available for {video.title}, skipping..."
            )
            # Don't re-raise, just skip this video
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
                    self._check_channel(channel, download_transcripts=True)
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
