"""Tests for channel subscription functionality."""

from unittest.mock import MagicMock, patch

import pytest

from src.rss_monitor import VideoInfo


@pytest.fixture
def mock_channel_config():
    """Create a mock channel configuration."""
    return {
        "name": "Test Channel",
        "handle": "@testchannel",
        "channel_id": "UC1234567890123456789012",
        "languages": ["en"],
        "auto_download": True,
        "auto_process": False,
    }


@pytest.fixture
def mock_videos():
    """Create mock video data from RSS feed."""
    return [
        VideoInfo(
            video_id="video1",
            title="Test Video 1",
            published="2024-01-01T00:00:00",
            channel_name="Test Channel",
            channel_id="UC1234567890123456789012",
            url="https://www.youtube.com/watch?v=video1",
        ),
        VideoInfo(
            video_id="video2",
            title="Test Video 2",
            published="2024-01-02T00:00:00",
            channel_name="Test Channel",
            channel_id="UC1234567890123456789012",
            url="https://www.youtube.com/watch?v=video2",
        ),
        VideoInfo(
            video_id="video3",
            title="Test Video 3",
            published="2024-01-03T00:00:00",
            channel_name="Test Channel",
            channel_id="UC1234567890123456789012",
            url="https://www.youtube.com/watch?v=video3",
        ),
    ]


