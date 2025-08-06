"""Tests for monitoring service functionality."""

import shutil
import tempfile
import unittest
from unittest.mock import Mock, patch

from config.channel_config import ChannelConfig, MonitoringSettings
from src.monitoring_service import ChannelMonitoringService
from src.rss_monitor import VideoInfo


class TestMonitoringService(unittest.TestCase):
    """Test cases for ChannelMonitoringService."""

    def setUp(self):
        """Set up test fixtures."""
        self.temp_dir = tempfile.mkdtemp()
        self.service = ChannelMonitoringService(config_dir=self.temp_dir)

        # Mock the transcript loader to avoid real API calls
        self.service.transcript_loader = Mock()
        self.service.transcript_loader.load_transcript.return_value = {
            "already_exists": False,
            "video_id": "test123",
            "language": "en",
        }
        self.service.transcript_loader.process_video.return_value = (True, "processed")
        # Mock the config to return a valid path for VideoFileManager
        self.service.transcript_loader.config.get.return_value = self.temp_dir

    def tearDown(self):
        """Clean up."""
        if self.service.running:
            self.service.stop_monitoring()
        shutil.rmtree(self.temp_dir)

    def test_initial_state(self):
        """Test initial service state."""
        self.assertFalse(self.service.running)
        self.assertIsNone(self.service.monitor_thread)

        status = self.service.get_monitoring_status()
        self.assertFalse(status["monitoring_active"])
        self.assertFalse(status["global_enabled"])

    def test_test_configuration(self):
        """Test configuration testing."""
        # Mock RSS monitor
        mock_monitor = Mock()
        mock_monitor.test_rss_feed.return_value = True
        mock_monitor.get_latest_videos.return_value = [
            VideoInfo(
                "video123", "Test Video", "2024-01-01", "Test Channel", "UC_test", ""
            )
        ]
        self.service.rss_monitor = mock_monitor

        # Add a channel with channel ID
        self.service.config_manager.add_channel(
            "Test Channel", "@testchannel", channel_id="UC_test123"
        )

        result = self.service.test_configuration()

        self.assertTrue(result)
        mock_monitor.test_rss_feed.assert_called_once_with("UC_test123")
        mock_monitor.get_latest_videos.assert_called_once_with("UC_test123", limit=1)

    def test_start_monitoring_disabled_globally(self):
        """Test starting monitoring when globally disabled."""
        # Global monitoring is disabled by default
        result = self.service.start_monitoring()
        self.assertFalse(result)
        self.assertFalse(self.service.running)

    def test_start_monitoring_no_channels(self):
        """Test starting monitoring with no enabled channels."""
        self.service.config_manager.set_global_monitoring(True)

        result = self.service.start_monitoring()
        self.assertFalse(result)
        self.assertFalse(self.service.running)

    @patch("src.monitoring_service.RSSChannelMonitor")
    def test_start_monitoring_success(self, mock_rss_monitor_class):
        """Test successful monitoring start."""
        # Mock RSS monitor
        mock_monitor = Mock()
        mock_monitor.test_rss_feed.return_value = True
        mock_rss_monitor_class.return_value = mock_monitor

        # Setup configuration
        self.service.config_manager.set_global_monitoring(True)
        self.service.config_manager.add_channel(
            "Test Channel", "@testchannel", channel_id="UC_test123"
        )

        result = self.service.start_monitoring()

        self.assertTrue(result)
        self.assertTrue(self.service.running)
        self.assertIsNotNone(self.service.monitor_thread)

        # Stop monitoring for cleanup
        self.service.stop_monitoring()

    def test_stop_monitoring(self):
        """Test stopping monitoring."""
        # Manually set running state
        self.service.running = True
        self.service.monitor_thread = Mock()

        self.service.stop_monitoring()

        self.assertFalse(self.service.running)
        self.service.monitor_thread.join.assert_called_once()

    @patch("src.file_manager.VideoFileManager")
    def test_check_channel_new_video(self, mock_file_manager_class):
        """Test checking a channel with unprocessed videos."""
        # Mock RSS monitor
        mock_monitor = Mock()
        test_video = VideoInfo(
            "new123", "New Video", "2024-01-01", "Test Channel", "UC_test", ""
        )
        mock_monitor.get_latest_videos.return_value = [test_video]
        self.service.rss_monitor = mock_monitor

        # Mock file manager to indicate video is not downloaded
        mock_file_manager = Mock()
        mock_file_manager.get_video_status.return_value = ("not_downloaded", None)
        mock_file_manager_class.return_value = mock_file_manager

        # Add the channel to the config manager first
        self.service.config_manager.add_channel(
            "Test Channel", "@testchannel", channel_id="UC_test123"
        )

        # Get the channel from config manager
        channel = self.service.config_manager.get_channel("@testchannel")

        # Mock callbacks
        self.service.on_new_video = Mock()

        # Check channel
        self.service._check_channel(channel)

        # Verify unprocessed video was detected and callback called
        self.service.on_new_video.assert_called_once_with(test_video, channel)

        # Verify last video ID was updated in the config manager
        updated_channel = self.service.config_manager.get_channel("@testchannel")
        self.assertEqual(updated_channel.last_video_id, "new123")

    @patch("src.file_manager.VideoFileManager")
    @patch("src.monitoring_service.RSSChannelMonitor")
    def test_check_channel_no_new_videos(
        self, mock_rss_monitor_class, mock_file_manager_class
    ):
        """Test checking a channel with already processed videos."""
        # Mock RSS monitor
        mock_monitor = Mock()
        existing_video = VideoInfo(
            "existing123", "Existing Video", "2024-01-01", "Test Channel", "UC_test", ""
        )
        mock_monitor.get_latest_videos.return_value = [existing_video]
        mock_rss_monitor_class.return_value = mock_monitor
        self.service.rss_monitor = mock_monitor

        # Mock file manager to indicate video is already processed
        mock_file_manager = Mock()
        mock_file_manager.get_video_status.return_value = ("processed", "/path/to/file")
        mock_file_manager_class.return_value = mock_file_manager

        # Create channel with last video ID matching the returned video
        channel = ChannelConfig(
            name="Test Channel",
            handle="@testchannel",
            channel_id="UC_test123",
            last_video_id="existing123",
            monitoring=MonitoringSettings(),
        )

        # Mock callbacks
        self.service.on_new_video = Mock()

        # Check channel
        self.service._check_channel(channel)

        # Verify no new video callback was made (video already processed)
        self.service.on_new_video.assert_not_called()

    def test_process_new_video_auto_download(self):
        """Test processing new video with auto-download enabled."""
        video = VideoInfo(
            "test123", "Test Video", "2024-01-01", "Test Channel", "UC_test", ""
        )
        channel = ChannelConfig(
            name="Test Channel",
            handle="@testchannel",
            monitoring=MonitoringSettings(auto_download=True, auto_process=False),
        )

        # Mock callbacks
        self.service.on_video_processed = Mock()

        # Process video
        self.service._process_new_video(video, channel)

        # Verify transcript loader was called
        self.service.transcript_loader.load_transcript.assert_called_once_with(
            video.url,
            languages=channel.monitoring.languages,
            force=False,
            save_markdown=True,
            channel=channel.name,
        )

        # Verify callback was called
        self.service.on_video_processed.assert_called_once()

    def test_process_new_video_auto_process(self):
        """Test processing new video with auto-process enabled."""
        video = VideoInfo(
            "test123", "Test Video", "2024-01-01", "Test Channel", "UC_test", ""
        )
        channel = ChannelConfig(
            name="Test Channel",
            handle="@testchannel",
            monitoring=MonitoringSettings(auto_download=True, auto_process=True),
        )

        # Process video
        self.service._process_new_video(video, channel)

        # Verify both transcript loading and processing were called
        self.service.transcript_loader.load_transcript.assert_called_once()
        self.service.transcript_loader.process_video.assert_called_once_with(
            video.video_id, channel.name
        )

    def test_check_now_no_channels(self):
        """Test manual check with no channels."""
        result = self.service.check_now()

        self.assertEqual(result["status"], "no_channels")

    @patch("src.monitoring_service.RSSChannelMonitor")
    def test_check_now_success(self, mock_rss_monitor_class):
        """Test successful manual check."""
        # Mock RSS monitor
        mock_monitor = Mock()
        mock_monitor.get_latest_videos.return_value = []
        mock_rss_monitor_class.return_value = mock_monitor
        self.service.rss_monitor = mock_monitor

        # Add a channel
        self.service.config_manager.add_channel(
            "Test Channel", "@testchannel", channel_id="UC_test123"
        )

        result = self.service.check_now()

        self.assertEqual(result["status"], "success")
        self.assertIn("@testchannel", result["results"])
        self.assertEqual(result["results"]["@testchannel"]["status"], "success")

    def test_get_monitoring_status(self):
        """Test getting monitoring status."""
        # Add some test data
        self.service.config_manager.add_channel("Test Channel", "@testchannel")
        self.service.config_manager.set_global_monitoring(True)
        self.service.config_manager.set_check_interval(600)

        status = self.service.get_monitoring_status()

        self.assertFalse(status["monitoring_active"])  # Not started
        self.assertTrue(status["global_enabled"])
        self.assertEqual(status["check_interval"], 600)

        self.assertEqual(len(status["channels"]), 1)
        channel_status = status["channels"][0]
        self.assertEqual(channel_status["name"], "Test Channel")
        self.assertEqual(channel_status["handle"], "@testchannel")
        self.assertTrue(channel_status["enabled"])
        self.assertFalse(channel_status["has_channel_id"])

    def test_test_configuration_no_channels(self):
        """Test configuration testing with no channels."""
        result = self.service.test_configuration()
        self.assertTrue(result)

    @patch("src.monitoring_service.RSSChannelMonitor")
    def test_test_configuration_with_channels(self, mock_rss_monitor_class):
        """Test configuration testing with channels."""
        # Setup mock RSS monitor
        mock_rss_monitor = Mock()
        mock_rss_monitor.test_rss_feed.return_value = True
        mock_rss_monitor.get_latest_videos.return_value = [
            VideoInfo(
                "test123",
                "Test Video",
                "2023-01-01",
                "Test Channel",
                "UC123",
                "https://www.youtube.com/watch?v=test123",
            )
        ]
        mock_rss_monitor_class.return_value = mock_rss_monitor
        self.service.rss_monitor = mock_rss_monitor

        # Add a test channel with channel ID
        self.service.config_manager.add_channel(
            "Test Channel", "@test", channel_id="UC123"
        )

        result = self.service.test_configuration()
        self.assertTrue(result)

        # Verify RSS feed was tested
        mock_rss_monitor.test_rss_feed.assert_called_with("UC123")
        mock_rss_monitor.get_latest_videos.assert_called_with("UC123", limit=1)

    def test_test_configuration_missing_channel_id(self):
        """Test configuration testing with missing channel ID."""
        # Add a channel without channel_id
        self.service.config_manager.add_channel("Test Channel", "@test")

        result = self.service.test_configuration()
        self.assertFalse(result)  # Should fail due to missing channel ID

    @patch("src.monitoring_service.RSSChannelMonitor")
    def test_test_configuration_rss_failure(self, mock_rss_monitor_class):
        """Test configuration testing with RSS failure."""
        # Setup mock RSS monitor to fail
        mock_rss_monitor = Mock()
        mock_rss_monitor.test_rss_feed.return_value = False
        mock_rss_monitor_class.return_value = mock_rss_monitor
        self.service.rss_monitor = mock_rss_monitor

        # Add a test channel
        self.service.config_manager.add_channel(
            "Test Channel", "@test", channel_id="UC123"
        )

        result = self.service.test_configuration()
        self.assertFalse(result)  # Should fail due to RSS failure

    def test_log_status_callback(self):
        """Test status logging callback."""
        status_messages = []
        self.service.on_status = lambda msg: status_messages.append(msg)

        self.service._log_status("Test message")
        self.assertEqual(len(status_messages), 1)
        # Status message includes timestamp prefix
        self.assertIn("Test message", status_messages[0])

    def test_log_error_callback(self):
        """Test error logging callback."""
        error_messages = []
        self.service.on_error = lambda context, error: error_messages.append(
            (context, str(error))
        )

        test_error = Exception("Test error")
        self.service._log_error("Test context", test_error)

        self.assertEqual(len(error_messages), 1)
        # Error message includes formatted context
        self.assertIn("Test context", error_messages[0][0])
        self.assertIn("Test error", error_messages[0][0])

    def test_rate_limit_recovery_state(self):
        """Test rate limit recovery state management."""
        from datetime import datetime, timedelta

        # Initially no rate limit recovery time set
        self.assertIsNone(self.service.rate_limit_recovery_until)

        # Set rate limit recovery time
        future_time = datetime.now() + timedelta(hours=1)
        with self.service._recovery_lock:
            self.service.rate_limit_recovery_until = future_time

        # Verify it's set
        self.assertEqual(self.service.rate_limit_recovery_until, future_time)

        # Call the check method (doesn't return anything, but shouldn't crash)
        self.service._check_rate_limit_recovery()

        # Should still be set because we're before the recovery time
        self.assertIsNotNone(self.service.rate_limit_recovery_until)

    def test_incomplete_backfills_management(self):
        """Test incomplete backfills tracking."""
        # Initially empty
        self.assertEqual(len(self.service.incomplete_backfills), 0)

        # Add incomplete backfill
        with self.service._recovery_lock:
            self.service.incomplete_backfills.add("@test_channel")

        self.assertIn("@test_channel", self.service.incomplete_backfills)

        # Clear incomplete backfills
        with self.service._recovery_lock:
            self.service.incomplete_backfills.clear()

        self.assertEqual(len(self.service.incomplete_backfills), 0)


if __name__ == "__main__":
    unittest.main()
