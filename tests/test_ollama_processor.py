"""Tests for Ollama transcript processor functionality."""

import tempfile
import unittest
from pathlib import Path
from unittest.mock import Mock, patch

from src.ollama_processor import OllamaTranscriptProcessor


class TestOllamaProcessor(unittest.TestCase):
    """Test cases for OllamaTranscriptProcessor."""

    def setUp(self):
        """Set up test fixtures."""
        self.temp_dir = Path(tempfile.mkdtemp())
        self.processor = OllamaTranscriptProcessor(
            model_name="test-model", ollama_host="http://localhost:11434"
        )

    def tearDown(self):
        """Clean up test fixtures."""
        # Clean up temp directory
        import shutil

        shutil.rmtree(self.temp_dir)

    def test_initialization(self):
        """Test processor initialization."""
        processor = OllamaTranscriptProcessor(
            model_name="qwen3:8b", ollama_host="http://test:11434"
        )

        self.assertEqual(processor.model_name, "qwen3:8b")
        self.assertEqual(processor.ollama_host, "http://test:11434")
        self.assertIsNotNone(processor.template_content)

    @patch("requests.get")
    def test_check_availability_success(self, mock_get):
        """Test Ollama availability check when successful."""
        # Mock successful response
        mock_response = Mock()
        mock_response.status_code = 200
        mock_response.json.return_value = {
            "models": [
                {"name": "test-model", "size": "1234567890"},
                {"name": "other-model", "size": "9876543210"},
            ]
        }
        mock_get.return_value = mock_response

        available, message = self.processor.check_ollama_availability()

        self.assertTrue(available)
        self.assertIn("test-model", message)
        mock_get.assert_called_once_with("http://localhost:11434/api/tags", timeout=5)

    @patch("requests.get")
    def test_check_availability_server_not_running(self, mock_get):
        """Test Ollama availability check when server is not running."""
        # Mock failed response
        mock_response = Mock()
        mock_response.status_code = 500
        mock_get.return_value = mock_response

        available, message = self.processor.check_ollama_availability()

        self.assertFalse(available)
        self.assertIn("not accessible", message)

    @patch("requests.get")
    def test_check_availability_model_not_found(self, mock_get):
        """Test Ollama availability check when model is not available."""
        # Mock response without the required model
        mock_response = Mock()
        mock_response.status_code = 200
        mock_response.json.return_value = {
            "models": [{"name": "other-model", "size": "9876543210"}]
        }
        mock_get.return_value = mock_response

        available, message = self.processor.check_ollama_availability()

        self.assertFalse(available)
        self.assertIn("not found", message)
        self.assertIn("other-model", message)

    @patch("requests.get")
    def test_check_availability_connection_error(self, mock_get):
        """Test Ollama availability check with connection error."""
        import requests

        mock_get.side_effect = requests.exceptions.ConnectionError("Connection failed")

        available, message = self.processor.check_ollama_availability()

        self.assertFalse(available)
        self.assertIn("Cannot connect", message)

    def test_extract_video_metadata(self):
        """Test video metadata extraction."""
        content = """
        Video ID: ABC123TEST
        Title: Test Video Title
        Channel: Test Channel Name
        Language: en
        Some other content here
        """

        metadata = self.processor._extract_video_metadata(content)

        self.assertEqual(metadata["video_id"], "ABC123TEST")
        self.assertEqual(metadata["title"], "Test Video Title")
        self.assertEqual(metadata["channel"], "Test Channel Name")
        self.assertEqual(metadata["language"], "en")
        self.assertEqual(metadata["url"], "https://www.youtube.com/watch?v=ABC123TEST")

    def test_extract_video_metadata_missing_fields(self):
        """Test metadata extraction with missing fields."""
        content = "Just some content without metadata"

        metadata = self.processor._extract_video_metadata(content)

        self.assertEqual(len(metadata), 0)

    def test_generate_prompt(self):
        """Test prompt generation."""
        transcript_content = "This is a test transcript about machine learning."
        metadata = {"video_id": "ABC123", "title": "ML Basics", "channel": "AI Channel"}

        prompt = self.processor._generate_prompt(transcript_content, metadata)

        self.assertIn("educational summary", prompt)
        self.assertIn(transcript_content, prompt)
        self.assertIn("TL;DR", prompt)
        self.assertIn("PRESERVE", prompt)

    @patch("requests.post")
    def test_call_ollama_api_success(self, mock_post):
        """Test successful Ollama API call."""
        # Mock successful API response
        mock_response = Mock()
        mock_response.status_code = 200
        mock_response.json.return_value = {
            "response": "Generated educational content here"
        }
        mock_post.return_value = mock_response

        prompt = "Test prompt"
        result = self.processor._call_ollama_api(prompt)

        self.assertEqual(result, "Generated educational content here")
        mock_post.assert_called_once()

        # Verify the payload structure
        call_args = mock_post.call_args
        payload = call_args[1]["json"]
        self.assertEqual(payload["model"], "test-model")
        self.assertEqual(payload["prompt"], prompt)
        self.assertFalse(payload["stream"])

    @patch("requests.post")
    def test_call_ollama_api_error(self, mock_post):
        """Test Ollama API call with error response."""
        # Mock error response
        mock_response = Mock()
        mock_response.status_code = 500
        mock_response.text = "Internal server error"
        mock_post.return_value = mock_response

        prompt = "Test prompt"
        result = self.processor._call_ollama_api(prompt, max_retries=1)

        self.assertIsNone(result)

    @patch("requests.post")
    def test_call_ollama_api_timeout(self, mock_post):
        """Test Ollama API call with timeout."""
        import requests

        mock_post.side_effect = requests.exceptions.Timeout("Request timed out")

        prompt = "Test prompt"
        result = self.processor._call_ollama_api(prompt, max_retries=1)

        self.assertIsNone(result)

    def test_process_transcript_file_already_processed(self):
        """Test processing a file that's already been processed."""
        # Create test file with processed content
        test_file = self.temp_dir / "test_processed.md"
        with open(test_file, "w", encoding="utf-8") as f:
            f.write("""
            # Test Title
            ## Deep Dive: Structured Learning Guide
            Some content here
            """)

        success, message = self.processor.process_transcript_file(test_file)

        self.assertTrue(success)
        self.assertIn("already processed", message)

    @patch("src.ollama_processor.OllamaTranscriptProcessor._call_ollama_api")
    def test_process_transcript_file_success(self, mock_api_call):
        """Test successful transcript file processing."""
        # Create test file
        test_file = self.temp_dir / "test_transcript.md"
        original_content = """
        Video ID: ABC123TEST
        Title: Test Video
        Channel: Test Channel

        This is the original transcript content.
        """

        with open(test_file, "w", encoding="utf-8") as f:
            f.write(original_content)

        # Mock API response
        processed_content = (
            """
        # Test Video Enhanced

        ## TL;DR
        A test video about important concepts.

        ## Video Information
        - **Video ID**: ABC123TEST
        - **Title**: Test Video
        - **Channel**: Test Channel

        ## Summary
        This is a comprehensive summary of the test video content.

        ## Deep Dive: Structured Learning Guide
        ### Key Concepts
        Important educational content here.

        ## Original Transcript
        """
            + original_content
        )

        mock_api_call.return_value = processed_content

        success, message = self.processor.process_transcript_file(test_file)

        self.assertTrue(success)
        self.assertIn("Successfully processed", message)

        # Verify file was updated
        with open(test_file, encoding="utf-8") as f:
            updated_content = f.read()

        self.assertIn("## Deep Dive: Structured Learning Guide", updated_content)
        self.assertIn("TL;DR", updated_content)

    @patch("src.ollama_processor.OllamaTranscriptProcessor._call_ollama_api")
    def test_process_transcript_file_api_failure(self, mock_api_call):
        """Test transcript processing with API failure."""
        # Create test file
        test_file = self.temp_dir / "test_transcript.md"
        with open(test_file, "w", encoding="utf-8") as f:
            f.write("Test content")

        # Mock API failure
        mock_api_call.return_value = None

        success, message = self.processor.process_transcript_file(test_file)

        self.assertFalse(success)
        self.assertIn("no response from Ollama", message)

    @patch("src.ollama_processor.OllamaTranscriptProcessor._call_ollama_api")
    def test_process_transcript_file_missing_sections(self, mock_api_call):
        """Test processing with incomplete API response."""
        # Create test file
        test_file = self.temp_dir / "test_transcript.md"
        with open(test_file, "w", encoding="utf-8") as f:
            f.write("Test content")

        # Mock incomplete API response (missing required sections)
        mock_api_call.return_value = "# Title\n## Summary\nSome content"

        success, message = self.processor.process_transcript_file(test_file)

        self.assertFalse(success)
        self.assertIn("missing sections", message)

    @patch("src.ollama_processor.OllamaTranscriptProcessor.process_transcript_file")
    def test_process_directory_success(self, mock_process_file):
        """Test processing directory with multiple files."""
        # Create test files
        file1 = self.temp_dir / "test1.md"
        file2 = self.temp_dir / "test2.md"
        file3 = self.temp_dir / "test3.txt"  # Non-markdown file

        for file_path in [file1, file2]:
            with open(file_path, "w", encoding="utf-8") as f:
                f.write("Test content")

        with open(file3, "w", encoding="utf-8") as f:
            f.write("Non-markdown content")

        # Mock successful processing
        mock_process_file.side_effect = [(True, "Success 1"), (True, "Success 2")]

        success_count, total_count, failed_files = self.processor.process_directory(
            self.temp_dir
        )

        self.assertEqual(success_count, 2)
        self.assertEqual(total_count, 2)  # Only .md files counted
        self.assertEqual(len(failed_files), 0)
        self.assertEqual(mock_process_file.call_count, 2)

    def test_process_directory_not_found(self):
        """Test processing non-existent directory."""
        non_existent_dir = self.temp_dir / "does_not_exist"

        success_count, total_count, failed_files = self.processor.process_directory(
            non_existent_dir
        )

        self.assertEqual(success_count, 0)
        self.assertEqual(total_count, 0)
        self.assertEqual(len(failed_files), 1)
        self.assertIn("Directory not found", failed_files[0])

    def test_process_directory_no_files(self):
        """Test processing directory with no markdown files."""
        # Create non-markdown file
        test_file = self.temp_dir / "test.txt"
        with open(test_file, "w") as f:
            f.write("Not markdown")

        success_count, total_count, failed_files = self.processor.process_directory(
            self.temp_dir
        )

        self.assertEqual(success_count, 0)
        self.assertEqual(total_count, 0)
        self.assertEqual(len(failed_files), 1)
        self.assertIn("No transcript files found", failed_files[0])

    @patch("src.ollama_processor.OllamaTranscriptProcessor.process_transcript_file")
    def test_process_directory_mixed_results(self, mock_process_file):
        """Test processing directory with mixed success/failure."""
        # Create test files
        file1 = self.temp_dir / "success.md"
        file2 = self.temp_dir / "failure.md"

        for file_path in [file1, file2]:
            with open(file_path, "w", encoding="utf-8") as f:
                f.write("Test content")

        # Mock mixed results based on filename
        def mock_process_side_effect(file_path):
            if "failure" in file_path.name:
                return (False, "Failed to process")
            else:
                return (True, "Success")

        mock_process_file.side_effect = mock_process_side_effect

        success_count, total_count, failed_files = self.processor.process_directory(
            self.temp_dir
        )

        self.assertEqual(success_count, 1)
        self.assertEqual(total_count, 2)
        self.assertEqual(len(failed_files), 1)
        self.assertTrue(
            any(failed_file == "failure.md" for failed_file in failed_files)
        )

    def test_default_template_loading(self):
        """Test loading default template when file not found."""
        processor = OllamaTranscriptProcessor(template_path="/nonexistent/path")

        self.assertIn("Transform raw YouTube transcripts", processor.template_content)
        self.assertIn("Required Structure", processor.template_content)

    def test_system_prompt_generation(self):
        """Test system prompt contains required elements."""
        system_prompt = self.processor._get_system_prompt()

        self.assertIn("Professor Synthesis", system_prompt)
        self.assertIn("CRITICAL REQUIREMENTS", system_prompt)
        self.assertIn("PRESERVE THE ORIGINAL TRANSCRIPT", system_prompt)
        self.assertIn("REQUIRED OUTPUT STRUCTURE", system_prompt)


if __name__ == "__main__":
    unittest.main()
