"""Test channel ID resolution functionality."""

from unittest.mock import MagicMock, patch

import pytest

from main import validate_handle_format
from src.rss_monitor import RSSChannelMonitor


class TestChannelIDResolution:
    """Test channel ID resolution from handles."""

    @pytest.fixture
    def monitor(self):
        """Create monitor instance."""
        return RSSChannelMonitor()

    def test_resolve_channel_id_success(self, monitor):
        """Test successful channel ID resolution."""
        # Mock response with channel ID in meta tag
        mock_response = MagicMock()
        mock_response.status_code = 200
        mock_response.text = """
        <html>
        <meta itemprop="channelId" content="UCdtcNj0e5bPpJLxLXRx_W_Q">
        </html>
        """

        with patch.object(monitor, "_request_with_backoff", return_value=mock_response):
            channel_id = monitor.resolve_channel_id_from_handle("@testchannel")
            assert channel_id == "UCdtcNj0e5bPpJLxLXRx_W_Q"

    def test_resolve_channel_id_with_external_id(self, monitor):
        """Test channel ID resolution using externalId."""
        mock_response = MagicMock()
        mock_response.status_code = 200
        mock_response.text = """
        <script>
        {"externalId":"UCdtcNj0e5bPpJLxLXRx_W_Q"}
        </script>
        """

        with patch.object(monitor, "_request_with_backoff", return_value=mock_response):
            channel_id = monitor.resolve_channel_id_from_handle("testchannel")
            assert channel_id == "UCdtcNj0e5bPpJLxLXRx_W_Q"

    def test_resolve_channel_id_with_browse_id(self, monitor):
        """Test channel ID resolution using browseId."""
        mock_response = MagicMock()
        mock_response.status_code = 200
        mock_response.text = """
        <script>
        {"browseId":"UCdtcNj0e5bPpJLxLXRx_W_Q"}
        </script>
        """

        with patch.object(monitor, "_request_with_backoff", return_value=mock_response):
            channel_id = monitor.resolve_channel_id_from_handle("@channel")
            assert channel_id == "UCdtcNj0e5bPpJLxLXRx_W_Q"

    def test_resolve_channel_id_adds_at_symbol(self, monitor):
        """Test that @ symbol is added if missing."""
        mock_response = MagicMock()
        mock_response.status_code = 200
        mock_response.text = '<meta itemprop="channelId" content="UCtest123">'

        with patch.object(
            monitor, "_request_with_backoff", return_value=mock_response
        ) as mock_request:
            channel_id = monitor.resolve_channel_id_from_handle("testchannel")
            assert channel_id == "UCtest123"
            # Verify URL was called with @ symbol
            mock_request.assert_called_with("https://www.youtube.com/@testchannel")

    def test_resolve_channel_id_not_found(self, monitor):
        """Test when channel ID cannot be found."""
        mock_response = MagicMock()
        mock_response.status_code = 200
        mock_response.text = "<html>No channel ID here</html>"

        with patch.object(monitor, "_request_with_backoff", return_value=mock_response):
            channel_id = monitor.resolve_channel_id_from_handle("@notfound")
            assert channel_id is None

    def test_resolve_channel_id_network_error(self, monitor):
        """Test when network request fails."""
        with patch.object(monitor, "_request_with_backoff", return_value=None):
            channel_id = monitor.resolve_channel_id_from_handle("@error")
            assert channel_id is None

    def test_resolve_channel_id_404(self, monitor):
        """Test when channel page returns 404."""
        mock_response = MagicMock()
        mock_response.status_code = 404

        with patch.object(monitor, "_request_with_backoff", return_value=mock_response):
            channel_id = monitor.resolve_channel_id_from_handle("@notexist")
            assert channel_id is None

    def test_resolve_channel_id_exception(self, monitor):
        """Test exception handling."""
        with patch.object(
            monitor, "_request_with_backoff", side_effect=Exception("Network error")
        ):
            channel_id = monitor.resolve_channel_id_from_handle("@error")
            assert channel_id is None

    def test_resolve_channel_id_malicious_input(self, monitor):
        """Test security - malicious input handling."""
        malicious_inputs = [
            "../../../etc/passwd",
            "@channel/../../../",
            "@channel%2F..%2F..",
            "@channel<script>alert()</script>",
            "@channel'; DROP TABLE users; --",
            "@channel\n\rSet-Cookie: evil=true",
            "https://evil.com/@channel",
            "@channel?param=value",
            "@channel#fragment",
            "@channel&param=value",
            "@channel|command",
            "@channel;command",
            "@channel`command`",
            "@channel$(command)",
            "@channel${variable}",
        ]

        for malicious_input in malicious_inputs:
            channel_id = monitor.resolve_channel_id_from_handle(malicious_input)
            assert channel_id is None, (
                f"Should reject malicious input: {malicious_input}"
            )

    def test_resolve_channel_id_special_characters(self, monitor):
        """Test that special characters are properly rejected."""
        invalid_handles = [
            "@channel!",
            "@channel@",
            "@channel#",
            "@channel$",
            "@channel%",
            "@channel^",
            "@channel&",
            "@channel*",
            "@channel(",
            "@channel)",
            "@channel+",
            "@channel=",
            "@channel[",
            "@channel]",
            "@channel{",
            "@channel}",
            "@channel|",
            "@channel\\",
            "@channel:",
            "@channel;",
            '@channel"',
            "@channel'",
            "@channel<",
            "@channel>",
            "@channel,",
            "@channel.",
            "@channel?",
            "@channel/",
            "@channel~",
            "@channel`",
            "@channel ",  # with space
            "@ channel",  # space after @
            "",  # empty
            " ",  # whitespace only
            "@",  # @ only
        ]

        for invalid_handle in invalid_handles:
            channel_id = monitor.resolve_channel_id_from_handle(invalid_handle)
            assert channel_id is None, (
                f"Should reject invalid handle: {repr(invalid_handle)}"
            )

    def test_resolve_channel_id_valid_characters(self, monitor):
        """Test that valid handles with allowed characters work."""
        valid_handles = [
            "@channel",
            "@channel123",
            "@channel_name",
            "@channel-name",
            "@Channel_123-test",
            "channel",  # without @
            "channel_123",
            "channel-name",
            "CHANNEL",
            "123channel",
        ]

        # Mock successful response
        mock_response = MagicMock()
        mock_response.status_code = 200
        mock_response.text = '<meta itemprop="channelId" content="UCtest123">'

        for valid_handle in valid_handles:
            with patch.object(
                monitor, "_request_with_backoff", return_value=mock_response
            ):
                channel_id = monitor.resolve_channel_id_from_handle(valid_handle)
                assert channel_id == "UCtest123", (
                    f"Should accept valid handle: {valid_handle}"
                )

    def test_resolve_channel_id_length_limits(self, monitor):
        """Test handle length validation."""
        # Too short
        assert monitor.resolve_channel_id_from_handle("") is None
        assert monitor.resolve_channel_id_from_handle("@") is None

        # Too long (more than 50 characters)
        long_handle = "@" + "a" * 50
        assert monitor.resolve_channel_id_from_handle(long_handle) is None

        # Valid length
        mock_response = MagicMock()
        mock_response.status_code = 200
        mock_response.text = '<meta itemprop="channelId" content="UCtest123">'

        # Minimum valid length (@ + 1 char)
        with patch.object(monitor, "_request_with_backoff", return_value=mock_response):
            channel_id = monitor.resolve_channel_id_from_handle("@a")
            assert channel_id == "UCtest123"

        # Normal length
        valid_handle = "@" + "a" * 20  # reasonable length
        with patch.object(monitor, "_request_with_backoff", return_value=mock_response):
            channel_id = monitor.resolve_channel_id_from_handle(valid_handle)
            assert channel_id == "UCtest123"

    def test_resolve_channel_id_none_input(self, monitor):
        """Test None input handling."""
        assert monitor.resolve_channel_id_from_handle(None) is None

    def test_resolve_channel_id_non_string_input(self, monitor):
        """Test non-string input handling."""
        assert monitor.resolve_channel_id_from_handle(123) is None
        assert monitor.resolve_channel_id_from_handle([]) is None
        assert monitor.resolve_channel_id_from_handle({}) is None

    def test_resolve_channel_id_whitespace_handling(self, monitor):
        """Test that whitespace is properly handled."""
        mock_response = MagicMock()
        mock_response.status_code = 200
        mock_response.text = '<meta itemprop="channelId" content="UCtest123">'

        # Handle with leading/trailing whitespace should be trimmed
        with patch.object(
            monitor, "_request_with_backoff", return_value=mock_response
        ) as mock_request:
            channel_id = monitor.resolve_channel_id_from_handle("  @channel  ")
            assert channel_id == "UCtest123"
            # Verify the URL was called with trimmed handle
            mock_request.assert_called_with("https://www.youtube.com/@channel")

    def test_resolve_channel_id_malformed_html_response(self, monitor):
        """Test handling of malformed HTML responses."""
        mock_response = MagicMock()
        mock_response.status_code = 200

        # Various malformed responses
        malformed_responses = [
            "",  # Empty response
            "Not HTML at all",  # Plain text
            "<html>",  # Unclosed HTML
            '{"json": "not html"}',  # JSON instead of HTML
            "<meta itemprop='channelId' content=>",  # Malformed meta tag
            "<meta itemprop=channelId content=UCtest>",  # Missing quotes
            None,  # None response
        ]

        for response_text in malformed_responses:
            mock_response.text = response_text
            with patch.object(
                monitor, "_request_with_backoff", return_value=mock_response
            ):
                channel_id = monitor.resolve_channel_id_from_handle("@channel")
                assert channel_id is None


