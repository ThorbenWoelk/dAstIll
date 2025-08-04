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

    def test_check_channel_new_video(self):
        """Test checking a channel with new videos."""
        # Mock RSS monitor
        mock_monitor = Mock()
        test_video = VideoInfo(
            "new123", "New Video", "2024-01-01", "Test Channel", "UC_test", ""
        )
        mock_monitor.get_latest_videos.return_value = [test_video]
        self.service.rss_monitor = mock_monitor

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

        # Verify new video was detected and callback called
        self.service.on_new_video.assert_called_once_with(test_video, channel)

        # Verify last video ID was updated in the config manager
        updated_channel = self.service.config_manager.get_channel("@testchannel")
        self.assertEqual(updated_channel.last_video_id, "new123")

    @patch("src.monitoring_service.RSSChannelMonitor")
    def test_check_channel_no_new_videos(self, mock_rss_monitor_class):
        """Test checking a channel with no new videos."""
        # Mock RSS monitor
        mock_monitor = Mock()
        existing_video = VideoInfo(
            "existing123", "Existing Video", "2024-01-01", "Test Channel", "UC_test", ""
        )
        mock_monitor.get_latest_videos.return_value = [existing_video]
        mock_rss_monitor_class.return_value = mock_monitor
        self.service.rss_monitor = mock_monitor

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

        # Verify no new video callback was made
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


if __name__ == "__main__":
    unittest.main()
