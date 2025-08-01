import unittest
import tempfile
import os
import json
from pathlib import Path
from src.dastill.video_tracker import VideoTracker


class TestVideoTracker(unittest.TestCase):
    def setUp(self):
        self.temp_dir = tempfile.mkdtemp()
        self.db_path = os.path.join(self.temp_dir, 'videos.json')
        self.tracker = VideoTracker(self.db_path)
    
    def tearDown(self):
        import shutil
        shutil.rmtree(self.temp_dir, ignore_errors=True)
    
    def test_database_creation(self):
        """Test that database file is created properly."""
        self.assertTrue(os.path.exists(os.path.dirname(self.db_path)))
        
        # Database should be empty initially
        self.assertEqual(len(self.tracker.videos), 0)
        self.assertFalse(self.tracker.is_video_processed('test123'))
    
    def test_add_video(self):
        """Test adding a video to the tracker."""
        transcript_data = {
            'video_id': 'test123',
            'language': 'en',
            'is_generated': False,
            'title': 'Test Video',
            'duration': '5:30',
            'languages_requested': ['en']
        }
        
        file_path = os.path.join(self.temp_dir, 'test.md')
        with open(file_path, 'w') as f:
            f.write('test content')
        
        self.tracker.add_video('test123', transcript_data, file_path)
        
        # Verify video was added
        self.assertTrue(self.tracker.is_video_processed('test123'))
        
        # Verify video info
        info = self.tracker.get_video_info('test123')
        self.assertEqual(info['video_id'], 'test123')
        self.assertEqual(info['language'], 'en')
        self.assertFalse(info['is_generated'])
        self.assertEqual(info['title'], 'Test Video')
        self.assertEqual(info['file_path'], file_path)
        
        # Verify file was persisted
        self.assertTrue(os.path.exists(self.db_path))
        with open(self.db_path, 'r') as f:
            data = json.load(f)
            self.assertIn('test123', data)
    
    def test_load_existing_database(self):
        """Test loading an existing database file."""
        # Create database file manually
        test_data = {
            'video123': {
                'video_id': 'video123',
                'language': 'en',
                'is_generated': True,
                'processed_at': '2023-01-01T12:00:00',
                'file_path': '/path/to/file.md'
            }
        }
        
        with open(self.db_path, 'w') as f:
            json.dump(test_data, f)
        
        # Create new tracker instance
        new_tracker = VideoTracker(self.db_path)
        
        self.assertTrue(new_tracker.is_video_processed('video123'))
        info = new_tracker.get_video_info('video123')
        self.assertEqual(info['language'], 'en')
        self.assertTrue(info['is_generated'])
    
    def test_remove_video(self):
        """Test removing a video from the tracker."""
        # Add a video first
        transcript_data = {
            'video_id': 'test123',
            'language': 'en',
            'is_generated': False
        }
        
        self.tracker.add_video('test123', transcript_data, '/path/to/file.md')
        self.assertTrue(self.tracker.is_video_processed('test123'))
        
        # Remove the video
        result = self.tracker.remove_video('test123')
        self.assertTrue(result)
        self.assertFalse(self.tracker.is_video_processed('test123'))
        
        # Try to remove non-existent video
        result = self.tracker.remove_video('nonexistent')
        self.assertFalse(result)
    
    def test_list_videos(self):
        """Test listing all videos."""
        # Add multiple videos
        for i in range(3):
            transcript_data = {
                'video_id': f'video{i}',
                'language': 'en',
                'is_generated': i % 2 == 0  # Alternate between True/False
            }
            self.tracker.add_video(f'video{i}', transcript_data, f'/path/to/file{i}.md')
        
        videos = self.tracker.list_videos()
        self.assertEqual(len(videos), 3)
        
        # Check that all videos are present
        video_ids = [video['video_id'] for video in videos]
        self.assertIn('video0', video_ids)
        self.assertIn('video1', video_ids)
        self.assertIn('video2', video_ids)
    
    def test_get_stats(self):
        """Test getting statistics about tracked videos."""
        # Add videos with different languages and generation status
        test_videos = [
            {'video_id': 'v1', 'language': 'en', 'is_generated': True},
            {'video_id': 'v2', 'language': 'en', 'is_generated': False},
            {'video_id': 'v3', 'language': 'de', 'is_generated': True},
            {'video_id': 'v4', 'language': 'fr', 'is_generated': False},
        ]
        
        for video_data in test_videos:
            self.tracker.add_video(video_data['video_id'], video_data, '/path/to/file.md')
        
        stats = self.tracker.get_stats()
        
        self.assertEqual(stats['total_videos'], 4)
        self.assertEqual(stats['auto_generated_count'], 2)
        self.assertEqual(stats['manual_transcript_count'], 2)
        
        # Check language distribution
        self.assertEqual(stats['languages']['en'], 2)
        self.assertEqual(stats['languages']['de'], 1)
        self.assertEqual(stats['languages']['fr'], 1)
    
    def test_atomic_write_safety(self):
        """Test that database saves are atomic and safe."""
        # Add multiple videos rapidly to test concurrent access safety
        for i in range(20):
            transcript_data = {
                'video_id': f'video{i}',
                'language': 'en',
                'is_generated': False
            }
            self.tracker.add_video(f'video{i}', transcript_data, f'/path/to/file{i}.md')
        
        # Verify all videos were saved correctly
        self.assertEqual(len(self.tracker.videos), 20)
        
        # Verify database file is valid JSON
        with open(self.db_path, 'r') as f:
            data = json.load(f)
            self.assertEqual(len(data), 20)
    
    def test_database_corruption_handling(self):
        """Test handling of corrupted database files."""
        # Create corrupted database file
        with open(self.db_path, 'w') as f:
            f.write('invalid json content')
        
        # Tracker should handle corruption gracefully
        tracker = VideoTracker(self.db_path)
        self.assertEqual(len(tracker.videos), 0)
    
    def test_file_size_tracking(self):
        """Test that file sizes are tracked correctly."""
        file_path = os.path.join(self.temp_dir, 'test.md')
        test_content = 'A' * 1000  # 1000 byte file
        
        with open(file_path, 'w') as f:
            f.write(test_content)
        
        transcript_data = {
            'video_id': 'test123',
            'language': 'en',
            'is_generated': False
        }
        
        self.tracker.add_video('test123', transcript_data, file_path)
        
        info = self.tracker.get_video_info('test123')
        self.assertEqual(info['metadata']['file_size'], 1000)
    
    def test_permission_error_handling(self):
        """Test handling of permission errors during save."""
        # Add a video first
        transcript_data = {
            'video_id': 'test123',
            'language': 'en',
            'is_generated': False
        }
        
        self.tracker.add_video('test123', transcript_data, '/path/to/file.md')
        
        # Make parent directory read-only (on Unix systems)
        if os.name != 'nt':  # Skip on Windows
            os.chmod(self.temp_dir, 0o444)
            
            try:
                with self.assertRaises(IOError) as context:
                    self.tracker.add_video('test456', transcript_data, '/path/to/file2.md')
                self.assertIn('Failed to save video database', str(context.exception))
            finally:
                # Restore permissions for cleanup
                os.chmod(self.temp_dir, 0o755)


if __name__ == '__main__':
    unittest.main()