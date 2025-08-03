#!/usr/bin/env python3
"""
Tests for StatelessYouTubeTranscriptLoader module.
"""
import pytest
import tempfile
import shutil
from pathlib import Path
from unittest.mock import patch, MagicMock

from src.transcript_loader import YouTubeTranscriptLoader


class TestYouTubeTranscriptLoader:
    
    def setup_method(self):
        """Setup test environment with temporary directory."""
        self.temp_dir = tempfile.mkdtemp()
        
        # Mock config to use our temp directory
        with patch('src.transcript_loader.Config') as mock_config:
            mock_config.return_value.get.side_effect = lambda key, default=None: {
                'storage.base_path': self.temp_dir,
                'storage.markdown_format': True,
                'transcript.default_languages': ['en'],
                'transcript.include_metadata': True,
                'transcript.clean_transcript': True
            }.get(key, default)
            
            self.loader = YouTubeTranscriptLoader()
    
    def teardown_method(self):
        """Clean up temporary directory."""
        shutil.rmtree(self.temp_dir)
    
    def test_extract_video_id_youtube_watch(self):
        """Test video ID extraction from youtube.com/watch URLs."""
        url = "https://www.youtube.com/watch?v=abc123def45"
        video_id = self.loader._extract_video_id(url)
        assert video_id == "abc123def45"
        
        # With additional parameters
        url = "https://www.youtube.com/watch?v=abc123def45&t=123s"
        video_id = self.loader._extract_video_id(url)
        assert video_id == "abc123def45"
    
    def test_extract_video_id_youtube_embed(self):
        """Test video ID extraction from youtube.com/embed URLs."""
        url = "https://www.youtube.com/embed/abc123def45"
        video_id = self.loader._extract_video_id(url)
        assert video_id == "abc123def45"
    
    def test_extract_video_id_youtu_be(self):
        """Test video ID extraction from youtu.be URLs."""
        url = "https://youtu.be/abc123def45"
        video_id = self.loader._extract_video_id(url)
        assert video_id == "abc123def45"
        
        # With parameters
        url = "https://youtu.be/abc123def45?t=123"
        video_id = self.loader._extract_video_id(url)
        assert video_id == "abc123def45"
    
    def test_extract_video_id_invalid_url(self):
        """Test video ID extraction from invalid URLs."""
        assert self.loader._extract_video_id("https://example.com") is None
        assert self.loader._extract_video_id("not-a-url") is None
    
    def test_clean_transcript_basic(self):
        """Test basic transcript cleaning."""
        raw_text = "[00:01] Hello world ♪ music ♪ [00:02] More text"
        cleaned = self.loader.clean_transcript(raw_text)
        assert "[00:01]" not in cleaned
        assert "[00:02]" not in cleaned
        assert "♪" not in cleaned
        assert "Hello world" in cleaned
        assert "More text" in cleaned
    
    def test_clean_transcript_whitespace(self):
        """Test transcript cleaning handles excessive whitespace."""
        raw_text = "Hello    world   \n\n\n   More   text"
        cleaned = self.loader.clean_transcript(raw_text)
        assert cleaned == "Hello world More text"
    
    @patch('src.transcript_loader.YouTubeTranscriptApi')
    def test_load_transcript_already_exists(self, mock_api):
        """Test loading transcript when video already exists."""
        video_id = "test12345678"
        
        # Setup existing video
        self.loader.manager.add_to_be_downloaded(video_id, "test_channel")
        content = "Test content"
        self.loader.manager.mark_downloaded(video_id, content, "test_channel")
        
        # Load transcript - should return existing info
        result = self.loader.load_transcript(video_id, force=False)
        
        assert result['already_exists'] is True
        assert result['video_id'] == video_id
        assert result['status'] == 'downloaded'
        assert result['channel'] == 'test_channel'
        
        # API should not be called
        mock_api.assert_not_called()
    
    @patch('src.transcript_loader.YouTubeTranscriptApi')
    def test_load_transcript_force_reload(self, mock_api):
        """Test force reloading transcript even when it exists."""
        video_id = "test12345678"
        
        # Setup existing video
        self.loader.manager.add_to_be_downloaded(video_id, "test_channel")
        self.loader.manager.mark_downloaded(video_id, "old content", "test_channel")
        
        # Mock API response
        mock_transcript = MagicMock()
        mock_transcript.language = "en"
        mock_transcript.is_generated = False
        mock_transcript.fetch.return_value = [
            {'text': 'Hello world', 'start': 0.0, 'duration': 2.0}
        ]
        
        mock_transcript_list = MagicMock()
        mock_transcript_list.find_transcript.return_value = mock_transcript
        mock_api.return_value.list.return_value = mock_transcript_list
        
        # Load with force=True
        result = self.loader.load_transcript(video_id, force=True, channel="new_channel")
        
        assert result['already_exists'] is False
        assert result['video_id'] == video_id
        assert result['language'] == "en"
        assert result['is_generated'] is False
        assert 'Hello world' in result['cleaned_text']
        
        # API should be called
        mock_api.return_value.list.assert_called_with(video_id)
    
    def test_get_video_info_exists(self):
        """Test getting video info for existing video."""
        video_id = "test12345678"
        channel = "test_channel"
        
        # Setup video
        self.loader.manager.add_to_be_downloaded(video_id, channel)
        
        info = self.loader.get_video_info(video_id)
        
        assert info is not None
        assert info['video_id'] == video_id
        assert info['status'] == 'to_be_downloaded'
        assert info['channel'] == channel
    
    def test_get_video_info_not_exists(self):
        """Test getting video info for non-existent video."""
        info = self.loader.get_video_info("nonexistent123")
        assert info is None
    
    def test_add_to_be_downloaded(self):
        """Test adding video to download queue."""
        video_id = "test12345678"
        channel = "test_channel"
        
        result = self.loader.add_to_be_downloaded(video_id, channel)
        assert result is True
        
        # Verify it was added
        status, _ = self.loader.manager.get_video_status(video_id)
        assert status == 'to_be_downloaded'
    
    def test_process_video_success(self):
        """Test processing video from downloaded to processed."""
        video_id = "test12345678"
        channel = "test_channel"
        
        # Setup downloaded video
        self.loader.manager.add_to_be_downloaded(video_id, channel)
        self.loader.manager.mark_downloaded(video_id, "content", channel)
        
        success, result = self.loader.process_video(video_id, channel)
        
        assert success is True
        assert isinstance(result, str)  # Should return path
        
        # Verify status changed
        status, _ = self.loader.manager.get_video_status(video_id)
        assert status == 'processed'
    
    def test_process_video_wrong_status(self):
        """Test processing video with wrong status."""
        video_id = "test12345678"
        
        # Video not downloaded
        success, result = self.loader.process_video(video_id)
        
        assert success is False
        assert "not in 'downloaded' status" in result
    
    def test_process_video_extract_channel_from_filename(self):
        """Test processing video extracts channel from filename when not specified."""
        video_id = "test12345678"
        channel = "original_channel"
        
        # Setup downloaded video
        self.loader.manager.add_to_be_downloaded(video_id, channel)
        self.loader.manager.mark_downloaded(video_id, "content", channel)
        
        # Process without specifying channel
        success, result = self.loader.process_video(video_id)
        
        assert success is True
        assert channel in result  # Should use extracted channel
    
    def test_list_processed_videos(self):
        """Test listing all processed videos."""
        # Add videos in different statuses (using realistic 11-character video IDs)
        self.loader.manager.add_to_be_downloaded("test1234567", "channel1")
        self.loader.manager.add_to_be_downloaded("test2345678", "channel2")
        self.loader.manager.mark_downloaded("test2345678", "content", "channel2")
        
        videos = self.loader.list_processed_videos()
        
        assert len(videos) == 2
        video_ids = [v['video_id'] for v in videos]
        assert "test1234567" in video_ids
        assert "test2345678" in video_ids
    
    def test_get_stats(self):
        """Test getting statistics."""
        # Add some videos (using realistic 11-character video IDs)
        self.loader.manager.add_to_be_downloaded("test1234567", "channel1")
        self.loader.manager.add_to_be_downloaded("test2345678", "channel2")
        self.loader.manager.mark_downloaded("test2345678", "content", "channel2")
        
        stats = self.loader.get_stats()
        
        assert stats['to_be_downloaded'] == 1
        assert stats['downloaded'] == 1
        assert stats['processed'] == 0
        assert stats['total'] == 2
    
    def test_save_transcript_custom_file(self):
        """Test saving transcript to custom file."""
        transcript_data = {
            'video_id': 'test12345678',
            'language': 'en',
            'is_generated': False,
            'cleaned_text': 'Test transcript content'
        }
        
        custom_file = Path(self.temp_dir) / "custom_transcript.txt"
        self.loader.save_transcript(transcript_data, str(custom_file))
        
        assert custom_file.exists()
        content = custom_file.read_text()
        assert 'test12345678' in content
        assert 'Test transcript content' in content