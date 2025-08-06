"""Claude Code integration for transcript processing."""

import asyncio
import os
import shutil
from pathlib import Path
from typing import Any

try:
    from claude_code_sdk import AssistantMessage, ClaudeCodeOptions, TextBlock, query

    SDK_AVAILABLE = True
except ImportError:
    SDK_AVAILABLE = False


class ClaudeCodeIntegration:
    """Integration with Claude Code for automated transcript processing."""

    def __init__(self):
        """Initialize Claude Code integration."""
        self.claude_path = self._find_claude_executable()
        self.is_docker = self._is_running_in_docker()
        self.is_claude_code_session = self._is_claude_code_session()
        self.sdk_available = SDK_AVAILABLE

    def _is_running_in_docker(self) -> bool:
        """Check if running inside a Docker container."""
        return Path("/.dockerenv").exists() or Path("/proc/1/cgroup").exists()

    def _is_claude_code_session(self) -> bool:
        """Check if running within a Claude Code session."""
        return (
            os.getenv("CLAUDE_CODE_SESSION") is not None
            or os.getenv("ANTHROPIC_CLI_SESSION") is not None
        )

    def _find_claude_executable(self) -> str | None:
        """Find Claude Code CLI executable."""
        # Try common CLI names
        for name in ["claude", "claude-code"]:
            if shutil.which(name):
                return name
        return None

    def is_available(self) -> bool:
        """Check if Claude Code SDK or CLI is available."""
        return self.sdk_available or self.claude_path is not None

    def check_authentication(self) -> tuple[bool, str]:
        """Check if Claude Code is authenticated."""
        if self.sdk_available:
            # SDK handles authentication internally
            return True, "SDK authenticated"

        if not self.is_available():
            return False, "Claude Code CLI not found"

        # Docker requires special setup for authentication
        if self.is_docker:
            return (
                False,
                "Docker environment detected - AI features require running on host system",
            )

        # For now, assume authentication works on host system
        # TODO: Implement proper authentication check
        return True, "Authenticated (assuming host authentication)"

    def get_status(self) -> dict[str, Any]:
        """Get comprehensive status of Claude Code integration."""
        status = {
            "available": self.is_available(),
            "sdk_available": self.sdk_available,
            "claude_path": self.claude_path,
            "authenticated": False,
            "auth_message": "",
            "message": "",
        }

        if status["available"]:
            is_auth, auth_msg = self.check_authentication()
            status["authenticated"] = is_auth
            status["auth_message"] = auth_msg
            status["message"] = auth_msg
        else:
            status["message"] = "Claude Code SDK/CLI not found"

        return status

    async def _process_with_sdk(
        self, transcript_content: str, transcript_name: str
    ) -> tuple[bool, str, str]:
        """Process transcript using Claude Code SDK."""
        try:
            options = ClaudeCodeOptions(max_turns=1, model="claude-3-5-sonnet-20241022")

            prompt = f"""You are the transcript-education-curator agent. Please process this YouTube transcript and transform it into a well-structured educational summary with key concepts, insights, and actionable takeaways.

Transcript file: {transcript_name}

---BEGIN TRANSCRIPT---
{transcript_content}
---END TRANSCRIPT---

Please provide a comprehensive educational summary that includes:
1. Main topic and key concepts
2. Important insights and lessons
3. Actionable takeaways
4. Technical details if applicable
5. References and resources mentioned"""

            enhanced_content = ""
            async for message in query(prompt=prompt, options=options):
                # Handle AssistantMessage with TextBlock content
                if isinstance(message, AssistantMessage):
                    for block in message.content:
                        if isinstance(block, TextBlock):
                            enhanced_content += block.text
                elif hasattr(message, "text"):
                    enhanced_content = message.text
                elif isinstance(message, str):
                    enhanced_content = message

            if enhanced_content:
                return True, "Successfully processed with SDK", enhanced_content
            else:
                return False, "No response from SDK", ""

        except Exception as e:
            return False, f"SDK processing error: {str(e)}", ""

    def process_transcript(self, transcript_path: Path) -> tuple[bool, str, str]:
        """Process a single transcript and return success status, message, and processed content."""
        if not transcript_path.exists():
            return False, f"Transcript file not found: {transcript_path}", ""

        try:
            # Read original content
            original_content = transcript_path.read_text(encoding="utf-8")

            if self.sdk_available:
                # Use SDK for processing
                success, message, enhanced_content = asyncio.run(
                    self._process_with_sdk(original_content, transcript_path.name)
                )
                return success, message, enhanced_content
            else:
                # Fallback: indicate SDK not available
                return (
                    False,
                    "Claude Code SDK not available - install with 'uv add claude-code-sdk'",
                    "",
                )

        except Exception as e:
            return False, f"Error in AI processing: {str(e)}", ""

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
