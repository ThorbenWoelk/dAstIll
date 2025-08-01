import os
import json
from pathlib import Path
from typing import Dict, Any


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
        default_config = {
            "storage": {
                "base_path": str(self.config_dir / "transcripts"),
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
        with open(self.config_path, 'w', encoding='utf-8') as f:
            json.dump(default_config, f, indent=2)
        
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
        with open(self.config_path, 'w', encoding='utf-8') as f:
            json.dump(self.config, f, indent=2)