class TestHandleValidation:
    """Test the shared handle validation function."""

    def test_validate_handle_format_valid_handles(self):
        """Test that valid handles pass validation."""
        valid_handles = [
            "@channel",
            "channel",
            "@channel123",
            "channel_name",
            "channel-name",
            "@Channel_123-test",
        ]

        for handle in valid_handles:
            is_valid, error_message = validate_handle_format(handle)
            assert is_valid, f"Should accept valid handle: {handle}"
            assert error_message == "", (
                f"Should have no error message for valid handle: {handle}"
            )

    def test_validate_handle_format_invalid_handles(self):
        """Test that invalid handles are rejected."""
        invalid_handles = [
            "",  # empty
            None,  # None
            "@",  # @ only
            "   ",  # whitespace only
            "@channel!",  # special character
            "@channel@test",  # multiple @
            "@channel space",  # space
            "@channel/path",  # slash
            123,  # non-string
        ]

        for handle in invalid_handles:
            is_valid, error_message = validate_handle_format(handle)
            assert not is_valid, f"Should reject invalid handle: {repr(handle)}"
            assert error_message != "", (
                f"Should have error message for invalid handle: {repr(handle)}"
            )

    def test_validate_handle_format_error_messages(self):
        """Test that appropriate error messages are returned."""
        # Empty handle
        is_valid, error_message = validate_handle_format("")
        assert not is_valid
        assert "empty" in error_message.lower()

        # None handle
        is_valid, error_message = validate_handle_format(None)
        assert not is_valid
        assert "empty" in error_message.lower()

        # Invalid characters
        is_valid, error_message = validate_handle_format("@channel!")
        assert not is_valid
        assert "letters, numbers, underscores, and dashes" in error_message
