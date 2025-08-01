import unittest
import tempfile
import os
from pathlib import Path
from unittest.mock import Mock, patch, MagicMock
from src.dastill.youtube_loader import YouTubeTranscriptLoader


class TestYouTubeTranscriptLoader(unittest.TestCase):
    def setUp(self):
        self.temp_dir = tempfile.mkdtemp()
        self.config_path = os.path.join(self.temp_dir, 'config.json')
        self.loader = YouTubeTranscriptLoader(self.config_path)
    
    def tearDown(self):
        import shutil
        shutil.rmtree(self.temp_dir, ignore_errors=True)
    
    def test_extract_video_id_from_watch_url(self):
        """Test video ID extraction from standard YouTube watch URLs."""
        test_cases = [
            ('https://www.youtube.com/watch?v=dQw4w9WgXcQ', 'dQw4w9WgXcQ'),
            ('https://youtube.com/watch?v=dQw4w9WgXcQ', 'dQw4w9WgXcQ'),
            ('https://youtube.com/watch?v=dQw4w9WgXcQ&t=10s', 'dQw4w9WgXcQ'),
        ]
        
        for url, expected_id in test_cases:
            with self.subTest(url=url):
                result = self.loader.extract_video_id(url)
                self.assertEqual(result, expected_id)
    
    def test_extract_video_id_from_embed_url(self):
        """Test video ID extraction from YouTube embed URLs."""
        url = 'https://www.youtube.com/embed/dQw4w9WgXcQ'
        result = self.loader.extract_video_id(url)
        self.assertEqual(result, 'dQw4w9WgXcQ')
    
    def test_extract_video_id_from_youtu_be_url(self):
        """Test video ID extraction from youtu.be short URLs."""
        test_cases = [
            ('https://youtu.be/dQw4w9WgXcQ', 'dQw4w9WgXcQ'),
            ('https://www.youtu.be/dQw4w9WgXcQ', 'dQw4w9WgXcQ'),
        ]
        
        for url, expected_id in test_cases:
            with self.subTest(url=url):
                result = self.loader.extract_video_id(url)
                self.assertEqual(result, expected_id)
    
    def test_extract_video_id_invalid_url(self):
        """Test video ID extraction from invalid URLs."""
        invalid_urls = [
            'https://example.com/watch?v=dQw4w9WgXcQ',
            'https://youtube.com/invalid',
            'not_a_url',
            '',
        ]
        
        for url in invalid_urls:
            with self.subTest(url=url):
                result = self.loader.extract_video_id(url)
                self.assertIsNone(result)
    
    def test_clean_transcript(self):
        """Test transcript cleaning functionality."""
        test_cases = [
            ('[Music] Hello world [Applause]', 'Hello world'),
            ('Hello ♪♪ world ♪', 'Hello world'),
            ('Hello    world   \n\n  test', 'Hello world test'),
            ('  [Inaudible]  Hello  world  ', 'Hello world'),
        ]
        
        for input_text, expected_output in test_cases:
            with self.subTest(input_text=input_text):
                result = self.loader.clean_transcript(input_text)
                self.assertEqual(result, expected_output)
    
    @patch('src.dastill.youtube_loader.YouTubeTranscriptApi')
    def test_load_transcript_success(self, mock_api_class):
        """Test successful transcript loading."""
        # Mock the API response
        mock_api = Mock()
        mock_api_class.return_value = mock_api
        
        mock_transcript_list = Mock()
        mock_api.list.return_value = mock_transcript_list
        
        mock_transcript = Mock()
        mock_transcript.language = 'en'
        mock_transcript.is_generated = False
        mock_transcript.fetch.return_value = [
            {'text': 'Hello world', 'start': 0.0, 'duration': 2.0}
        ]
        
        mock_transcript_list.find_transcript.return_value = mock_transcript
        
        # Mock the formatter
        with patch.object(self.loader.formatter, 'format_transcript') as mock_format:
            mock_format.return_value = 'Hello world'
            
            result = self.loader.load_transcript('dQw4w9WgXcQ', force=True, save_markdown=False)
            
            self.assertEqual(result['video_id'], 'dQw4w9WgXcQ')
            self.assertEqual(result['language'], 'en')
            self.assertFalse(result['is_generated'])
            self.assertEqual(result['cleaned_text'], 'Hello world')
    
    @patch('src.dastill.youtube_loader.YouTubeTranscriptApi')
    def test_load_transcript_already_processed(self, mock_api_class):
        """Test loading transcript for already processed video."""
        # Add a video to tracker
        test_data = {
            'video_id': 'test123',
            'language': 'en',
            'is_generated': False,
            'file_path': 'test_path.md'
        }
        self.loader.tracker.add_video('test123', test_data, 'test_path.md')
        
        result = self.loader.load_transcript('test123', force=False)
        
        self.assertTrue(result['already_exists'])
        self.assertEqual(result['video_id'], 'test123')
    
    @patch('src.dastill.youtube_loader.YouTubeTranscriptApi')
    def test_load_transcript_api_error(self, mock_api_class):
        """Test transcript loading when API throws an error."""
        mock_api = Mock()
        mock_api_class.return_value = mock_api
        mock_api.list.side_effect = Exception("API Error")
        
        with self.assertRaises(Exception) as context:
            self.loader.load_transcript('invalid_video', force=True, save_markdown=False)
        
        self.assertIn("Failed to load transcript", str(context.exception))
    
    def test_remove_video_success(self):
        """Test successful video removal."""
        # Add a video first
        test_data = {
            'video_id': 'test123',
            'language': 'en',
            'is_generated': False,
            'file_path': os.path.join(self.temp_dir, 'test.md')
        }
        
        # Create a test file
        with open(test_data['file_path'], 'w') as f:
            f.write('test content')
        
        self.loader.tracker.add_video('test123', test_data, test_data['file_path'])
        
        # Remove video with file deletion
        result = self.loader.remove_video('test123', delete_file=True)
        
        self.assertTrue(result['video_removed_from_tracker'])
        self.assertTrue(result['file_deleted'])
        self.assertIsNone(result['deletion_error'])
        self.assertFalse(os.path.exists(test_data['file_path']))
    
    def test_remove_video_file_not_found(self):
        """Test video removal when file doesn't exist."""
        # Add a video with non-existent file
        test_data = {
            'video_id': 'test123',
            'language': 'en',
            'is_generated': False,
            'file_path': '/nonexistent/path.md'
        }
        
        self.loader.tracker.add_video('test123', test_data, test_data['file_path'])
        
        result = self.loader.remove_video('test123', delete_file=True)
        
        self.assertTrue(result['video_removed_from_tracker'])
        self.assertFalse(result['file_deleted'])
        self.assertIn('File not found', result['deletion_error'])
    
    def test_remove_video_not_found(self):
        """Test removing video that doesn't exist in tracker."""
        result = self.loader.remove_video('nonexistent')
        
        self.assertFalse(result['video_removed_from_tracker'])
        self.assertFalse(result['file_deleted'])
        self.assertIsNone(result['deletion_error'])


if __name__ == '__main__':
    unittest.main()