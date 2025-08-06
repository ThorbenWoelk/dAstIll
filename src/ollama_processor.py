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

    def _extract_video_metadata(self, content: str) -> dict[str, Any]:
        """Extract video metadata from transcript content.

        Args:
            content: Raw transcript content

        Returns:
            Dictionary containing extracted metadata
        """
        metadata = {}

        # Extract video ID from content
        video_id_match = re.search(r"Video ID[:\s]*([a-zA-Z0-9_-]+)", content)
        if video_id_match:
            metadata["video_id"] = video_id_match.group(1)
            metadata["url"] = f"https://www.youtube.com/watch?v={metadata['video_id']}"

        # Extract title
        title_match = re.search(r"Title[:\s]*(.+)", content)
        if title_match:
            metadata["title"] = title_match.group(1).strip()

        # Extract channel
        channel_match = re.search(r"Channel[:\s]*(.+)", content)
        if channel_match:
            metadata["channel"] = channel_match.group(1).strip()

        # Extract language
        language_match = re.search(r"Language[:\s]*(.+)", content)
        if language_match:
            metadata["language"] = language_match.group(1).strip()

        return metadata

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
        prompt = f"""Please process this YouTube transcript into an educational summary following the exact structure specified in your system prompt.

TRANSCRIPT TO PROCESS:
{transcript_content}

REQUIREMENTS:
1. Create a clear, educational title based on the content
2. Write a 50-word maximum TL;DR
3. Include complete video metadata
4. Write a comprehensive 500-word maximum summary
5. Transform the transcript into a structured learning guide with proper formatting
6. PRESERVE the original transcript at the bottom exactly as provided

Focus on making the content educational and well-structured while maintaining all original information."""

        return prompt

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
            # Read the original file
            with open(file_path, encoding="utf-8") as f:
                original_content = f.read()

            # Check if already processed (contains "## Deep Dive" section)
            if "## Deep Dive: Structured Learning Guide" in original_content:
                return True, f"File {file_path.name} already processed, skipping"

            # Extract metadata
            metadata = self._extract_video_metadata(original_content)

            # Generate prompt
            prompt = self._generate_prompt(original_content, metadata)

            # Process with Ollama
            print(f"Processing {file_path.name} with {self.model_name}...")
            processed_content = self._call_ollama_api(prompt)

            if not processed_content:
                return (
                    False,
                    f"Failed to process {file_path.name} - no response from Ollama",
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

            # Create backup of original
            backup_path = file_path.with_suffix(".md.backup")
            file_path.replace(backup_path)

            # Write processed content
            with open(file_path, "w", encoding="utf-8") as f:
                f.write(processed_content)

            # Remove backup if successful
            backup_path.unlink()

            return True, f"Successfully processed {file_path.name}"

        except Exception as e:
            return False, f"Error processing {file_path.name}: {str(e)}"

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
    """Test the Ollama processor."""
    processor = OllamaTranscriptProcessor()

    # Check availability
    available, status = processor.check_ollama_availability()
    print(f"Ollama status: {status}")

    if not available:
        print("Ollama not available, exiting")
        return

    # Test with a sample directory (adjust path as needed)
    from pathlib import Path

    test_dir = (
        Path.home() / "Documents/totos-vault/AI Memory/youtube library/downloaded"
    )

    if test_dir.exists():
        success, total, failed = processor.process_directory(test_dir)
        print(f"\nProcessing complete: {success}/{total} successful")
        if failed:
            print(f"Failed files: {', '.join(failed)}")


if __name__ == "__main__":
    main()
