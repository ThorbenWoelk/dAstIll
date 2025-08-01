import os
import json
from pathlib import Path
from typing import Dict, Any

# Import fcntl only on Unix systems
try:
    import fcntl
    HAS_FCNTL = True
except ImportError:
    HAS_FCNTL = False


class Config:
    def __init__(self, config_path: str = None):
        if config_path is None:
            config_path = os.path.expanduser("~/.dastill/config.json")
        
        self.config_path = Path(config_path)
        self.config_dir = self.config_path.parent
        self.config = self._load_config()
    
    def _load_config(self) -> Dict[str, Any]:
        if self.config_path.exists():
            with open(self.config_path, 'r', encoding='utf-8') as f:
                return json.load(f)
        else:
            return self._create_default_config()
    
    def _create_default_config(self) -> Dict[str, Any]:
        # Use a user-friendly location for transcripts (not hidden)
        home_dir = Path.home()
        transcripts_dir = home_dir / "dAstIll-transcripts"
        
        default_config = {
            "storage": {
                "base_path": str(transcripts_dir),
                "organize_by_date": True,
                "markdown_format": True
            },
            "tracking": {
                "database_path": str(self.config_dir / "videos.json")
            },
            "transcript": {
                "default_languages": ["en"],
                "include_metadata": True,
                "clean_transcript": True
            }
        }
        
        self.config_dir.mkdir(parents=True, exist_ok=True)
        self._save_config_data(default_config)
        
        return default_config
    
    def get(self, key: str, default=None):
        keys = key.split('.')
        value = self.config
        for k in keys:
            value = value.get(k, {})
            if not isinstance(value, dict) and k != keys[-1]:
                return default
        return value if value != {} else default
    
    def set(self, key: str, value: Any):
        keys = key.split('.')
        config = self.config
        for k in keys[:-1]:
            if k not in config:
                config[k] = {}
            config = config[k]
        config[keys[-1]] = value
        self._save_config()
    
    def _save_config(self):
        self._save_config_data(self.config)
    
    def _save_config_data(self, config_data: Dict[str, Any]):
        """Save config with atomic write and file locking to prevent race conditions."""
        temp_path = self.config_path.with_suffix('.tmp')
        
        try:
            with open(temp_path, 'w', encoding='utf-8') as f:
                # Lock the file to prevent concurrent writes (Unix only)
                if HAS_FCNTL:
                    fcntl.flock(f.fileno(), fcntl.LOCK_EX)
                json.dump(config_data, f, indent=2)
                f.flush()
                os.fsync(f.fileno())  # Ensure data is written to disk
            
            # Atomic rename - this is atomic on most filesystems
            temp_path.replace(self.config_path)
            
        except Exception as e:
            # Clean up temp file if something went wrong
            if temp_path.exists():
                temp_path.unlink()
            raise IOError(f"Failed to save configuration: {str(e)}")