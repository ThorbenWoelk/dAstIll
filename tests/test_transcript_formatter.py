"""Tests for transcript formatter functionality."""

import tempfile
import unittest
from pathlib import Path

from src.transcript_formatter import TranscriptFormatter


class TestTranscriptFormatter(unittest.TestCase):
    """Test cases for TranscriptFormatter."""

    def setUp(self):
        """Set up test fixtures."""
        self.temp_dir = tempfile.mkdtemp()
        self.formatter = TranscriptFormatter(self.temp_dir)

    def test_init(self):
        """Test formatter initialization."""
        self.assertEqual(self.formatter.base_path, Path(self.temp_dir))

    def test_sanitize_channel_name_valid(self):
        """Test channel name sanitization with valid names."""
        self.assertEqual(
            self.formatter._sanitize_channel_name("TestChannel"), "TestChannel"
        )
        self.assertEqual(
            self.formatter._sanitize_channel_name("Channel123"), "Channel123"
        )
        self.assertEqual(
            self.formatter._sanitize_channel_name("Test-Channel_Name"),
            "Test-Channel_Name",
        )

    def test_sanitize_channel_name_invalid_chars(self):
        """Test channel name sanitization removes invalid characters."""
        self.assertEqual(
            self.formatter._sanitize_channel_name("Test<>Channel"), "TestChannel"
        )
        self.assertEqual(
            self.formatter._sanitize_channel_name('Test"|?*Channel'), "TestChannel"
        )
        self.assertEqual(
            self.formatter._sanitize_channel_name("Test:Channel"), "TestChannel"
        )

    def test_sanitize_channel_name_path_traversal(self):
        """Test channel name sanitization prevents path traversal."""
        self.assertEqual(
            self.formatter._sanitize_channel_name("../malicious"), "malicious"
        )
        self.assertEqual(
            self.formatter._sanitize_channel_name("..\\\\malicious"), "malicious"
        )
        self.assertEqual(
            self.formatter._sanitize_channel_name("test/../path"), "testpath"
        )
        self.assertEqual(
            self.formatter._sanitize_channel_name("test/sub/path"), "testsubpath"
        )

    def test_sanitize_channel_name_empty_or_whitespace(self):
        """Test channel name sanitization with empty or whitespace input."""
        self.assertEqual(self.formatter._sanitize_channel_name(""), "unknown")
        self.assertEqual(self.formatter._sanitize_channel_name("   "), "unknown")
        self.assertEqual(self.formatter._sanitize_channel_name(None), "unknown")
        self.assertEqual(self.formatter._sanitize_channel_name("   test   "), "test")

    def test_sanitize_channel_name_length_limit(self):
        """Test channel name sanitization respects length limits."""
        long_name = "a" * 150
        result = self.formatter._sanitize_channel_name(long_name)
        self.assertEqual(len(result), 100)
        self.assertEqual(result, "a" * 100)

    def test_sanitize_channel_name_becomes_empty(self):
        """Test channel name that becomes empty after sanitization."""
        self.assertEqual(self.formatter._sanitize_channel_name("../"), "unknown")
        self.assertEqual(self.formatter._sanitize_channel_name("<>|?*"), "unknown")

    def test_sanitize_filename_valid(self):
        """Test filename sanitization with valid names."""
        self.assertEqual(self.formatter._sanitize_filename("test_file"), "test_file")
        self.assertEqual(
            self.formatter._sanitize_filename("TestFile123"), "TestFile123"
        )

    def test_sanitize_filename_invalid_chars(self):
        """Test filename sanitization removes invalid characters."""
        self.assertEqual(self.formatter._sanitize_filename("test<>file"), "test__file")
        self.assertEqual(
            self.formatter._sanitize_filename('test"|?*file'), "test____file"
        )
        self.assertEqual(self.formatter._sanitize_filename("test:file"), "test_file")
        self.assertEqual(self.formatter._sanitize_filename("test/file"), "test_file")
        self.assertEqual(
            self.formatter._sanitize_filename("test\\\\file"), "test__file"
        )

    def test_sanitize_filename_empty(self):
        """Test filename sanitization with empty input."""
        self.assertEqual(self.formatter._sanitize_filename(""), "")
        self.assertEqual(self.formatter._sanitize_filename(None), "")

    def test_sanitize_filename_length_limit(self):
        """Test filename sanitization respects length limits."""
        long_name = "a" * 60
        result = self.formatter._sanitize_filename(long_name)
        self.assertEqual(len(result), 50)
        self.assertEqual(result, "a" * 50)

    def test_sanitize_filename_whitespace(self):
        """Test filename sanitization handles whitespace."""
        self.assertEqual(
            self.formatter._sanitize_filename("  test file  "), "test file"
        )

    def test_generate_filename_full_info(self):
        """Test filename generation with all information."""
        result = self.formatter._generate_filename(
            "ABC123", "Test Video", "TestChannel"
        )
        self.assertEqual(result, "ABC123_TestChannel_Test Video.md")

    def test_generate_filename_no_title(self):
        """Test filename generation without title."""
        result = self.formatter._generate_filename("ABC123", "", "TestChannel")
        self.assertEqual(result, "ABC123_TestChannel.md")

        result = self.formatter._generate_filename("ABC123", None, "TestChannel")
        self.assertEqual(result, "ABC123_TestChannel.md")

    def test_generate_filename_unknown_channel(self):
        """Test filename generation with unknown channel."""
        result = self.formatter._generate_filename("ABC123", "Test Video", "unknown")
        self.assertEqual(result, "ABC123_Test Video.md")

        result = self.formatter._generate_filename("ABC123", "", "unknown")
        self.assertEqual(result, "ABC123.md")

    def test_generate_filename_sanitization(self):
        """Test filename generation applies sanitization."""
        result = self.formatter._generate_filename(
            "ABC123", "Test<>Video", "Test|Channel"
        )
        self.assertEqual(result, "ABC123_TestChannel_Test__Video.md")

    def test_format_transcript_content_minimal(self):
        """Test transcript content formatting with minimal data."""
        transcript_data = {
            "video_id": "ABC123",
            "cleaned_text": "This is a test transcript.",
        }

        result = self.formatter.format_transcript_content(transcript_data)

        # Check basic structure
        self.assertIn("# YouTube Video ABC123", result)
        self.assertIn("- **Video ID**: ABC123", result)
        self.assertIn("https://www.youtube.com/watch?v=ABC123", result)
        self.assertIn("This is a test transcript.", result)
        self.assertIn("*Generated by dAstIll - YouTube Transcript Loader*", result)

    def test_format_transcript_content_full_data(self):
        """Test transcript content formatting with full data."""
        transcript_data = {
            "video_id": "ABC123",
            "title": "Test Video Title",
            "language": "en",
            "is_generated": False,
            "duration": "10:30",
            "cleaned_text": "This is a test transcript.",
            "formatted_text": "Raw transcript text",
        }

        result = self.formatter.format_transcript_content(transcript_data)

        # Check all fields are included
        self.assertIn("# Test Video Title", result)
        self.assertIn("- **Video ID**: ABC123", result)
        self.assertIn("- **Language**: en", result)
        self.assertIn("- **Auto-generated**: False", result)
        self.assertIn("- **Duration**: 10:30", result)
        self.assertIn("This is a test transcript.", result)

        # Should use cleaned_text over formatted_text
        self.assertNotIn("Raw transcript text", result)

    def test_format_transcript_content_fallback_to_formatted_text(self):
        """Test transcript content formatting falls back to formatted_text."""
        transcript_data = {
            "video_id": "ABC123",
            "formatted_text": "Raw transcript text",
        }

        result = self.formatter.format_transcript_content(transcript_data)

        self.assertIn("Raw transcript text", result)

    def test_format_transcript_content_with_summary(self):
        """Test transcript content formatting with summary."""
        transcript_data = {
            "video_id": "ABC123",
            "cleaned_text": "This is a test transcript.",
        }
        summary = "This video discusses important topics."

        result = self.formatter.format_transcript_content(transcript_data, summary)

        self.assertIn("## Summary", result)
        self.assertIn("This video discusses important topics.", result)

    def test_format_transcript_content_defaults(self):
        """Test transcript content formatting uses appropriate defaults."""
        transcript_data = {"video_id": "ABC123"}

        result = self.formatter.format_transcript_content(transcript_data)

        # Check defaults
        self.assertIn("- **Language**: unknown", result)
        self.assertIn("- **Auto-generated**: False", result)
        self.assertIn("- **Duration**: unknown", result)

        # Should handle empty transcript gracefully
        self.assertIn("## Transcript", result)

    def test_format_transcript_content_special_characters(self):
        """Test transcript content formatting handles special characters."""
        transcript_data = {
            "video_id": "ABC123",
            "title": "Test & <Special> [Characters]",
            "cleaned_text": "Transcript with **markdown** and *emphasis*.",
        }

        result = self.formatter.format_transcript_content(transcript_data)

        # Title should be preserved as-is in markdown
        self.assertIn("# Test & <Special> [Characters]", result)
        self.assertIn("Transcript with **markdown** and *emphasis*.", result)


if __name__ == "__main__":
    unittest.main()
