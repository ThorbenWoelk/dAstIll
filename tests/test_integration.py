import os
import tempfile
import unittest

from src.transcript_loader import YouTubeTranscriptLoader


class TestIntegration(unittest.TestCase):
    """Integration tests for the complete transcript loading workflow."""

    def setUp(self):
        self.temp_dir = tempfile.mkdtemp()
        self.config_path = os.path.join(self.temp_dir, "config.json")

        # Create config with temp directory as base path
        from config.config import Config

        config = Config(self.config_path)
        config.set("storage.base_path", self.temp_dir)

        self.loader = YouTubeTranscriptLoader(self.config_path)

    def tearDown(self):
        import shutil

        shutil.rmtree(self.temp_dir, ignore_errors=True)

    def test_config_integration(self):
        """Test that configuration settings are properly applied."""
        # Modify configuration
        self.loader.config.set("transcript.default_languages", ["de", "en"])
        self.loader.config.set("storage.organize_by_date", False)

        # Create a new loader instance to test config persistence
        new_loader = YouTubeTranscriptLoader(self.config_path)

        self.assertEqual(
            new_loader.config.get("transcript.default_languages"), ["de", "en"]
        )
        self.assertFalse(new_loader.config.get("storage.organize_by_date"))

    def test_video_management_workflow(self):
        """Test complete video management workflow."""
        # Add some test videos using the new stateless file-based system
        test_videos = [
            {"video_id": "test1234567", "channel": "test_channel1"},
            {"video_id": "test2345678", "channel": "test_channel2"},
        ]

        # Add videos using the new file manager API
        for video in test_videos:
            # Add to be downloaded first
            self.loader.manager.add_to_be_downloaded(
                video["video_id"], video["channel"]
            )
            # Mark as downloaded with content
            self.loader.manager.mark_downloaded(
                video["video_id"], f"Content for {video['video_id']}", video["channel"]
            )

        # Test listing videos
        videos = self.loader.list_processed_videos()
        self.assertEqual(len(videos), 2)

        # Test getting stats
        stats = self.loader.get_stats()
        self.assertEqual(stats["total"], 2)
        self.assertEqual(stats["downloaded"], 2)
        self.assertEqual(stats["to_be_downloaded"], 0)
        self.assertEqual(stats["processed"], 0)

        # Test getting video info
        info = self.loader.get_video_info("test1234567")
        self.assertEqual(info["video_id"], "test1234567")
        self.assertEqual(info["status"], "downloaded")

        # Test removing video with file deletion
        result = self.loader.manager.remove_video("test1234567", delete_file=True)
        self.assertTrue(result["found"])
        self.assertTrue(result["file_deleted"])

        # Verify video was removed
        status, _ = self.loader.manager.get_video_status("test1234567")
        self.assertEqual(status, "not_downloaded")

        # Test removing non-existent video
        result = self.loader.manager.remove_video("nonexistent123")
        self.assertFalse(result["found"])

    def test_error_handling_integration(self):
        """Test error handling in integrated scenarios."""
        # Test with invalid video URL
        with self.assertRaises(ValueError) as context:
            self.loader.load_transcript("https://invalid.com/video")

        self.assertIn("Could not extract video ID", str(context.exception))


if __name__ == "__main__":
    unittest.main()
