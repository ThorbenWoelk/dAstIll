"""Test channel ID resolution functionality."""

from unittest.mock import MagicMock, patch

import pytest

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
