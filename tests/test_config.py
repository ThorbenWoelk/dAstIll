import unittest
import tempfile
import os
import json
from pathlib import Path
from config.config import Config


class TestConfig(unittest.TestCase):
    def setUp(self):
        self.temp_dir = tempfile.mkdtemp()
        self.config_path = os.path.join(self.temp_dir, 'config.json')
    
    def tearDown(self):
        import shutil
        shutil.rmtree(self.temp_dir, ignore_errors=True)
    
    def test_create_default_config(self):
        """Test default configuration creation."""
        config = Config(self.config_path)
        
        # Check that config file was created
        self.assertTrue(os.path.exists(self.config_path))
        
        # Check default values
        self.assertEqual(config.get('transcript.default_languages'), ['en'])
        self.assertTrue(config.get('transcript.include_metadata'))
        self.assertTrue(config.get('transcript.clean_transcript'))
        self.assertTrue(config.get('storage.organize_by_date'))
        self.assertTrue(config.get('storage.markdown_format'))
    
    def test_load_existing_config(self):
        """Test loading existing configuration file."""
        # Create a custom config file
        custom_config = {
            'transcript': {
                'default_languages': ['de', 'en'],
                'include_metadata': False
            }
        }
        
        with open(self.config_path, 'w') as f:
            json.dump(custom_config, f)
        
        config = Config(self.config_path)
        
        self.assertEqual(config.get('transcript.default_languages'), ['de', 'en'])
        self.assertFalse(config.get('transcript.include_metadata'))
    
    def test_get_nested_key(self):
        """Test getting nested configuration values."""
        config = Config(self.config_path)
        
        # Test existing nested key
        result = config.get('storage.base_path')
        self.assertIsInstance(result, str)
        
        # Test non-existent nested key with default
        result = config.get('nonexistent.key', 'default_value')
        self.assertEqual(result, 'default_value')
        
        # Test partially existing path
        result = config.get('storage.nonexistent', 'default')
        self.assertEqual(result, 'default')
    
    def test_set_nested_key(self):
        """Test setting nested configuration values."""
        config = Config(self.config_path)
        
        # Set existing nested key
        config.set('transcript.default_languages', ['fr', 'es'])
        self.assertEqual(config.get('transcript.default_languages'), ['fr', 'es'])
        
        # Set new nested key
        config.set('new.nested.key', 'test_value')
        self.assertEqual(config.get('new.nested.key'), 'test_value')
        
        # Verify persistence
        new_config = Config(self.config_path)
        self.assertEqual(new_config.get('transcript.default_languages'), ['fr', 'es'])
        self.assertEqual(new_config.get('new.nested.key'), 'test_value')
    
    def test_config_file_corruption_recovery(self):
        """Test handling of corrupted config files."""
        # Create a corrupted config file
        with open(self.config_path, 'w') as f:
            f.write('invalid json content')
        
        # Config should fall back to defaults
        config = Config(self.config_path)
        self.assertEqual(config.get('transcript.default_languages'), ['en'])
    
    def test_atomic_write_safety(self):
        """Test that config saves are atomic and safe."""
        config = Config(self.config_path)
        
        # Modify config multiple times rapidly
        for i in range(10):
            config.set(f'test.key{i}', f'value{i}')
        
        # Verify all values were saved correctly
        for i in range(10):
            self.assertEqual(config.get(f'test.key{i}'), f'value{i}')
        
        # Verify file is valid JSON
        with open(self.config_path, 'r') as f:
            loaded_config = json.load(f)
            self.assertIsInstance(loaded_config, dict)
    
    def test_permission_error_handling(self):
        """Test handling of permission errors during save."""
        config = Config(self.config_path)
        
        # Make parent directory read-only (on Unix systems)
        if os.name != 'nt':  # Skip on Windows
            os.chmod(self.temp_dir, 0o444)
            
            try:
                with self.assertRaises((IOError, OSError)) as context:
                    config.set('test.key', 'value')
                # The exception should contain some indication of failure
                self.assertTrue(str(context.exception))
            finally:
                # Restore permissions for cleanup
                os.chmod(self.temp_dir, 0o755)


if __name__ == '__main__':
    unittest.main()