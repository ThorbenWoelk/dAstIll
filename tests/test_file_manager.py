#!/usr/bin/env python3
"""
Tests for StatelessVideoManager module.
"""

import shutil
import tempfile
from pathlib import Path

import pytest

from src.file_manager import VideoFileManager


class TestVideoFileManager:
    def setup_method(self):
        """Setup test environment with temporary directory."""
        self.temp_dir = tempfile.mkdtemp()
        self.base_path = Path(self.temp_dir)
        self.manager = VideoFileManager(str(self.base_path))

    def teardown_method(self):
        """Clean up temporary directory."""
        shutil.rmtree(self.temp_dir)

    def test_init_creates_directories(self):
        """Test that initialization creates required directories."""
        assert (self.base_path / "to_be_downloaded").exists()
        assert (self.base_path / "downloaded").exists()
        assert (self.base_path / "unknown").exists()

    def test_sanitize_channel_name_basic(self):
        """Test basic channel name sanitization."""
        # Normal channel name
        assert self.manager._sanitize_channel_name("tina huang") == "tina huang"

        # Empty/None cases
        assert self.manager._sanitize_channel_name("") == "unknown"
        assert self.manager._sanitize_channel_name(None) == "unknown"
        assert self.manager._sanitize_channel_name("   ") == "unknown"

    def test_sanitize_channel_name_security(self):
        """Test channel name sanitization prevents path traversal."""
        # Path traversal attempts
        assert self.manager._sanitize_channel_name("../evil") == "evil"
        assert self.manager._sanitize_channel_name("../../etc/passwd") == "etcpasswd"
        assert self.manager._sanitize_channel_name("dir/subdir") == "dirsubdir"

        # Invalid filesystem characters
        assert (
            self.manager._sanitize_channel_name('test<>:"|?*channel') == "testchannel"
        )

        # Long name truncation
        long_name = "a" * 150
        result = self.manager._sanitize_channel_name(long_name)
        assert len(result) == 100

    def test_sanitize_video_id_valid(self):
        """Test video ID sanitization with valid IDs."""
        assert self.manager._sanitize_video_id("abc123def45") == "abc123def45"
        assert self.manager._sanitize_video_id("test_video-1") == "test_video-1"

    def test_sanitize_video_id_invalid(self):
        """Test video ID sanitization with invalid input."""
        with pytest.raises(ValueError):
            self.manager._sanitize_video_id("")

        with pytest.raises(ValueError):
            self.manager._sanitize_video_id(None)

        # Invalid characters should be removed
        assert self.manager._sanitize_video_id("abc@123#def") == "abc123def"

    def test_get_video_status_not_downloaded(self):
        """Test status detection for non-existent video."""
        status, path = self.manager.get_video_status("nonexistent123")
        assert status == "not_downloaded"
        assert path is None

    def test_add_to_be_downloaded_success(self):
        """Test adding video to download queue."""
        video_id = "test12345678"
        channel = "test channel"

        result = self.manager.add_to_be_downloaded(video_id, channel)
        assert result is True

        # Check file was created
        expected_file = (
            self.base_path / "to_be_downloaded" / f"{video_id}_test channel.md"
        )
        assert expected_file.exists()

        # Check status
        status, path = self.manager.get_video_status(video_id)
        assert status == "to_be_downloaded"
        assert path == str(expected_file)

    def test_add_to_be_downloaded_duplicate(self):
        """Test that duplicate videos are not added."""
        video_id = "test12345678"

        # First add should succeed
        assert self.manager.add_to_be_downloaded(video_id) is True

        # Second add should fail
        assert self.manager.add_to_be_downloaded(video_id) is False

    def test_mark_downloaded_from_to_be_downloaded(self):
        """Test moving video from to_be_downloaded to downloaded."""
        video_id = "test12345678"
        channel = "test channel"
        content = "Test transcript content"

        # First add to queue
        self.manager.add_to_be_downloaded(video_id, channel)

        # Then mark as downloaded
        result_path = self.manager.mark_downloaded(video_id, content, channel)

        assert result_path is not None
        assert "downloaded" in result_path

        # Check original placeholder is removed
        placeholder_file = (
            self.base_path / "to_be_downloaded" / f"{video_id}_test channel.md"
        )
        assert not placeholder_file.exists()

        # Check downloaded file exists with content
        downloaded_file = Path(result_path)
        assert downloaded_file.exists()
        assert downloaded_file.read_text() == content

        # Check status
        status, path = self.manager.get_video_status(video_id)
        assert status == "downloaded"
        assert path == result_path

    def test_mark_processed_success(self):
        """Test moving video from downloaded to processed."""
        video_id = "test12345678"
        channel = "test channel"
        content = "Test transcript content"

        # Setup: add and download
        self.manager.add_to_be_downloaded(video_id, channel)
        downloaded_path = self.manager.mark_downloaded(video_id, content, channel)

        # Mark as processed
        processed_path = self.manager.mark_processed(video_id, channel)

        assert processed_path is not None
        assert "test channel" in processed_path

        # Check downloaded file is removed
        assert not Path(downloaded_path).exists()

        # Check processed file exists
        processed_file = Path(processed_path)
        assert processed_file.exists()
        assert processed_file.read_text() == content

        # Check status
        status, path = self.manager.get_video_status(video_id)
        assert status == "processed"
        assert path == processed_path

    def test_extract_video_id_from_filename_patterns(self):
        """Test video ID extraction from various filename patterns."""
        # Standard pattern: videoId_channel.md
        assert (
            self.manager._extract_video_id_from_filename("abc123def45_tina huang.md")
            == "abc123def45"
        )

        # Simple pattern: videoId.md
        assert (
            self.manager._extract_video_id_from_filename("abc123def45.md")
            == "abc123def45"
        )

        # Complex pattern with title
        assert (
            self.manager._extract_video_id_from_filename("abc123def45_channel_title.md")
            == "abc123def45"
        )

    def test_is_valid_video_id(self):
        """Test video ID validation."""
        # Valid IDs
        assert self.manager._is_valid_video_id("abc123def45") is True
        assert self.manager._is_valid_video_id("A1B2C3D4E5F") is True
        assert self.manager._is_valid_video_id("test_video1") is True
        assert self.manager._is_valid_video_id("test-video1") is True

        # Invalid IDs
        assert self.manager._is_valid_video_id("abc123def") is False  # Too short
        assert self.manager._is_valid_video_id("abc123def456") is False  # Too long
        assert self.manager._is_valid_video_id("abc@123def4") is False  # Invalid char

    def test_list_videos_by_status(self):
        """Test listing videos by status."""
        # Add videos in different statuses
        video1 = "test1234567"
        video2 = "test2345678"

        # Add one to queue
        self.manager.add_to_be_downloaded(video1, "channel1")

        # Add and download another
        self.manager.add_to_be_downloaded(video2, "channel2")
        self.manager.mark_downloaded(video2, "content", "channel2")

        # Test listing
        to_be_downloaded = self.manager.list_videos_by_status("to_be_downloaded")
        assert len(to_be_downloaded) == 1
        assert to_be_downloaded[0]["video_id"] == video1

        downloaded = self.manager.list_videos_by_status("downloaded")
        assert len(downloaded) == 1
        assert downloaded[0]["video_id"] == video2

        processed = self.manager.list_videos_by_status("processed")
        assert len(processed) == 0

    def test_get_stats(self):
        """Test statistics generation."""
        # Add videos
        self.manager.add_to_be_downloaded("test1234567", "channel1")
        self.manager.add_to_be_downloaded("test2345678", "channel2")
        self.manager.mark_downloaded("test2345678", "content", "channel2")

        stats = self.manager.get_stats()

        assert stats["to_be_downloaded"] == 1
        assert stats["downloaded"] == 1
        assert stats["processed"] == 0
        assert stats["total"] == 2
        assert "channel1" in stats["channels"]
        assert "channel2" in stats["channels"]

    def test_remove_video(self):
        """Test video removal."""
        video_id = "test1234567"

        # Add video
        self.manager.add_to_be_downloaded(video_id)

        # Remove without deleting file
        result = self.manager.remove_video(video_id, delete_file=False)
        assert result["found"] is True
        assert result["file_deleted"] is False
        assert result["previous_status"] == "to_be_downloaded"

        # Remove with file deletion
        self.manager.add_to_be_downloaded(video_id)
        result = self.manager.remove_video(video_id, delete_file=True)
        assert result["found"] is True
        assert result["file_deleted"] is True
