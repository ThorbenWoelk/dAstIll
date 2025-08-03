import unittest
import tempfile
import os
from unittest.mock import Mock, patch
from src.transcript_loader import YouTubeTranscriptLoader


class TestIntegration(unittest.TestCase):
    """Integration tests for the complete transcript loading workflow."""
    
    def setUp(self):
        self.temp_dir = tempfile.mkdtemp()
        self.config_path = os.path.join(self.temp_dir, 'config.json')
        self.loader = YouTubeTranscriptLoader(self.config_path)
    
    def tearDown(self):
        import shutil
        shutil.rmtree(self.temp_dir, ignore_errors=True)
    
    @patch('src.transcript_loader.YouTubeTranscriptApi')
    def test_complete_transcript_workflow(self, mock_api_class):
        """Test complete workflow from URL to saved markdown file."""
        # Setup mock API
        mock_api = Mock()
        mock_api_class.return_value = mock_api
        
        mock_transcript_list = Mock()
        mock_api.list.return_value = mock_transcript_list
        
        mock_transcript = Mock()
        mock_transcript.language = 'en'
        mock_transcript.is_generated = False
        mock_transcript.fetch.return_value = [
            {'text': '[Music] Hello world! [Applause]', 'start': 0.0, 'duration': 2.0},
            {'text': 'This is a test transcript.', 'start': 2.0, 'duration': 3.0}
        ]
        
        mock_transcript_list.find_transcript.return_value = mock_transcript
        
        # Mock the formatter
        with patch.object(self.loader.formatter, 'format_transcript') as mock_format:
            mock_format.return_value = '[Music] Hello world! [Applause]\nThis is a test transcript.'
            
            # Test the complete workflow
            result = self.loader.load_transcript(
                'https://www.youtube.com/watch?v=dQw4w9WgXcQ',
                languages=['en'],
                force=True,
                save_markdown=True
            )
            
            # Verify the result
            self.assertEqual(result['video_id'], 'dQw4w9WgXcQ')
            self.assertEqual(result['language'], 'en')
            self.assertFalse(result['is_generated'])
            self.assertEqual(result['cleaned_text'], 'Hello world!\nThis is a test transcript.')
            
            # Verify markdown file was created
            file_path = result['file_path']
            self.assertTrue(os.path.exists(file_path))
            
            # Verify file content
            with open(file_path, 'r', encoding='utf-8') as f:
                content = f.read()
            
            self.assertIn('dQw4w9WgXcQ', content)
            self.assertIn('Hello world!', content)
            self.assertIn('This is a test transcript.', content)
            self.assertIn('**Language**: en', content)
            
            # Verify video was tracked
            self.assertTrue(self.loader.tracker.is_video_processed('dQw4w9WgXcQ'))
            
            # Test loading the same video again (should use cache)
            result2 = self.loader.load_transcript(
                'https://www.youtube.com/watch?v=dQw4w9WgXcQ',
                force=False
            )
            
            self.assertTrue(result2['already_exists'])
            self.assertEqual(result2['video_id'], 'dQw4w9WgXcQ')
    
    @patch('src.transcript_loader.YouTubeTranscriptApi')
    def test_fallback_to_generated_transcript(self, mock_api_class):
        """Test fallback to auto-generated transcript when manual is not available."""
        mock_api = Mock()
        mock_api_class.return_value = mock_api
        
        mock_transcript_list = Mock()
        mock_api.list.return_value = mock_transcript_list
        
        # First call to find_transcript raises exception (no manual transcript)
        mock_transcript_list.find_transcript.side_effect = Exception("No manual transcript")
        
        # Generated transcript is available
        mock_generated_transcript = Mock()
        mock_generated_transcript.language = 'en'
        mock_generated_transcript.is_generated = True
        mock_generated_transcript.fetch.return_value = [
            {'text': 'Auto-generated transcript content', 'start': 0.0, 'duration': 2.0}
        ]
        
        mock_transcript_list.find_generated_transcript.return_value = mock_generated_transcript
        
        with patch.object(self.loader.formatter, 'format_transcript') as mock_format:
            mock_format.return_value = 'Auto-generated transcript content'
            
            result = self.loader.load_transcript(
                'test_video_id',
                languages=['en'],
                force=True,
                save_markdown=True
            )
            
            self.assertTrue(result['is_generated'])
            self.assertEqual(result['language'], 'en')
            self.assertIn('Auto-generated', result['cleaned_text'])
    
    def test_config_integration(self):
        """Test that configuration settings are properly applied."""
        # Modify configuration
        self.loader.config.set('transcript.default_languages', ['de', 'en'])
        self.loader.config.set('storage.organize_by_date', False)
        
        # Create a new loader instance to test config persistence
        new_loader = YouTubeTranscriptLoader(self.config_path)
        
        self.assertEqual(new_loader.config.get('transcript.default_languages'), ['de', 'en'])
        self.assertFalse(new_loader.config.get('storage.organize_by_date'))
    
    def test_video_management_workflow(self):
        """Test complete video management workflow."""
        # Add some test videos to tracker manually
        test_videos = [
            {
                'video_id': 'video1',
                'language': 'en',
                'is_generated': False,
                'title': 'Test Video 1'
            },
            {
                'video_id': 'video2',
                'language': 'de',
                'is_generated': True,
                'title': 'Test Video 2'
            }
        ]
        
        # Create test files
        for video in test_videos:
            file_path = os.path.join(self.temp_dir, f"{video['video_id']}.md")
            with open(file_path, 'w') as f:
                f.write(f"Content for {video['video_id']}")
            
            self.loader.tracker.add_video(video['video_id'], video, file_path)
        
        # Test listing videos
        videos = self.loader.list_processed_videos()
        self.assertEqual(len(videos), 2)
        
        # Test getting stats
        stats = self.loader.get_stats()
        self.assertEqual(stats['total_videos'], 2)
        self.assertEqual(stats['auto_generated_count'], 1)
        self.assertEqual(stats['manual_transcript_count'], 1)
        
        # Test getting video info
        info = self.loader.get_video_info('video1')
        self.assertEqual(info['video_id'], 'video1')
        self.assertEqual(info['language'], 'en')
        
        # Test removing video with file deletion
        result = self.loader.remove_video('video1', delete_file=True)
        self.assertTrue(result['video_removed_from_tracker'])
        self.assertTrue(result['file_deleted'])
        
        # Verify video was removed
        self.assertFalse(self.loader.tracker.is_video_processed('video1'))
        
        # Test removing non-existent video
        result = self.loader.remove_video('nonexistent')
        self.assertFalse(result['video_removed_from_tracker'])
    
    def test_error_handling_integration(self):
        """Test error handling in integrated scenarios."""
        # Test with invalid video URL
        with self.assertRaises(ValueError) as context:
            self.loader.load_transcript('https://invalid.com/video')
        
        self.assertIn('Could not extract video ID', str(context.exception))
    
    @patch('src.transcript_loader.YouTubeTranscriptApi')
    def test_language_preference_handling(self, mock_api_class):
        """Test language preference handling in transcript loading."""
        mock_api = Mock()
        mock_api_class.return_value = mock_api
        
        mock_transcript_list = Mock()
        mock_api.list.return_value = mock_transcript_list
        
        # Mock different language transcripts
        mock_de_transcript = Mock()
        mock_de_transcript.language = 'de'
        mock_de_transcript.is_generated = False
        mock_de_transcript.fetch.return_value = [
            {'text': 'Deutscher Text', 'start': 0.0, 'duration': 2.0}
        ]
        
        # First language (de) is found
        mock_transcript_list.find_transcript.return_value = mock_de_transcript
        
        with patch.object(self.loader.formatter, 'format_transcript') as mock_format:
            mock_format.return_value = 'Deutscher Text'
            
            result = self.loader.load_transcript(
                'test_video',
                languages=['de', 'en'],  # German preferred
                force=True,
                save_markdown=False
            )
            
            self.assertEqual(result['language'], 'de')
            self.assertEqual(result['cleaned_text'], 'Deutscher Text')
            
            # Verify the API was called with German first
            mock_transcript_list.find_transcript.assert_called_with(['de'])


if __name__ == '__main__':
    unittest.main()