class TestChannelSubscription:
    """Test channel subscription functionality."""

    @patch("main.ChannelConfigManager")
    @patch("src.rss_monitor.RSSChannelMonitor")
    @patch("src.transcript_loader.YouTubeTranscriptLoader")
    def test_subscribe_new_channel_success(
        self,
        mock_loader_class,
        mock_rss_class,
        mock_config_class,
        mock_channel_config,
        mock_videos,
    ):
        """Test successful subscription to a new channel."""
        # Setup mocks
        mock_config_manager = MagicMock()
        mock_config_class.return_value = mock_config_manager
        mock_config_manager.add_channel.return_value = True

        mock_rss_monitor = MagicMock()
        mock_rss_class.return_value = mock_rss_monitor
        mock_rss_monitor.get_latest_videos.return_value = mock_videos[:3]

        mock_loader = MagicMock()
        mock_loader_class.return_value = mock_loader
        mock_loader.load_transcript.return_value = {"already_exists": False}

        # Create args mock
        args = MagicMock()
        args.channel_action = "subscribe"
        args.name = mock_channel_config["name"]
        args.handle = mock_channel_config["handle"]
        args.channel_id = mock_channel_config["channel_id"]
        args.languages = mock_channel_config["languages"]
        args.auto_download = mock_channel_config["auto_download"]
        args.auto_process = mock_channel_config["auto_process"]
        args.recent_count = 3

        # Import and run the handler
        from main import handle_channel

        handle_channel(args)

        # Verify channel was added
        mock_config_manager.add_channel.assert_called_once_with(
            name=mock_channel_config["name"],
            handle=mock_channel_config["handle"],
            channel_id=mock_channel_config["channel_id"],
            languages=mock_channel_config["languages"],
            auto_download=mock_channel_config["auto_download"],
            auto_process=mock_channel_config["auto_process"],
        )

        # Verify RSS feed was queried
        mock_rss_monitor.get_latest_videos.assert_called_once_with(
            mock_channel_config["channel_id"], limit=3
        )

        # Verify transcripts were downloaded
        assert mock_loader.load_transcript.call_count == 3
        for video in mock_videos[:3]:
            mock_loader.load_transcript.assert_any_call(
                video.url,
                languages=mock_channel_config["languages"],
                force=False,
                save_markdown=True,
                channel=mock_channel_config["name"],
            )

        # Verify last video ID was updated
        mock_config_manager.update_last_video_id.assert_called_once_with(
            mock_channel_config["handle"], "video1"
        )

    @patch("main.ChannelConfigManager")
    @patch("src.rss_monitor.RSSChannelMonitor")
    def test_subscribe_existing_channel_fails(
        self, mock_rss_class, mock_config_class, mock_channel_config
    ):
        """Test subscription fails for existing channel."""
        # Setup mocks
        mock_config_manager = MagicMock()
        mock_config_class.return_value = mock_config_manager
        mock_config_manager.add_channel.return_value = False  # Channel already exists

        # Create args mock
        args = MagicMock()
        args.channel_action = "subscribe"
        args.name = mock_channel_config["name"]
        args.handle = mock_channel_config["handle"]
        args.channel_id = mock_channel_config["channel_id"]
        args.languages = mock_channel_config["languages"]
        args.auto_download = mock_channel_config["auto_download"]
        args.auto_process = mock_channel_config["auto_process"]
        args.recent_count = 15

        # Import and run the handler
        from main import handle_channel

        handle_channel(args)

        # Verify channel add was attempted
        mock_config_manager.add_channel.assert_called_once_with(
            name=mock_channel_config["name"],
            handle=mock_channel_config["handle"],
            channel_id=mock_channel_config["channel_id"],
            languages=mock_channel_config["languages"],
            auto_download=mock_channel_config["auto_download"],
            auto_process=mock_channel_config["auto_process"],
        )

        # Verify no RSS monitor was created since the channel already exists
        mock_rss_class.assert_not_called()

        # Verify no further actions were taken
        mock_config_manager.update_last_video_id.assert_not_called()

    @patch("main.RSSChannelMonitor")
    @patch("main.YouTubeTranscriptLoader")
    @patch("main.validate_channel_id")
    @patch("main.check_disk_space")
    @patch("main.Config")
    @patch("main.ChannelConfigManager")
    def test_subscribe_with_auto_process(
        self,
        mock_config_class,
        mock_main_config_class,
        mock_disk_space,
        mock_validate_channel,
        mock_loader_class,
        mock_rss_class,
        mock_channel_config,
        mock_videos,
    ):
        """Test subscription with auto-process enabled."""
        # Setup mocks
        mock_config_manager = MagicMock()
        mock_config_class.return_value = mock_config_manager
        mock_config_manager.add_channel.return_value = True

        # Mock Config class for main.py
        mock_main_config = MagicMock()
        mock_main_config_class.return_value = mock_main_config
        mock_main_config.get.return_value = 20  # max_recent_videos

        # Mock disk space check
        mock_disk_space.return_value = True

        # Mock channel validation
        mock_validate_channel.return_value = True

        mock_rss_monitor = MagicMock()
        mock_rss_class.return_value = mock_rss_monitor
        mock_rss_monitor.get_latest_videos.return_value = [
            mock_videos[0]
        ]  # Just one video

        mock_loader = MagicMock()
        mock_loader_class.return_value = mock_loader
        mock_loader.load_transcript.return_value = {"already_exists": False}
        mock_loader.process_video.return_value = (True, "/path/to/processed/video.md")

        # Create args mock with auto_process enabled
        args = MagicMock()
        args.channel_action = "subscribe"
        args.name = mock_channel_config["name"]
        args.handle = mock_channel_config["handle"]
        args.channel_id = mock_channel_config["channel_id"]
        args.languages = mock_channel_config["languages"]
        args.auto_download = True
        args.auto_process = True  # Enable auto-process
        args.recent_count = 1

        # Import and run the handler
        from main import handle_channel

        handle_channel(args)

        # Verify video was processed
        mock_loader.process_video.assert_called_once_with(
            "video1", mock_channel_config["name"]
        )

    @patch("main.RSSChannelMonitor")
    @patch("main.validate_channel_id")
    @patch("main.check_disk_space")
    @patch("main.Config")
    @patch("main.ChannelConfigManager")
    def test_subscribe_no_videos_available(
        self,
        mock_config_class,
        mock_main_config_class,
        mock_disk_space,
        mock_validate_channel,
        mock_rss_class,
        mock_channel_config,
    ):
        """Test subscription when RSS feed returns no videos."""
        # Setup mocks
        mock_config_manager = MagicMock()
        mock_config_class.return_value = mock_config_manager
        mock_config_manager.add_channel.return_value = True

        # Mock Config class for main.py
        mock_main_config = MagicMock()
        mock_main_config_class.return_value = mock_main_config
        mock_main_config.get.return_value = 20  # max_recent_videos

        # Mock disk space check
        mock_disk_space.return_value = True

        # Mock channel validation
        mock_validate_channel.return_value = True

        mock_rss_monitor = MagicMock()
        mock_rss_class.return_value = mock_rss_monitor
        mock_rss_monitor.get_latest_videos.return_value = []  # No videos

        # Create args mock
        args = MagicMock()
        args.channel_action = "subscribe"
        args.name = mock_channel_config["name"]
        args.handle = mock_channel_config["handle"]
        args.channel_id = mock_channel_config["channel_id"]
        args.languages = mock_channel_config["languages"]
        args.auto_download = mock_channel_config["auto_download"]
        args.auto_process = mock_channel_config["auto_process"]
        args.recent_count = 15

        # Import and run the handler
        from main import handle_channel

        handle_channel(args)

        # Verify channel was added
        mock_config_manager.add_channel.assert_called_once_with(
            name=mock_channel_config["name"],
            handle=mock_channel_config["handle"],
            channel_id=mock_channel_config["channel_id"],
            languages=mock_channel_config["languages"],
            auto_download=mock_channel_config["auto_download"],
            auto_process=mock_channel_config["auto_process"],
        )

        # Verify RSS feed was queried
        mock_rss_monitor.get_latest_videos.assert_called_once_with(
            mock_channel_config["channel_id"], limit=15
        )

        # Verify no video processing occurred
        mock_config_manager.update_last_video_id.assert_not_called()

    @patch("main.RSSChannelMonitor")
    @patch("main.YouTubeTranscriptLoader")
    @patch("main.validate_channel_id")
    @patch("main.check_disk_space")
    @patch("main.Config")
    @patch("main.ChannelConfigManager")
    def test_subscribe_handles_download_errors(
        self,
        mock_config_class,
        mock_main_config_class,
        mock_disk_space,
        mock_validate_channel,
        mock_loader_class,
        mock_rss_class,
        mock_channel_config,
        mock_videos,
    ):
        """Test subscription handles errors during video download."""
        # Setup mocks
        mock_config_manager = MagicMock()
        mock_config_class.return_value = mock_config_manager
        mock_config_manager.add_channel.return_value = True

        # Mock Config class for main.py
        mock_main_config = MagicMock()
        mock_main_config_class.return_value = mock_main_config
        mock_main_config.get.return_value = 20  # max_recent_videos

        # Mock disk space check
        mock_disk_space.return_value = True

        # Mock channel validation
        mock_validate_channel.return_value = True

        mock_rss_monitor = MagicMock()
        mock_rss_class.return_value = mock_rss_monitor
        mock_rss_monitor.get_latest_videos.return_value = mock_videos[:2]

        mock_loader = MagicMock()
        mock_loader_class.return_value = mock_loader
        # First video succeeds, second fails
        mock_loader.load_transcript.side_effect = [
            {"already_exists": False},
            Exception("Download failed"),
        ]

        # Create args mock
        args = MagicMock()
        args.channel_action = "subscribe"
        args.name = mock_channel_config["name"]
        args.handle = mock_channel_config["handle"]
        args.channel_id = mock_channel_config["channel_id"]
        args.languages = mock_channel_config["languages"]
        args.auto_download = mock_channel_config["auto_download"]
        args.auto_process = mock_channel_config["auto_process"]
        args.recent_count = 2

        # Import and run the handler
        from main import handle_channel

        handle_channel(args)

        # Verify channel was added
        mock_config_manager.add_channel.assert_called_once_with(
            name=mock_channel_config["name"],
            handle=mock_channel_config["handle"],
            channel_id=mock_channel_config["channel_id"],
            languages=mock_channel_config["languages"],
            auto_download=mock_channel_config["auto_download"],
            auto_process=mock_channel_config["auto_process"],
        )

        # Verify both downloads were attempted
        assert mock_loader.load_transcript.call_count == 2

        # Verify last video ID was still updated (to prevent re-downloading)
        mock_config_manager.update_last_video_id.assert_called_once_with(
            mock_channel_config["handle"], "video1"
        )

    @patch("main.RSSChannelMonitor")
    @patch("main.YouTubeTranscriptLoader")
    @patch("main.validate_channel_id")
    @patch("main.check_disk_space")
    @patch("main.Config")
    @patch("main.ChannelConfigManager")
    def test_subscribe_respects_recent_count_limit(
        self,
        mock_config_class,
        mock_main_config_class,
        mock_disk_space,
        mock_validate_channel,
        mock_loader_class,
        mock_rss_class,
        mock_channel_config,
    ):
        """Test subscription respects the recent count limit."""
        # Setup mocks
        mock_config_manager = MagicMock()
        mock_config_class.return_value = mock_config_manager
        mock_config_manager.add_channel.return_value = True

        # Mock Config class for main.py
        mock_main_config = MagicMock()
        mock_main_config_class.return_value = mock_main_config
        mock_main_config.get.return_value = 20  # max_recent_videos

        # Mock disk space check
        mock_disk_space.return_value = True

        # Mock channel validation
        mock_validate_channel.return_value = True

        mock_rss_monitor = MagicMock()
        mock_rss_class.return_value = mock_rss_monitor

        mock_loader = MagicMock()
        mock_loader_class.return_value = mock_loader
        mock_loader.load_transcript.return_value = {"already_exists": False}

        # Create many mock videos
        many_videos = [
            VideoInfo(
                video_id=f"video{i}",
                title=f"Test Video {i}",
                published=f"2024-01-{i:02d}T00:00:00",
                channel_name="Test Channel",
                channel_id="UC1234567890123456789012",
                url=f"https://www.youtube.com/watch?v=video{i}",
            )
            for i in range(1, 31)
        ]
        mock_rss_monitor.get_latest_videos.return_value = many_videos

        # Create args mock with high recent_count
        args = MagicMock()
        args.channel_action = "subscribe"
        args.name = mock_channel_config["name"]
        args.handle = mock_channel_config["handle"]
        args.channel_id = mock_channel_config["channel_id"]
        args.languages = mock_channel_config["languages"]
        args.auto_download = mock_channel_config["auto_download"]
        args.auto_process = mock_channel_config["auto_process"]
        args.recent_count = 25  # Request more than 20

        # Import and run the handler
        from main import handle_channel

        handle_channel(args)

        # Verify RSS was queried with limit of 20 (capped)
        mock_rss_monitor.get_latest_videos.assert_called_once_with(
            mock_channel_config["channel_id"], limit=20
        )
