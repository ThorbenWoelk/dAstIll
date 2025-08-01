import unittest
import tempfile
import os
from pathlib import Path
from src.dastill.markdown_storage import MarkdownStorage


class TestMarkdownStorage(unittest.TestCase):
    def setUp(self):
        self.temp_dir = tempfile.mkdtemp()
        self.storage = MarkdownStorage(self.temp_dir, organize_by_date=False)
        self.storage_with_date = MarkdownStorage(self.temp_dir + '_date', organize_by_date=True)
    
    def tearDown(self):
        import shutil
        shutil.rmtree(self.temp_dir, ignore_errors=True)
        shutil.rmtree(self.temp_dir + '_date', ignore_errors=True)
    
    def test_sanitize_filename(self):
        """Test filename sanitization."""
        test_cases = [
            ('Normal Title', 'Normal Title'),
            ('Title with <invalid> chars', 'Title with _invalid_ chars'),
            ('Title: with colon', 'Title_ with colon'),
            ('Very long title that exceeds the fifty character limit for filenames', 
             'Very long title that exceeds the fifty character'),
            ('  Title with spaces  ', 'Title with spaces'),
            ('', ''),
        ]
        
        for input_title, expected_output in test_cases:
            with self.subTest(input_title=input_title):
                result = self.storage._sanitize_filename(input_title)
                self.assertEqual(result, expected_output)
    
    def test_generate_filename(self):
        """Test filename generation."""
        # With title
        filename = self.storage._generate_filename('dQw4w9WgXcQ', 'Test Video')
        self.assertEqual(filename, 'dQw4w9WgXcQ_Test Video.md')
        
        # Without title
        filename = self.storage._generate_filename('dQw4w9WgXcQ', '')
        self.assertEqual(filename, 'dQw4w9WgXcQ.md')
        
        # With sanitized title
        filename = self.storage._generate_filename('dQw4w9WgXcQ', 'Title: with colon')
        self.assertEqual(filename, 'dQw4w9WgXcQ_Title_ with colon.md')
    
    def test_storage_path_without_date_organization(self):
        """Test storage path generation without date organization."""
        path = self.storage._get_storage_path('dQw4w9WgXcQ', 'Test Video')
        
        expected_path = Path(self.temp_dir) / 'dQw4w9WgXcQ_Test Video.md'
        self.assertEqual(path, expected_path)
    
    def test_storage_path_with_date_organization(self):
        """Test storage path generation with date organization."""
        from datetime import datetime
        
        path = self.storage_with_date._get_storage_path('dQw4w9WgXcQ', 'Test Video')
        
        current_month = datetime.now().strftime("%Y-%m")
        expected_path = Path(self.temp_dir + '_date') / current_month / 'dQw4w9WgXcQ_Test Video.md'
        self.assertEqual(path, expected_path)
    
    def test_save_transcript(self):
        """Test saving transcript as markdown."""
        transcript_data = {
            'video_id': 'dQw4w9WgXcQ',
            'language': 'en',
            'is_generated': False,
            'title': 'Test Video',
            'duration': '3:32',
            'cleaned_text': 'This is the transcript text.',
            'formatted_text': 'This is the transcript text.'
        }
        
        file_path = self.storage.save_transcript(transcript_data)
        
        # Check that file was created
        self.assertTrue(os.path.exists(file_path))
        
        # Check file content
        with open(file_path, 'r', encoding='utf-8') as f:
            content = f.read()
        
        self.assertIn('# Test Video', content)
        self.assertIn('dQw4w9WgXcQ', content)
        self.assertIn('This is the transcript text.', content)
        self.assertIn('https://www.youtube.com/watch?v=dQw4w9WgXcQ', content)
        self.assertIn('**Language**: en', content)
        self.assertIn('**Auto-generated**: False', content)
    
    def test_save_transcript_with_summary(self):
        """Test saving transcript with summary."""
        transcript_data = {
            'video_id': 'dQw4w9WgXcQ',
            'language': 'en',
            'is_generated': True,
            'title': 'Test Video',
            'cleaned_text': 'Transcript content'
        }
        
        summary = 'This video is about testing.'
        file_path = self.storage.save_transcript(transcript_data, summary)
        
        with open(file_path, 'r', encoding='utf-8') as f:
            content = f.read()
        
        self.assertIn('## Summary', content)
        self.assertIn('This video is about testing.', content)
    
    def test_load_transcript(self):
        """Test loading transcript from file."""
        # First save a transcript
        transcript_data = {
            'video_id': 'test123',
            'language': 'en',
            'is_generated': False,
            'title': 'Test Video',
            'cleaned_text': 'Test transcript content'
        }
        
        self.storage.save_transcript(transcript_data)
        
        # Now load it back
        loaded_content = self.storage.load_transcript('test123')
        
        self.assertIsNotNone(loaded_content)
        self.assertIn('Test transcript content', loaded_content)
        self.assertIn('Test Video', loaded_content)
    
    def test_load_nonexistent_transcript(self):
        """Test loading transcript that doesn't exist."""
        result = self.storage.load_transcript('nonexistent')
        self.assertIsNone(result)
    
    def test_file_exists(self):
        """Test checking if transcript file exists."""
        # Initially should not exist
        self.assertFalse(self.storage.file_exists('test123'))
        
        # Save a transcript
        transcript_data = {
            'video_id': 'test123',
            'language': 'en',
            'is_generated': False,
            'cleaned_text': 'Test content'
        }
        
        self.storage.save_transcript(transcript_data)
        
        # Now should exist
        self.assertTrue(self.storage.file_exists('test123'))
    
    def test_markdown_format_structure(self):
        """Test that markdown content follows expected structure."""
        transcript_data = {
            'video_id': 'dQw4w9WgXcQ',
            'language': 'en',
            'is_generated': True,
            'title': 'Rick Astley - Never Gonna Give You Up',
            'duration': '3:33',
            'cleaned_text': 'Never gonna give you up, never gonna let you down'
        }
        
        content = self.storage._format_as_markdown(transcript_data)
        
        # Check structure
        lines = content.split('\n')
        
        # Title should be first line
        self.assertTrue(lines[0].startswith('# '))
        
        # Should have video information section
        self.assertIn('## Video Information', content)
        
        # Should have transcript section
        self.assertIn('## Transcript', content)
        
        # Should have footer
        self.assertIn('*Generated by dAstIll', content)
        
        # Should have proper markdown links
        self.assertIn('[https://www.youtube.com/watch?v=dQw4w9WgXcQ]', content)
    
    def test_permission_error_handling(self):
        """Test handling of permission errors during save."""
        transcript_data = {
            'video_id': 'test123',
            'language': 'en',
            'is_generated': False,
            'cleaned_text': 'Test content'
        }
        
        # Make storage directory read-only (on Unix systems)
        if os.name != 'nt':  # Skip on Windows
            os.chmod(self.temp_dir, 0o444)
            
            try:
                with self.assertRaises(IOError) as context:
                    self.storage.save_transcript(transcript_data)
                self.assertIn('Permission denied', str(context.exception))
            finally:
                # Restore permissions for cleanup
                os.chmod(self.temp_dir, 0o755)
    
    def test_unicode_handling(self):
        """Test handling of unicode characters in titles and content."""
        transcript_data = {
            'video_id': 'test123',
            'language': 'de',
            'is_generated': False,
            'title': 'Vidéo avec des caractères spéciaux: éàù',
            'cleaned_text': 'Contenu avec des émojis 🎵 et caractères spéciaux ñáéíóú'
        }
        
        file_path = self.storage.save_transcript(transcript_data)
        
        # Verify file was created and content is preserved
        with open(file_path, 'r', encoding='utf-8') as f:
            content = f.read()
        
        self.assertIn('🎵', content)
        self.assertIn('ñáéíóú', content)
        self.assertIn('éàù', content)
    
    def test_directory_creation(self):
        """Test that storage creates necessary directories."""
        new_base_path = os.path.join(self.temp_dir, 'new', 'nested', 'path')
        storage = MarkdownStorage(new_base_path, organize_by_date=True)
        
        transcript_data = {
            'video_id': 'test123',
            'language': 'en',
            'is_generated': False,
            'cleaned_text': 'Test content'
        }
        
        file_path = storage.save_transcript(transcript_data)
        
        # Verify all directories were created
        self.assertTrue(os.path.exists(os.path.dirname(file_path)))
        self.assertTrue(os.path.exists(file_path))


if __name__ == '__main__':
    unittest.main()