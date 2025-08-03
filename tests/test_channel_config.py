"""Tests for channel configuration management."""

import unittest
import tempfile
import shutil
import json
from pathlib import Path
from config.channel_config import ChannelConfigManager, ChannelConfig, MonitoringSettings


class TestChannelConfig(unittest.TestCase):
    """Test cases for ChannelConfigManager."""
    
    def setUp(self):
        """Set up test fixtures with temporary directory."""
        self.temp_dir = tempfile.mkdtemp()
        self.config_manager = ChannelConfigManager(self.temp_dir)
    
    def tearDown(self):
        """Clean up temporary directory."""
        shutil.rmtree(self.temp_dir)
    
    def test_initial_config_creation(self):
        """Test that initial configuration is created properly."""
        self.assertTrue(self.config_manager.channels_file.exists())
        self.assertEqual(len(self.config_manager.channels), 0)
        self.assertFalse(self.config_manager.global_config.enabled)
        self.assertEqual(self.config_manager.global_config.check_interval, 300)
    
    def test_add_channel(self):
        """Test adding a new channel."""
        success = self.config_manager.add_channel(
            name="Test Channel",
            handle="@testchannel",
            languages=["en", "de"],
            auto_download=True,
            auto_process=False
        )
        
        self.assertTrue(success)
        self.assertEqual(len(self.config_manager.channels), 1)
        
        channel = self.config_manager.get_channel("@testchannel")
        self.assertIsNotNone(channel)
        self.assertEqual(channel.name, "Test Channel")
        self.assertEqual(channel.handle, "@testchannel")
        self.assertEqual(channel.monitoring.languages, ["en", "de"])
        self.assertTrue(channel.monitoring.auto_download)
        self.assertFalse(channel.monitoring.auto_process)
    
    def test_add_channel_without_at_prefix(self):
        """Test adding a channel without @ prefix (should be added automatically)."""
        success = self.config_manager.add_channel(
            name="Test Channel",
            handle="testchannel"
        )
        
        self.assertTrue(success)
        channel = self.config_manager.get_channel("@testchannel")
        self.assertIsNotNone(channel)
        self.assertEqual(channel.handle, "@testchannel")
    
    def test_add_duplicate_channel(self):
        """Test that adding a duplicate channel fails."""
        # Add first channel
        success1 = self.config_manager.add_channel("Test Channel", "@testchannel")
        self.assertTrue(success1)
        
        # Try to add duplicate
        success2 = self.config_manager.add_channel("Another Channel", "@testchannel")
        self.assertFalse(success2)
        
        # Should still have only one channel
        self.assertEqual(len(self.config_manager.channels), 1)
    
    def test_remove_channel(self):
        """Test removing a channel."""
        # Add channel first
        self.config_manager.add_channel("Test Channel", "@testchannel")
        self.assertEqual(len(self.config_manager.channels), 1)
        
        # Remove channel
        success = self.config_manager.remove_channel("@testchannel")
        self.assertTrue(success)
        self.assertEqual(len(self.config_manager.channels), 0)
    
    def test_remove_nonexistent_channel(self):
        """Test removing a channel that doesn't exist."""
        success = self.config_manager.remove_channel("@nonexistent")
        self.assertFalse(success)
    
    def test_get_enabled_channels(self):
        """Test getting only enabled channels."""
        # Add enabled channel
        self.config_manager.add_channel("Enabled Channel", "@enabled")
        
        # Add disabled channel
        self.config_manager.add_channel("Disabled Channel", "@disabled")
        self.config_manager.enable_channel("@disabled", False)
        
        enabled_channels = self.config_manager.get_enabled_channels()
        self.assertEqual(len(enabled_channels), 1)
        self.assertEqual(enabled_channels[0].handle, "@enabled")
    
    def test_update_channel_id(self):
        """Test updating channel ID."""
        self.config_manager.add_channel("Test Channel", "@testchannel")
        
        success = self.config_manager.update_channel_id("@testchannel", "UC_12345")
        self.assertTrue(success)
        
        channel = self.config_manager.get_channel("@testchannel")
        self.assertEqual(channel.channel_id, "UC_12345")
    
    def test_update_last_video_id(self):
        """Test updating last video ID."""
        self.config_manager.add_channel("Test Channel", "@testchannel")
        
        success = self.config_manager.update_last_video_id("@testchannel", "video123")
        self.assertTrue(success)
        
        channel = self.config_manager.get_channel("@testchannel")
        self.assertEqual(channel.last_video_id, "video123")
    
    def test_enable_disable_channel(self):
        """Test enabling and disabling channels."""
        self.config_manager.add_channel("Test Channel", "@testchannel")
        
        # Disable channel
        success = self.config_manager.enable_channel("@testchannel", False)
        self.assertTrue(success)
        
        channel = self.config_manager.get_channel("@testchannel")
        self.assertFalse(channel.monitoring.enabled)
        
        # Enable channel
        success = self.config_manager.enable_channel("@testchannel", True)
        self.assertTrue(success)
        
        channel = self.config_manager.get_channel("@testchannel")
        self.assertTrue(channel.monitoring.enabled)
    
    def test_global_monitoring_settings(self):
        """Test global monitoring settings."""
        # Test enable
        self.config_manager.set_global_monitoring(True)
        self.assertTrue(self.config_manager.global_config.enabled)
        
        # Test disable
        self.config_manager.set_global_monitoring(False)
        self.assertFalse(self.config_manager.global_config.enabled)
    
    def test_check_interval_setting(self):
        """Test setting check interval."""
        # Valid interval
        self.config_manager.set_check_interval(600)
        self.assertEqual(self.config_manager.global_config.check_interval, 600)
        
        # Invalid interval (too small) should be rejected
        self.config_manager.set_check_interval(30)
        self.assertEqual(self.config_manager.global_config.check_interval, 600)  # Should remain unchanged
    
    def test_get_stats(self):
        """Test getting configuration statistics."""
        # Add some channels
        self.config_manager.add_channel("Channel 1", "@channel1")
        self.config_manager.add_channel("Channel 2", "@channel2")
        self.config_manager.enable_channel("@channel2", False)  # Disable one
        
        # Set some data
        self.config_manager.update_channel_id("@channel1", "UC_123")
        self.config_manager.update_last_video_id("@channel1", "video123")
        self.config_manager.set_global_monitoring(True)
        
        stats = self.config_manager.get_stats()
        
        self.assertEqual(stats['total_channels'], 2)
        self.assertEqual(stats['enabled_channels'], 1)
        self.assertEqual(stats['channels_with_ids'], 1)
        self.assertEqual(stats['channels_with_last_video'], 1)
        self.assertTrue(stats['global_monitoring_enabled'])
    
    def test_configuration_persistence(self):
        """Test that configuration persists across instances."""
        # Add data in first instance
        self.config_manager.add_channel("Persistent Channel", "@persistent")
        self.config_manager.update_channel_id("@persistent", "UC_persistent")
        self.config_manager.set_global_monitoring(True)
        self.config_manager.set_check_interval(900)
        
        # Create new instance with same config directory
        new_config_manager = ChannelConfigManager(self.temp_dir)
        
        # Verify data was loaded
        self.assertEqual(len(new_config_manager.channels), 1)
        self.assertTrue(new_config_manager.global_config.enabled)
        self.assertEqual(new_config_manager.global_config.check_interval, 900)
        
        channel = new_config_manager.get_channel("@persistent")
        self.assertIsNotNone(channel)
        self.assertEqual(channel.name, "Persistent Channel")
        self.assertEqual(channel.channel_id, "UC_persistent")
    
    def test_corrupted_config_handling(self):
        """Test handling of corrupted configuration file."""
        # Write invalid JSON to config file
        with open(self.config_manager.channels_file, 'w') as f:
            f.write("invalid json content")
        
        # Create new instance - should handle corruption gracefully
        new_config_manager = ChannelConfigManager(self.temp_dir)
        
        # Should have default configuration
        self.assertEqual(len(new_config_manager.channels), 0)
        self.assertFalse(new_config_manager.global_config.enabled)
    
    def test_list_channels_filtering(self):
        """Test listing channels with filtering."""
        # Add mix of enabled and disabled channels
        self.config_manager.add_channel("Enabled 1", "@enabled1")
        self.config_manager.add_channel("Enabled 2", "@enabled2")
        self.config_manager.add_channel("Disabled 1", "@disabled1")
        self.config_manager.enable_channel("@disabled1", False)
        
        # Test all channels
        all_channels = self.config_manager.list_channels()
        self.assertEqual(len(all_channels), 3)
        
        # Test enabled only
        enabled_channels = self.config_manager.list_channels(enabled_only=True)
        self.assertEqual(len(enabled_channels), 2)
        
        enabled_handles = [ch.handle for ch in enabled_channels]
        self.assertIn("@enabled1", enabled_handles)
        self.assertIn("@enabled2", enabled_handles)
        self.assertNotIn("@disabled1", enabled_handles)


if __name__ == '__main__':
    unittest.main()