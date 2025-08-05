"""Tests for Claude Code integration."""

from pathlib import Path
from unittest.mock import MagicMock, patch

from src.claude_integration import ClaudeCodeIntegration


class TestClaudeCodeIntegration:
    """Test Claude Code integration functionality."""

    def test_init_finds_claude_executable(self):
        """Test that initialization finds Claude executable."""
        integration = ClaudeCodeIntegration()
        # If Claude Code is actually installed, this should work
        assert integration.is_available()
        assert integration.claude_path is not None

    def test_init_no_claude_executable(self):
        """Test initialization when Claude is not available."""
        with patch("src.claude_integration.shutil.which", return_value=None):
            integration = ClaudeCodeIntegration()
            assert integration.claude_path is None
            assert not integration.is_available()

    def test_check_authentication_success(self):
        """Test successful authentication check."""
        with (
            patch(
                "src.claude_integration.shutil.which",
                return_value="/usr/local/bin/claude",
            ),
            patch("subprocess.run") as mock_run,
        ):
            mock_run.return_value.returncode = 0
            integration = ClaudeCodeIntegration()

            is_auth, message = integration.check_authentication()
            assert is_auth
            assert message == "Authenticated"

    def test_check_authentication_failure(self):
        """Test failed authentication check."""
        with (
            patch(
                "src.claude_integration.shutil.which",
                return_value="/usr/local/bin/claude",
            ),
            patch("subprocess.run") as mock_run,
        ):
            mock_run.return_value.returncode = 1
            mock_run.return_value.stderr = "Not authenticated"
            integration = ClaudeCodeIntegration()

            is_auth, message = integration.check_authentication()
            assert not is_auth
            assert "Not authenticated" in message

    def test_check_authentication_no_claude(self):
        """Test authentication check when Claude is not available."""
        with patch("src.claude_integration.shutil.which", return_value=None):
            integration = ClaudeCodeIntegration()

            is_auth, message = integration.check_authentication()
            assert not is_auth
            assert "not found" in message

    def test_get_status_complete(self):
        """Test getting complete status information."""
        integration = ClaudeCodeIntegration()
        status = integration.get_status()

        # Should always have these keys
        assert "available" in status
        assert "claude_path" in status
        assert "authenticated" in status
        assert "auth_message" in status

        # If Claude is available, path should be set
        if status["available"]:
            assert status["claude_path"] is not None

    def test_process_transcript_success(self):
        """Test successful transcript processing."""
        with (
            patch(
                "src.claude_integration.shutil.which",
                return_value="/usr/local/bin/claude",
            ),
            patch("subprocess.run") as mock_run,
            patch("pathlib.Path.exists", return_value=True),
            patch("pathlib.Path.read_text") as mock_read,
        ):
            # Mock authentication success
            mock_run.side_effect = [
                MagicMock(returncode=0),  # auth check
                MagicMock(returncode=0, stdout="processed content"),  # agent processing
            ]

            # Mock file content
            mock_read.side_effect = ["original content", "processed content"]

            integration = ClaudeCodeIntegration()
            test_path = Path("/test/transcript.md")

            success, message, content = integration.process_transcript(test_path)
            assert success
            assert "Successfully processed" in message
            assert content == "processed content"

    def test_process_transcript_no_claude(self):
        """Test transcript processing when Claude is not available."""
        with patch("src.claude_integration.shutil.which", return_value=None):
            integration = ClaudeCodeIntegration()
            test_path = Path("/test/transcript.md")

            success, message, content = integration.process_transcript(test_path)
            assert not success
            assert "not available" in message
            assert content == ""

    def test_update_transcript_file_success(self):
        """Test successful transcript file update."""
        with (
            patch("pathlib.Path.rename") as mock_rename,
            patch("pathlib.Path.write_text") as mock_write,
            patch("pathlib.Path.unlink") as mock_unlink,
        ):
            integration = ClaudeCodeIntegration()
            test_path = Path("/test/transcript.md")

            success = integration.update_transcript_file(test_path, "new content")
            assert success
            mock_rename.assert_called_once()
            mock_write.assert_called_once_with("new content", encoding="utf-8")
            mock_unlink.assert_called_once()

    def test_update_transcript_file_failure_with_restore(self):
        """Test transcript file update failure with backup restore."""
        with (
            patch("pathlib.Path.rename") as mock_rename,
            patch("pathlib.Path.write_text", side_effect=Exception("Write failed")),
            patch("pathlib.Path.exists", return_value=True),
            patch("builtins.print"),
        ):
            integration = ClaudeCodeIntegration()
            test_path = Path("/test/transcript.md")

            success = integration.update_transcript_file(test_path, "new content")
            assert not success
            # Should have called rename twice: backup and restore
            assert mock_rename.call_count == 2
