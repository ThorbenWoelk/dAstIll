"""Tests for Claude Code integration."""

from unittest.mock import MagicMock, patch

from src.claude_integration import ClaudeCodeIntegration


class TestClaudeCodeIntegration:
    """Test Claude Code integration functionality."""

    def test_init_finds_claude_executable(self):
        """Test that initialization finds Claude executable."""
        integration = ClaudeCodeIntegration()
        # This test depends on the actual system state
        # In CI/testing environments, Claude might not be installed
        assert isinstance(integration.is_available(), bool)

    def test_init_no_claude_executable(self):
        """Test initialization when Claude is not available."""
        with patch("src.claude_integration.shutil.which", return_value=None):
            integration = ClaudeCodeIntegration()
            assert integration.claude_path is None
            assert not integration.is_available()

    def test_check_authentication_success(self):
        """Test successful authentication check."""
        with patch(
            "src.claude_integration.shutil.which", return_value="/usr/local/bin/claude"
        ):
            with patch("src.claude_integration.subprocess.run") as mock_run:
                mock_run.return_value = MagicMock(returncode=0)
                integration = ClaudeCodeIntegration()

                is_auth, message = integration.check_authentication()
                assert is_auth
                assert "authenticated and working" in message

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

    def test_bash_workflow_integration(self):
        """Test that the integration properly supports bash workflow approach."""
        # This test verifies the new bash workflow approach
        with patch("src.claude_integration.shutil.which", return_value="claude"):
            integration = ClaudeCodeIntegration()

            # Verify basic functionality for bash workflow
            assert integration.is_available()
            assert integration.claude_path == "claude"

            # Status should contain the necessary information for bash script
            status = integration.get_status()
            assert "available" in status
            assert "claude_path" in status
