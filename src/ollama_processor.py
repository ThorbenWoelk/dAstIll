"""Ollama-based local AI transcript processor for educational content curation.

This module provides a cost-effective alternative to Claude Code by using local Ollama models
to process YouTube transcripts into structured educational summaries.

Implements the Professor Synthesis persona for educational content transformation,
following the same template and quality standards as the Claude Code integration.
"""

import re
import time
from pathlib import Path
from typing import Any

import requests


class OllamaTranscriptProcessor:
    """Local AI transcript processor using Ollama models."""

    def __init__(
        self,
        model_name: str = "qwen3:8b",
        ollama_host: str = "http://localhost:11434",
        template_path: str | None = None,
    ):
        """Initialize the Ollama processor.

        Args:
            model_name: Name of the Ollama model to use
            ollama_host: Ollama server host URL
            template_path: Path to the transcript template for reference
        """
        self.model_name = model_name
        self.ollama_host = ollama_host
        self.template_path = template_path
        self.template_content = self._load_template()

    def _load_template(self) -> str:
        """Load the transcript template for processing standards."""
        if not self.template_path or not Path(self.template_path).exists():
            # Return basic template if file not found
            return self._get_default_template()

        try:
            with open(self.template_path, encoding="utf-8") as f:
                return f.read()
        except Exception:
            return self._get_default_template()

    def _get_default_template(self) -> str:
        """Get the default transcript processing template."""
        return """
# Transcript Processing Template

Transform raw YouTube transcripts into educational resources with this structure:

## Required Structure:
1. TL;DR (max 50 words)
2. Video Information (metadata)
3. Summary (max 500 words)
4. Deep Dive: Structured Learning Guide
5. Original Transcript (preserved)

## Quality Standards:
- Transform choppy speech into flowing prose
- Remove filler words and repetition
- Maintain all technical accuracy
- Use proper markdown formatting
- Create logical flow and structure
"""

    def _get_system_prompt(self) -> str:
        """Get the system prompt for Professor Synthesis persona."""
        return f"""You are Professor Synthesis, a distinguished academic with decades of experience in educational content curation and pedagogical design. You possess exceptional text editing skills and a deep understanding of how knowledge should be structured for optimal learning and retention.

ROLE IN dAstIll ECOSYSTEM:
You serve as the "professor" for the dAstIll application - a Python CLI tool for YouTube transcript management. While dAstIll handles the technical aspects (downloading, organizing files by channel), you provide the educational analysis and content transformation.

TEMPLATE REFERENCE:
{self.template_content}

CRITICAL REQUIREMENTS:
1. ALWAYS PRESERVE THE ORIGINAL TRANSCRIPT - Never overwrite or delete existing content
2. ENHANCE, DON'T REPLACE - Add structured content while keeping original intact
3. READABLE TEXT TRANSFORMATION - Transform transcript into well-written, flowing text
4. MANDATORY STRUCTURE - Every processed transcript must include the 7-section structure

REQUIRED OUTPUT STRUCTURE:
```markdown
# [Short, Clear Topic Title]

## TL;DR
[Maximum 50 words summarizing the core message/value]

## Video Information
- **Video ID**: [YouTube video ID]
- **URL**: https://www.youtube.com/watch?v=[VIDEO_ID]
- **Title**: [Video title]
- **Channel**: [Channel name]
- **Language**: [Language]
- **Processed**: [Current date]

## Summary
[Maximum 500 words providing comprehensive overview]

## Deep Dive: Structured Learning Guide
[Transform original transcript into well-organized guide with:
- Clear hierarchical structure using headers
- Grammatically correct, flowing prose
- Key concepts in **bold**
- Important insights in *italics*
- Code blocks when applicable
- Logical flow following video progression]

---
## Original Transcript
[PRESERVE THE ENTIRE ORIGINAL TRANSCRIPT EXACTLY AS IT WAS]
```

EDITING EXCELLENCE STANDARDS:
- Transform choppy transcript speech into smooth, readable prose
- Maintain all technical accuracy and details
- Remove "um," "uh," repetitions, and speech artifacts
- Ensure proper grammar and sentence structure
- Create logical paragraph breaks and transitions
- Use markdown formatting to enhance readability
- Preserve speaker's intent while improving clarity

Your mission is to transform raw YouTube transcripts into polished educational resources while preserving the complete original content for reference."""

    def check_ollama_availability(self) -> tuple[bool, str]:
        """Check if Ollama is available and the model is ready.

        Returns:
            Tuple of (is_available, status_message)
        """
        try:
            # Check if Ollama server is running
            response = requests.get(f"{self.ollama_host}/api/tags", timeout=5)
            if response.status_code != 200:
                return False, f"Ollama server not accessible at {self.ollama_host}"

            # Check if model is available
            models = response.json().get("models", [])
            model_names = [model["name"] for model in models]

            if self.model_name not in model_names:
                return (
                    False,
                    f"Model {self.model_name} not found. Available: {', '.join(model_names)}",
                )

            return True, f"Ollama ready with model {self.model_name}"

        except requests.exceptions.RequestException as e:
            return False, f"Cannot connect to Ollama: {str(e)}"
        except Exception as e:
            return False, f"Error checking Ollama: {str(e)}"

    def get_available_models(self) -> list[dict[str, Any]]:
        """Get list of available Ollama models.

        Returns:
            List of model information dictionaries
        """
        try:
            response = requests.get(f"{self.ollama_host}/api/tags", timeout=5)
            if response.status_code == 200:
                return response.json().get("models", [])
        except requests.exceptions.RequestException:
            pass
        except Exception:
            pass
        return []

    def _extract_video_metadata(self, content: str) -> dict[str, Any]:
        """Extract video metadata from transcript content.

        Args:
            content: Raw transcript content

        Returns:
            Dictionary containing extracted metadata
        """
        metadata = {}

        # Extract video ID from content with validation
        video_id_match = re.search(r"Video ID[:\s]*([a-zA-Z0-9_-]+)", content)
        if video_id_match:
            video_id = video_id_match.group(1)
            # Validate YouTube video ID format (11 characters, alphanumeric + _ -)
            if self._is_valid_video_id(video_id):
                metadata["video_id"] = video_id
                metadata["url"] = f"https://www.youtube.com/watch?v={video_id}"

        # Extract title with sanitization
        title_match = re.search(r"Title[:\s]*(.+)", content)
        if title_match:
            title = title_match.group(1).strip()
            metadata["title"] = self._sanitize_text_field(title, max_length=200)

        # Extract channel with sanitization
        channel_match = re.search(r"Channel[:\s]*(.+)", content)
        if channel_match:
            channel = channel_match.group(1).strip()
            metadata["channel"] = self._sanitize_text_field(channel, max_length=100)

        # Extract language with validation
        language_match = re.search(r"Language[:\s]*(.+)", content)
        if language_match:
            language = language_match.group(1).strip()
            # Validate language format (2-3 letter codes)
            if re.match(r"^[a-z]{2,3}(-[A-Z]{2})?$", language):
                metadata["language"] = language

        return metadata

    def _is_valid_video_id(self, video_id: str) -> bool:
        """Validate YouTube video ID format.

        Args:
            video_id: Video ID to validate

        Returns:
            True if valid YouTube video ID format
        """
        if not video_id or not isinstance(video_id, str):
            return False
        # YouTube video IDs are 11 characters, alphanumeric plus _ and -
        return bool(re.match(r"^[a-zA-Z0-9_-]{11}$", video_id))

    def _sanitize_text_field(self, text: str, max_length: int = 500) -> str:
        """Sanitize text fields to prevent injection attacks.

        Args:
            text: Text to sanitize
            max_length: Maximum allowed length

        Returns:
            Sanitized text
        """
        if not text or not isinstance(text, str):
            return ""

        # Remove potential path traversal sequences
        text = re.sub(r"\.\./", "", text)
        text = re.sub(r"\.\.\\", "", text)

        # Remove control characters and normalize whitespace
        text = re.sub(r"[\x00-\x1f\x7f-\x9f]", "", text)
        text = re.sub(r"\s+", " ", text).strip()

        # Truncate to max length
        return text[:max_length] if len(text) > max_length else text

    def _generate_prompt(
        self, transcript_content: str, metadata: dict[str, Any]
    ) -> str:
        """Generate processing prompt for the transcript.

        Args:
            transcript_content: Raw transcript content
            metadata: Extracted video metadata

        Returns:
            Formatted prompt for the AI model
        """
        # Sanitize transcript content to prevent prompt injection
        sanitized_content = self._sanitize_transcript_content(transcript_content)

        prompt = f"""Please process this YouTube transcript into an educational summary following the exact structure specified in your system prompt.

TRANSCRIPT TO PROCESS:
{sanitized_content}

REQUIREMENTS:
1. Create a clear, educational title based on the content
2. Write a 50-word maximum TL;DR
3. Include complete video metadata
4. Write a comprehensive 500-word maximum summary
5. Transform the transcript into a structured learning guide with proper formatting
6. PRESERVE the original transcript at the bottom exactly as provided

Focus on making the content educational and well-structured while maintaining all original information."""

        return prompt

    def _sanitize_transcript_content(self, content: str) -> str:
        """Sanitize transcript content to prevent prompt injection.

        Args:
            content: Raw transcript content

        Returns:
            Sanitized content safe for AI prompts
        """
        if not content or not isinstance(content, str):
            return ""

        # Remove potential prompt injection patterns
        # Remove system prompt markers and commands
        content = re.sub(
            r"```\s*(system|assistant|user|human)",
            "```text",
            content,
            flags=re.IGNORECASE,
        )
        content = re.sub(r"\bsystem\s*:", "note:", content, flags=re.IGNORECASE)
        content = re.sub(r"\bassistant\s*:", "speaker:", content, flags=re.IGNORECASE)

        # Remove potential instruction injection
        content = re.sub(
            r"\b(ignore|disregard|forget)\s+(all\s+)?(previous\s+)?(instructions?|prompts?)",
            "refer to previous content",
            content,
            flags=re.IGNORECASE,
        )

        # Normalize excessive whitespace and control characters
        content = re.sub(r"[\x00-\x08\x0b\x0c\x0e-\x1f\x7f-\x9f]", "", content)
        content = re.sub(r"\n{3,}", "\n\n", content)
        content = re.sub(r" {3,}", " ", content)

        return content.strip()

    def _call_ollama_api(self, prompt: str, max_retries: int = 3) -> str | None:
        """Call Ollama API to process the transcript.

        Args:
            prompt: The processing prompt
            max_retries: Maximum number of retry attempts

        Returns:
            Generated response or None if failed
        """
        for attempt in range(max_retries):
            try:
                payload = {
                    "model": self.model_name,
                    "prompt": prompt,
                    "system": self._get_system_prompt(),
                    "stream": False,
                    "options": {
                        "temperature": 0.3,  # Lower temperature for more consistent output
                        "top_p": 0.9,
                        "top_k": 40,
                    },
                }

                response = requests.post(
                    f"{self.ollama_host}/api/generate",
                    json=payload,
                    timeout=300,  # 5 minutes timeout for processing
                )

                if response.status_code == 200:
                    result = response.json()
                    return result.get("response", "")
                else:
                    print(f"Ollama API error {response.status_code}: {response.text}")

            except requests.exceptions.Timeout:
                print(f"Timeout on attempt {attempt + 1}, retrying...")
                continue
            except requests.exceptions.RequestException as e:
                print(f"Request error on attempt {attempt + 1}: {str(e)}")
                if attempt < max_retries - 1:
                    time.sleep(2**attempt)  # Exponential backoff
                    continue
                else:
                    break
            except Exception as e:
                print(f"Unexpected error: {str(e)}")
                break

        return None

    def process_transcript_file(self, file_path: Path) -> tuple[bool, str]:
        """Process a single transcript file.

        Args:
            file_path: Path to the transcript file

        Returns:
            Tuple of (success, message)
        """
        try:
            # Validate and secure the file path
            secure_path = self._validate_file_path(file_path)
            if not secure_path:
                return False, f"Invalid or unsafe file path: {file_path}"

            # Read the original file
            with open(secure_path, encoding="utf-8") as f:
                original_content = f.read()

            # Check if already processed (contains "## Deep Dive" section)
            if "## Deep Dive: Structured Learning Guide" in original_content:
                return True, f"File {secure_path.name} already processed, skipping"

            # Extract metadata
            metadata = self._extract_video_metadata(original_content)

            # Generate prompt
            prompt = self._generate_prompt(original_content, metadata)

            # Process with Ollama
            print(f"Processing {secure_path.name} with {self.model_name}...")
            processed_content = self._call_ollama_api(prompt)

            if not processed_content:
                return (
                    False,
                    f"Failed to process {secure_path.name} - no response from Ollama",
                )

            # Validate the output has required sections
            required_sections = [
                "## TL;DR",
                "## Video Information",
                "## Summary",
                "## Deep Dive",
            ]
            missing_sections = [
                section
                for section in required_sections
                if section not in processed_content
            ]

            if missing_sections:
                return (
                    False,
                    f"Processed content missing sections: {', '.join(missing_sections)}",
                )

            # Atomic file write using temporary file
            success = self._write_processed_file_atomic(secure_path, processed_content)
            if not success:
                return False, f"Failed to write processed content to {secure_path.name}"

            return True, f"Successfully processed {secure_path.name}"

        except Exception as e:
            return False, f"Error processing {file_path.name}: {str(e)}"

    def _validate_file_path(self, file_path: Path) -> Path | None:
        """Validate file path to prevent path traversal attacks.

        Args:
            file_path: Path to validate

        Returns:
            Validated path or None if invalid
        """
        try:
            # Resolve to absolute path and check for path traversal
            resolved_path = file_path.resolve()

            # Ensure the file is a markdown file
            if resolved_path.suffix.lower() != ".md":
                return None

            # Ensure file exists and is actually a file
            if not resolved_path.exists() or not resolved_path.is_file():
                return None

            # Basic security check - ensure path doesn't contain suspicious patterns
            path_str = str(resolved_path)
            if ".." in path_str or "~" in path_str:
                return None

            return resolved_path

        except (OSError, RuntimeError):
            return None

    def _write_processed_file_atomic(self, file_path: Path, content: str) -> bool:
        """Atomically write processed content to file.

        Args:
            file_path: Target file path
            content: Content to write

        Returns:
            True if successful, False otherwise
        """
        import shutil
        import tempfile

        try:
            # Create temporary file in the same directory
            temp_dir = file_path.parent
            with tempfile.NamedTemporaryFile(
                mode="w", encoding="utf-8", dir=temp_dir, delete=False, suffix=".tmp"
            ) as temp_file:
                temp_file.write(content)
                temp_path = Path(temp_file.name)

            # Atomic move (rename) to final location
            shutil.move(str(temp_path), str(file_path))
            return True

        except Exception:
            # Clean up temp file if it exists
            try:
                temp_path.unlink()
            except (NameError, OSError):
                pass
            return False

    def process_directory(self, directory_path: Path) -> tuple[int, int, list[str]]:
        """Process all transcript files in a directory.

        Args:
            directory_path: Path to directory containing transcript files

        Returns:
            Tuple of (success_count, total_count, failed_files)
        """
        if not directory_path.exists() or not directory_path.is_dir():
            return 0, 0, [f"Directory not found: {directory_path}"]

        # Find all markdown files
        transcript_files = list(directory_path.glob("*.md"))

        if not transcript_files:
            return 0, 0, ["No transcript files found"]

        success_count = 0
        failed_files = []

        print(f"Found {len(transcript_files)} transcript files to process")

        for file_path in transcript_files:
            success, message = self.process_transcript_file(file_path)
            print(f"  {'✓' if success else '✗'} {message}")

            if success:
                success_count += 1
            else:
                failed_files.append(file_path.name)

        return success_count, len(transcript_files), failed_files


def main():
    """Test the Ollama processor with configurable directory."""
    import sys

    processor = OllamaTranscriptProcessor()

    # Check availability
    available, status = processor.check_ollama_availability()
    print(f"Ollama status: {status}")

    if not available:
        print("Ollama not available, exiting")
        return

    # Use command line argument or current directory
    if len(sys.argv) > 1:
        test_dir = Path(sys.argv[1])
    else:
        test_dir = Path.cwd() / "data" / "downloaded"

    print(f"Testing with directory: {test_dir}")

    if test_dir.exists() and test_dir.is_dir():
        success, total, failed = processor.process_directory(test_dir)
        print(f"\nProcessing complete: {success}/{total} successful")
        if failed:
            print(f"Failed files: {', '.join(failed)}")
    else:
        print(f"Directory does not exist: {test_dir}")
        print("Usage: python ollama_processor.py [directory_path]")


if __name__ == "__main__":
    main()
