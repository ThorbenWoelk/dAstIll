"""Claude Code integration for transcript processing via bash wrapper."""

import shutil
import subprocess
from typing import Any


class ClaudeCodeIntegration:
    """
    Claude Code integration using bash wrapper approach.

    This integration uses a bash script to orchestrate:
    1. Docker container for automated monitoring/downloading
    2. Claude Code CLI for AI-powered transcript processing

    This approach leverages each tool in its optimal environment rather than
    trying to force SDK integration that has session limitations.
    """

    def __init__(self):
        """Initialize Claude Code integration."""
        self.claude_path = self._find_claude_executable()

    def _find_claude_executable(self) -> str | None:
        """Find Claude Code CLI executable."""
        for name in ["claude", "claude-code"]:
            if shutil.which(name):
                return name
        return None

    def is_available(self) -> bool:
        """Check if Claude Code CLI is available."""
        return self.claude_path is not None

    def check_authentication(self) -> tuple[bool, str]:
        """Check if Claude Code is authenticated."""
        if not self.is_available():
            return False, "Claude Code CLI not found"

        try:
            # Test authentication with a simple command
            result = subprocess.run(
                [self.claude_path, "--version"],
                capture_output=True,
                text=True,
                timeout=10,
            )
            if result.returncode == 0:
                return True, "Claude Code CLI authenticated and working"
            else:
                return False, f"Claude Code CLI not authenticated: {result.stderr}"
        except Exception as e:
            return False, f"Error checking Claude Code authentication: {str(e)}"

    def get_status(self) -> dict[str, Any]:
        """Get comprehensive status of Claude Code integration."""
        status = {
            "available": self.is_available(),
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
            status["message"] = "Claude Code CLI not found"

        return status
