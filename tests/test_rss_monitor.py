"""Tests for RSS channel monitoring functionality."""

import unittest
from unittest.mock import Mock, patch, MagicMock
from src.rss_monitor import RSSChannelMonitor, VideoInfo


class TestRSSChannelMonitor(unittest.TestCase):
    """Test cases for RSSChannelMonitor."""
    
    def setUp(self):
        """Set up test fixtures."""
        self.monitor = RSSChannelMonitor()
    
    def test_video_info_creation(self):
        """Test VideoInfo dataclass creation."""
        video = VideoInfo(
            video_id="test123",
            title="Test Video",
            published="2024-01-01T00:00:00Z",
            channel_name="Test Channel",
            channel_id="UC_test123",
            url=""
        )
        
        # URL should be auto-generated if empty
        self.assertEqual(video.url, "https://www.youtube.com/watch?v=test123")
        
        # Test with provided URL
        video_with_url = VideoInfo(
            video_id="test456",
            title="Test Video 2",
            published="2024-01-01T00:00:00Z",
            channel_name="Test Channel",
            channel_id="UC_test123",
            url="https://custom.url"
        )
        self.assertEqual(video_with_url.url, "https://custom.url")
    
    @patch('requests.Session.get')
    def test_get_latest_videos_success(self, mock_get):
        """Test successful RSS feed parsing."""
        # Mock RSS response
        mock_response = Mock()
        mock_response.status_code = 200
        mock_response.content = '''<?xml version="1.0" encoding="UTF-8"?>
        <feed xmlns="http://www.w3.org/2005/Atom" xmlns:yt="http://www.youtube.com/xml/schemas/2015">
            <entry>
                <yt:videoId>video123</yt:videoId>
                <title>Test Video Title</title>
                <published>2024-01-01T00:00:00Z</published>
                <author>
                    <name>Test Channel</name>
                </author>
            </entry>
            <entry>
                <yt:videoId>video456</yt:videoId>
                <title>Another Video</title>
                <published>2024-01-02T00:00:00Z</published>
                <author>
                    <name>Test Channel</name>
                </author>
            </entry>
        </feed>'''.encode('utf-8')
        mock_get.return_value = mock_response
        
        videos = self.monitor.get_latest_videos("UC_test123", limit=10)
        
        self.assertEqual(len(videos), 2)
        self.assertEqual(videos[0].video_id, "video123")
        self.assertEqual(videos[0].title, "Test Video Title")
        self.assertEqual(videos[0].channel_name, "Test Channel")
        self.assertEqual(videos[0].url, "https://www.youtube.com/watch?v=video123")
        
        self.assertEqual(videos[1].video_id, "video456")
        self.assertEqual(videos[1].title, "Another Video")
    
    @patch('requests.Session.get')
    def test_get_latest_videos_http_error(self, mock_get):
        """Test handling of HTTP errors."""
        mock_response = Mock()
        mock_response.status_code = 404
        mock_get.return_value = mock_response
        
        videos = self.monitor.get_latest_videos("UC_invalid", limit=10)
        self.assertEqual(videos, [])
    
    @patch('requests.Session.get')
    def test_get_latest_videos_network_error(self, mock_get):
        """Test handling of network errors."""
        mock_get.side_effect = Exception("Network error")
        
        videos = self.monitor.get_latest_videos("UC_test123", limit=10)
        self.assertEqual(videos, [])
    
    @patch('requests.Session.get')
    def test_resolve_channel_id_success(self, mock_get):
        """Test successful channel ID resolution."""
        mock_response = Mock()
        mock_response.status_code = 200
        mock_response.text = '''
        <html>
        <head>
            <meta property="og:url" content="https://www.youtube.com/channel/UC_resolved123">
        </head>
        <body>Channel content</body>
        </html>
        '''
        mock_get.return_value = mock_response
        
        channel_id = self.monitor.resolve_channel_id("@testchannel")
        self.assertEqual(channel_id, "UC_resolved123")
    
    @patch('requests.Session.get')
    def test_resolve_channel_id_multiple_patterns(self, mock_get):
        """Test channel ID resolution with different patterns."""
        mock_response = Mock()
        mock_response.status_code = 200
        mock_response.text = '''
        <script>
        var ytInitialData = {"channelId":"UC_pattern123"};
        </script>
        '''
        mock_get.return_value = mock_response
        
        channel_id = self.monitor.resolve_channel_id("testchannel")
        self.assertEqual(channel_id, "UC_pattern123")
    
    @patch('requests.Session.get')
    def test_resolve_channel_id_not_found(self, mock_get):
        """Test channel ID resolution when not found."""
        mock_response = Mock()
        mock_response.status_code = 200
        mock_response.text = "<html><body>No channel ID here</body></html>"
        mock_get.return_value = mock_response
        
        channel_id = self.monitor.resolve_channel_id("@nonexistent")
        self.assertIsNone(channel_id)
    
    @patch('requests.Session.get')
    def test_resolve_channel_id_http_error(self, mock_get):
        """Test channel ID resolution with HTTP error."""
        mock_response = Mock()
        mock_response.status_code = 404
        mock_get.return_value = mock_response
        
        channel_id = self.monitor.resolve_channel_id("@notfound")
        self.assertIsNone(channel_id)
    
    @patch('requests.Session.get')
    def test_test_rss_feed_success(self, mock_get):
        """Test RSS feed accessibility check."""
        mock_response = Mock()
        mock_response.status_code = 200
        mock_get.return_value = mock_response
        
        result = self.monitor.test_rss_feed("UC_test123")
        self.assertTrue(result)
    
    @patch('requests.Session.get')
    def test_test_rss_feed_failure(self, mock_get):
        """Test RSS feed accessibility check failure."""
        mock_response = Mock()
        mock_response.status_code = 404
        mock_get.return_value = mock_response
        
        result = self.monitor.test_rss_feed("UC_invalid")
        self.assertFalse(result)
    
    @patch('requests.Session.get')
    def test_verify_channel_exists_success(self, mock_get):
        """Test channel existence verification."""
        mock_response = Mock()
        mock_response.status_code = 200
        mock_response.text = "<title>Test Channel - YouTube</title>"
        mock_get.return_value = mock_response
        
        result = self.monitor.verify_channel_exists("@testchannel")
        
        self.assertTrue(result["exists"])
        self.assertEqual(result["name"], "Test Channel")
        self.assertEqual(result["handle"], "@testchannel")
        self.assertIn("youtube.com/@testchannel", result["url"])
    
    @patch('requests.Session.get')
    def test_verify_channel_exists_not_found(self, mock_get):
        """Test channel existence verification when not found."""
        mock_response = Mock()
        mock_response.status_code = 404
        mock_get.return_value = mock_response
        
        result = self.monitor.verify_channel_exists("@nonexistent")
        
        self.assertFalse(result["exists"])
        self.assertIn("error", result)
    
    def test_handle_formatting(self):
        """Test handle formatting in various methods."""
        # Test with and without @ prefix
        with patch('requests.Session.get') as mock_get:
            mock_response = Mock()
            mock_response.status_code = 200
            mock_response.text = '<meta property="og:url" content="https://www.youtube.com/channel/UC_test123">'
            mock_get.return_value = mock_response
            
            # Both should work the same way
            result1 = self.monitor.resolve_channel_id("@testchannel")
            result2 = self.monitor.resolve_channel_id("testchannel")
            
            self.assertEqual(result1, result2)


if __name__ == '__main__':
    unittest.main()