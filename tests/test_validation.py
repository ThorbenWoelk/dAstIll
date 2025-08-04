"""Tests for validation functions."""

from unittest.mock import patch

from main import check_disk_space, validate_channel_id


class TestChannelIdValidation:
    """Test channel ID validation function."""

    def test_valid_standard_channel_ids(self):
        """Test valid UC format channel IDs."""
        valid_ids = [
            "UC1234567890123456789012",  # Standard format
            "UCabcdefghijklmnopqrstuv",  # With letters
            "UC_____________________X",  # With underscores
            "UC---------------------Y",  # With hyphens
            "UC123abc456def789ghi012j",  # Mixed alphanumeric
        ]

        for channel_id in valid_ids:
            assert validate_channel_id(channel_id), (
                f"Should accept valid UC ID: {channel_id}"
            )

    def test_valid_legacy_usernames(self):
        """Test valid legacy usernames and custom channel names."""
        valid_names = [
            "google",
            "TestChannel",
            "user123",
            "channel_name",
            "MyChannel123",
            "user_123_test",
            "a1b2c3",
            "X",  # Single character
        ]

        for name in valid_names:
            assert validate_channel_id(name), f"Should accept valid legacy name: {name}"

    def test_invalid_channel_ids(self):
        """Test invalid channel ID formats."""
        invalid_ids = [
            "",  # Empty string
            "UC123",  # Too short for UC format
            "UC12345678901234567890123",  # Too long for UC format
            "AB1234567890123456789012",  # Wrong prefix for 24-char format
            "UC123456789012345678901@",  # Invalid character in UC format
            "UC123456789012345678901#",  # Invalid character in UC format
            "UC123456789012345678901 ",  # Space in UC format
            "../../../etc/passwd",  # Path traversal attempt
            "test..channel",  # Consecutive periods
            ".hidden",  # Starting with period
            "test.channel",  # Single period (security risk for hidden files)
            "Channel.Name.Test",  # Multiple periods (security risk)
            "test/channel",  # Forward slash
            "test\\channel",  # Backslash
            "test channel",  # Space
            "test@channel",  # At symbol
            "test#channel",  # Hash symbol
            "test$channel",  # Dollar symbol
            "test%channel",  # Percent symbol
            "test&channel",  # Ampersand
            "test*channel",  # Asterisk
            "test+channel",  # Plus sign
            "test=channel",  # Equals sign
            "test|channel",  # Pipe symbol
            "ch",  # Too short (less than 3 chars for legacy)
            "a" * 31,  # Too long (more than 30 chars for legacy)
            "123456789012345678901234567890X",  # Too long overall
        ]

        for channel_id in invalid_ids:
            assert not validate_channel_id(channel_id), (
                f"Should reject invalid ID: {channel_id}"
            )

    def test_edge_cases(self):
        """Test edge cases for channel ID validation."""
        # Boundary cases for legacy names
        assert validate_channel_id("abc")  # Minimum length
        assert validate_channel_id("a" * 30)  # Maximum length
        assert not validate_channel_id("ab")  # Too short
        assert not validate_channel_id("a" * 31)  # Too long

        # UC format boundary cases
        assert validate_channel_id("UC" + "a" * 22)  # Exactly 24 chars
        assert not validate_channel_id("UC" + "a" * 21)  # 23 chars
        assert not validate_channel_id("UC" + "a" * 23)  # 25 chars

        # Special valid cases
        assert validate_channel_id("a")  # Single alphanumeric
        assert validate_channel_id("9")  # Single digit
        assert not validate_channel_id(".")  # Single period
        assert not validate_channel_id("_")  # Single underscore

    def test_security_cases(self):
        """Test security-related validation cases."""
        malicious_inputs = [
            "../",
            "../../",
            "../../../etc/passwd",
            "..\\",
            "test/../admin",
            "test\\..\\admin",
            "..",
            "....",
            "test..test",
            ".test",
            "test.",
            "CON",  # Windows reserved name
            "PRN",  # Windows reserved name
            "AUX",  # Windows reserved name
            "NUL",  # Windows reserved name
        ]

        for malicious in malicious_inputs:
            assert not validate_channel_id(malicious), (
                f"Should reject malicious input: {malicious}"
            )


class TestDiskSpaceValidation:
    """Test disk space checking function."""

    @patch("main.shutil.disk_usage")
    def test_sufficient_disk_space(self, mock_disk_usage):
        """Test when there is sufficient disk space."""
        # Mock disk usage: total, used, free (in bytes)
        # 1GB free = 1024 * 1024 * 1024 bytes
        mock_disk_usage.return_value = (2 * 1024**3, 1024**3, 1024**3)

        # Check for 100MB (default)
        assert check_disk_space("/test/path") is True

        # Check for 500MB (should still pass with 1GB free)
        assert check_disk_space("/test/path", 500) is True

    @patch("main.shutil.disk_usage")
    def test_insufficient_disk_space(self, mock_disk_usage):
        """Test when there is insufficient disk space."""
        # Mock disk usage: 50MB free
        mock_disk_usage.return_value = (1024**3, 1024**3 - 50 * 1024**2, 50 * 1024**2)

        # Check for 100MB (should fail with only 50MB free)
        assert check_disk_space("/test/path", 100) is False

        # Check for 200MB (should fail)
        assert check_disk_space("/test/path", 200) is False

    @patch("main.shutil.disk_usage")
    def test_exact_disk_space_boundary(self, mock_disk_usage):
        """Test boundary conditions for disk space."""
        # Mock disk usage: exactly 100MB free
        mock_disk_usage.return_value = (1024**3, 1024**3 - 100 * 1024**2, 100 * 1024**2)

        # Check for exactly 100MB (should pass)
        assert check_disk_space("/test/path", 100) is True

        # Check for 101MB (should fail)
        assert check_disk_space("/test/path", 101) is False

    @patch("main.shutil.disk_usage")
    def test_disk_usage_exception(self, mock_disk_usage):
        """Test when disk usage check raises an exception."""
        mock_disk_usage.side_effect = OSError("Permission denied")

        # Should return True when exception occurs (assume space available)
        assert check_disk_space("/test/path", 100) is True

    def test_disk_space_default_requirement(self):
        """Test that default disk space requirement is reasonable."""
        with patch("main.shutil.disk_usage") as mock_disk_usage:
            # Mock 200MB free
            mock_disk_usage.return_value = (
                1024**3,
                1024**3 - 200 * 1024**2,
                200 * 1024**2,
            )

            # Default requirement (100MB) should pass
            assert check_disk_space("/test/path") is True

            # Mock 50MB free
            mock_disk_usage.return_value = (
                1024**3,
                1024**3 - 50 * 1024**2,
                50 * 1024**2,
            )

            # Default requirement should fail with only 50MB
            assert check_disk_space("/test/path") is False
