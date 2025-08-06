import json
import os
import tempfile
import unittest

from config.config import Config


class TestConfig(unittest.TestCase):
    def setUp(self):
        self.temp_dir = tempfile.mkdtemp()
        self.config_path = os.path.join(self.temp_dir, "config.json")

    def tearDown(self):
        import shutil

        shutil.rmtree(self.temp_dir, ignore_errors=True)

    def test_create_default_config(self):
        """Test default configuration creation."""
        config = Config(self.config_path)

        # Check that config file was created
        self.assertTrue(os.path.exists(self.config_path))

        # Check default values
        self.assertEqual(config.get("transcript.default_languages"), ["en"])
        self.assertTrue(config.get("transcript.include_metadata"))
        self.assertTrue(config.get("transcript.clean_transcript"))
        self.assertTrue(config.get("storage.organize_by_date"))
        self.assertTrue(config.get("storage.markdown_format"))

    def test_load_existing_config(self):
        """Test loading existing configuration file."""
        # Create a custom config file
        custom_config = {
            "transcript": {"default_languages": ["de", "en"], "include_metadata": False}
        }

        with open(self.config_path, "w") as f:
            json.dump(custom_config, f)

        config = Config(self.config_path)

        self.assertEqual(config.get("transcript.default_languages"), ["de", "en"])
        self.assertFalse(config.get("transcript.include_metadata"))

    def test_get_nested_key(self):
        """Test getting nested configuration values."""
        config = Config(self.config_path)

        # Test existing nested key
        result = config.get("storage.base_path")
        self.assertIsInstance(result, str)

        # Test non-existent nested key with default
        result = config.get("nonexistent.key", "default_value")
        self.assertEqual(result, "default_value")

        # Test partially existing path
        result = config.get("storage.nonexistent", "default")
        self.assertEqual(result, "default")

    def test_set_nested_key(self):
        """Test setting nested configuration values."""
        config = Config(self.config_path)

        # Set existing nested key
        config.set("transcript.default_languages", ["fr", "es"])
        self.assertEqual(config.get("transcript.default_languages"), ["fr", "es"])

        # Set new nested key
        config.set("new.nested.key", "test_value")
        self.assertEqual(config.get("new.nested.key"), "test_value")

        # Verify persistence
        new_config = Config(self.config_path)
        self.assertEqual(new_config.get("transcript.default_languages"), ["fr", "es"])
        self.assertEqual(new_config.get("new.nested.key"), "test_value")

    def test_config_file_corruption_recovery(self):
        """Test handling of corrupted config files."""
        # Create a corrupted config file
        with open(self.config_path, "w") as f:
            f.write("invalid json content")

        # Config should fall back to defaults
        config = Config(self.config_path)
        self.assertEqual(config.get("transcript.default_languages"), ["en"])

    def test_atomic_write_safety(self):
        """Test that config saves are atomic and safe."""
        config = Config(self.config_path)

        # Modify config multiple times rapidly
        for i in range(10):
            config.set(f"test.key{i}", f"value{i}")

        # Verify all values were saved correctly
        for i in range(10):
            self.assertEqual(config.get(f"test.key{i}"), f"value{i}")

        # Verify file is valid JSON
        with open(self.config_path) as f:
            loaded_config = json.load(f)
            self.assertIsInstance(loaded_config, dict)

    def test_permission_error_handling(self):
        """Test handling of permission errors during save."""
        config = Config(self.config_path)

        # Make parent directory read-only (on Unix systems)
        if os.name != "nt":  # Skip on Windows
            os.chmod(self.temp_dir, 0o444)

            try:
                with self.assertRaises((IOError, OSError)) as context:
                    config.set("test.key", "value")
                # The exception should contain some indication of failure
                self.assertTrue(str(context.exception))
            finally:
                # Restore permissions for cleanup
                os.chmod(self.temp_dir, 0o755)

    def test_config_validation_max_recent_videos(self):
        """Test validation of monitoring.max_recent_videos config value."""
        config = Config(self.config_path)

        # Valid values should work
        config.set("monitoring.max_recent_videos", 1)
        self.assertEqual(config.get("monitoring.max_recent_videos"), 1)

        config.set("monitoring.max_recent_videos", 10)
        self.assertEqual(config.get("monitoring.max_recent_videos"), 10)

        config.set("monitoring.max_recent_videos", 20)
        self.assertEqual(config.get("monitoring.max_recent_videos"), 20)

        # Invalid values should raise ValueError
        with self.assertRaises(ValueError):
            config.set("monitoring.max_recent_videos", 0)

        with self.assertRaises(ValueError):
            config.set("monitoring.max_recent_videos", 21)

        with self.assertRaises(ValueError):
            config.set("monitoring.max_recent_videos", -1)

        with self.assertRaises(ValueError):
            config.set("monitoring.max_recent_videos", "not_a_number")

        with self.assertRaises(ValueError):
            config.set("monitoring.max_recent_videos", 50.5)

    def test_environment_variable_override_validation(self):
        """Test environment variable override with validation."""
        config = Config(self.config_path)

        # Test valid absolute path
        valid_path = "/tmp/test_transcripts"
        os.environ["DASTILL_BASE_PATH"] = valid_path
        try:
            result = config.get("storage.base_path")
            # Should resolve to absolute path
            self.assertTrue(os.path.isabs(result))
            self.assertIn("test_transcripts", result)
        finally:
            del os.environ["DASTILL_BASE_PATH"]

        # Test invalid relative path - should fall back to config
        os.environ["DASTILL_BASE_PATH"] = "relative/path"
        try:
            # Should fall back to default config value (not the invalid env var)
            result = config.get("storage.base_path")
            self.assertTrue(os.path.isabs(result))
            # Should contain default path elements
            self.assertIn("Documents", result)
        finally:
            del os.environ["DASTILL_BASE_PATH"]

        # Test home directory expansion
        os.environ["DASTILL_BASE_PATH"] = "~/test_transcripts"
        try:
            result = config.get("storage.base_path")
            self.assertTrue(os.path.isabs(result))
            self.assertNotIn("~", result)  # Should be expanded
        finally:
            del os.environ["DASTILL_BASE_PATH"]

    def test_config_hierarchy_documentation(self):
        """Test that config hierarchy is working as documented."""
        # Store original environment variable if it exists
        original_env = os.environ.get("DASTILL_BASE_PATH")
        if "DASTILL_BASE_PATH" in os.environ:
            del os.environ["DASTILL_BASE_PATH"]

        try:
            config = Config(self.config_path)

            # 1. Default value (lowest priority)
            default_value = config.get("storage.base_path")
            self.assertTrue(os.path.isabs(default_value))

            # 2. Config file value (middle priority)
            config.set("storage.base_path", "/config/file/path")
            config_file_value = config.get("storage.base_path")
            self.assertEqual(config_file_value, "/config/file/path")

            # 3. Environment variable (highest priority)
            os.environ["DASTILL_BASE_PATH"] = "/env/var/path"
            try:
                env_override_value = config.get("storage.base_path")
                self.assertEqual(env_override_value, "/env/var/path")
            finally:
                del os.environ["DASTILL_BASE_PATH"]

            # After removing env var, should fall back to config file
            fallback_value = config.get("storage.base_path")
            self.assertEqual(fallback_value, "/config/file/path")

        finally:
            # Restore original environment variable if it existed
            if original_env is not None:
                os.environ["DASTILL_BASE_PATH"] = original_env


if __name__ == "__main__":
    unittest.main()
