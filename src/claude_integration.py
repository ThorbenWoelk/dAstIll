"""Claude Code integration for transcript processing."""

import shutil
import subprocess
from pathlib import Path
from typing import Any


class ClaudeCodeIntegration:
    """Integration with Claude Code for automated transcript processing."""

    def __init__(self):
        """Initialize Claude Code integration."""
        self.claude_path = self._find_claude_executable()

    def _find_claude_executable(self) -> str | None:
        """Find Claude Code CLI executable."""
        # Try common locations for Claude Code CLI
        possible_paths = [
            "claude",  # In PATH
            "claude-code",  # Alternative name
            "/usr/local/bin/claude",
            "/opt/homebrew/bin/claude",
        ]

        for path in possible_paths:
            if shutil.which(path):
                return path

        return None

    def is_available(self) -> bool:
        """Check if Claude Code CLI is available."""
        return self.claude_path is not None

    def check_authentication(self) -> tuple[bool, str]:
        """Check if Claude Code is authenticated."""
        if not self.is_available():
            return False, "Claude Code CLI not found"

        try:
            result = subprocess.run(
                [self.claude_path, "auth", "status"],
                capture_output=True,
                text=True,
                timeout=10,
            )

            if result.returncode == 0:
                return True, "Authenticated"
            else:
                return False, result.stderr.strip() or "Authentication failed"

        except subprocess.TimeoutExpired:
            return False, "Authentication check timed out"
        except Exception as e:
            return False, f"Error checking authentication: {str(e)}"

    def get_status(self) -> dict[str, Any]:
        """Get comprehensive status of Claude Code integration."""
        status = {
            "available": self.is_available(),
            "claude_path": self.claude_path,
            "authenticated": False,
            "auth_message": "",
        }

        if status["available"]:
            is_auth, auth_msg = self.check_authentication()
            status["authenticated"] = is_auth
            status["auth_message"] = auth_msg

        return status

    def process_transcript_with_agent(self, transcript_path: Path) -> tuple[bool, str]:
        """Process a transcript using the transcript-education-curator agent."""
        if not self.is_available():
            return False, "Claude Code CLI not available"

        is_auth, auth_msg = self.check_authentication()
        if not is_auth:
            return False, f"Authentication failed: {auth_msg}"

        try:
            # Use the transcript-education-curator agent to process the transcript
            cmd = [
                self.claude_path,
                "--agent",
                "transcript-education-curator",
                str(transcript_path),
            ]

            result = subprocess.run(
                cmd,
                capture_output=True,
                text=True,
                timeout=300,  # 5 minute timeout
                cwd=transcript_path.parent,
            )

            if result.returncode == 0:
                return True, "Successfully processed transcript with AI agent"
            else:
                error_msg = result.stderr.strip() or result.stdout.strip()
                return False, f"Agent processing failed: {error_msg}"

        except subprocess.TimeoutExpired:
            return False, "Agent processing timed out (5 minutes)"
        except Exception as e:
            return False, f"Error running agent: {str(e)}"

    def process_downloaded_transcripts(self, downloaded_dir: Path) -> dict[str, Any]:
        """Process all transcripts in the downloaded directory using AI agent."""
        results = {"total": 0, "processed": 0, "failed": 0, "errors": []}

        if not downloaded_dir.exists():
            results["errors"].append("Downloaded directory does not exist")
            return results

        # Find all transcript files in downloaded directory
        transcript_files = list(downloaded_dir.glob("*.md"))
        results["total"] = len(transcript_files)

        if results["total"] == 0:
            return results

        print(f"Processing {results['total']} transcripts with AI agent...")

        for transcript_file in transcript_files:
            print(f"  Processing: {transcript_file.name}")

            success, message = self.process_transcript_with_agent(transcript_file)

            if success:
                results["processed"] += 1
                print(f"    ✅ {message}")
            else:
                results["failed"] += 1
                results["errors"].append(f"{transcript_file.name}: {message}")
                print(f"    ❌ {message}")

        return results

    def process_transcript(self, transcript_path: Path) -> tuple[bool, str, str]:
        """Process a single transcript and return success status, message, and processed content."""
        if not self.is_available():
            return False, "Claude Code CLI not available", ""

        is_auth, auth_msg = self.check_authentication()
        if not is_auth:
            return False, f"Authentication failed: {auth_msg}", ""

        if not transcript_path.exists():
            return False, f"Transcript file not found: {transcript_path}", ""

        try:
            # Read original content
            original_content = transcript_path.read_text(encoding="utf-8")

            # Use the transcript-education-curator agent to process the transcript
            cmd = [
                self.claude_path,
                "--agent",
                "transcript-education-curator",
                str(transcript_path),
            ]

            result = subprocess.run(
                cmd,
                capture_output=True,
                text=True,
                timeout=300,  # 5 minute timeout
                cwd=transcript_path.parent,
            )

            if result.returncode == 0:
                # Read the processed content (assuming the agent modifies the file in place)
                processed_content = transcript_path.read_text(encoding="utf-8")

                # If content changed, return the new content
                if processed_content != original_content:
                    return (
                        True,
                        "Successfully processed transcript with AI agent",
                        processed_content,
                    )
                else:
                    # If agent didn't modify the file, use stdout as processed content
                    processed_content = result.stdout.strip()
                    if processed_content:
                        return (
                            True,
                            "Successfully processed transcript with AI agent",
                            processed_content,
                        )
                    else:
                        return (
                            True,
                            "AI agent completed but no changes made",
                            original_content,
                        )
            else:
                error_msg = result.stderr.strip() or result.stdout.strip()
                return False, f"Agent processing failed: {error_msg}", ""

        except subprocess.TimeoutExpired:
            return False, "Agent processing timed out (5 minutes)", ""
        except Exception as e:
            return False, f"Error running agent: {str(e)}", ""

    def update_transcript_file(self, transcript_path: Path, content: str) -> bool:
        """Update a transcript file with processed content."""
        try:
            # Create backup
            backup_path = transcript_path.with_suffix(
                transcript_path.suffix + ".backup"
            )
            transcript_path.rename(backup_path)

            # Write new content
            transcript_path.write_text(content, encoding="utf-8")

            # Remove backup on success
            backup_path.unlink()

            return True

        except Exception as e:
            # Restore backup if something went wrong
            backup_path = transcript_path.with_suffix(
                transcript_path.suffix + ".backup"
            )
            if backup_path.exists():
                backup_path.rename(transcript_path)

            print(f"Error updating transcript file: {str(e)}")
            return False
