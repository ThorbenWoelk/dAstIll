import json
import os
from pathlib import Path
from typing import Any

from dotenv import load_dotenv

# Load .env file at module import
load_dotenv()

# Import fcntl only on Unix systems
try:
    import fcntl

    HAS_FCNTL = True
except ImportError:
    HAS_FCNTL = False


class Config:
    def __init__(self, config_path: str = None):
        if config_path is None:
            # Check environment variable first, then fallback to config directory
            config_dir = os.getenv(
                "DASTILL_CONFIG_DIR", os.path.join(os.getcwd(), "config")
            )
            os.makedirs(config_dir, exist_ok=True)
            config_path = os.path.join(config_dir, "config.json")

        self.config_path = Path(config_path)
        self.config_dir = self.config_path.parent
        self.config = self._load_config()

    def _load_config(self) -> dict[str, Any]:
        if self.config_path.exists():
            try:
                with open(self.config_path, encoding="utf-8") as f:
                    return json.load(f)
            except (json.JSONDecodeError, OSError):
                # If config is corrupted or unreadable, fall back to defaults
                return self._create_default_config()
        else:
            return self._create_default_config()

    def _create_default_config(self) -> dict[str, Any]:
        # Use a user-friendly location for transcripts (not hidden)
        home_dir = Path.home()

        default_config = {
            "storage": {
                "base_path": str(home_dir / "Documents" / "dAstIll" / "transcripts"),
                "markdown_format": True,
                "organize_by_date": True,
            },
            "transcript": {
                "default_languages": ["en"],
                "include_metadata": True,
                "clean_transcript": True,
                "method": "api_with_fallback",  # "api_only", "api_with_fallback", "browser_only"
            },
            "monitoring": {
                "max_recent_videos": 20,
            },
        }

        self.config_dir.mkdir(parents=True, exist_ok=True)
        self._save_config_data(default_config)

        return default_config

    def get(self, key: str, default=None):
        """Get configuration value with environment variable override support.

        Configuration hierarchy (highest to lowest priority):
        1. Environment variables (DASTILL_BASE_PATH)
        2. Configuration file values
        3. Default values

        Args:
            key: Configuration key in dot notation (e.g., "storage.base_path")
            default: Default value if key not found

        Returns:
            Configuration value with appropriate type casting and validation
        """
        # Check for environment variable overrides first
        env_value = self._get_env_override(key)
        if env_value is not None:
            return env_value

        keys = key.split(".")
        value = self.config
        for k in keys:
            value = value.get(k, {})
            if not isinstance(value, dict) and k != keys[-1]:
                return default
        return value if value != {} else default

    def _get_env_override(self, key: str) -> Any:
        """Get environment variable override for configuration keys.

        Supported environment variable overrides:
        - storage.base_path -> DASTILL_BASE_PATH

        Args:
            key: Configuration key to check for environment override

        Returns:
            Environment variable value if found and valid, None otherwise
        """
        if key == "storage.base_path":
            env_base_path = os.getenv("DASTILL_BASE_PATH")
            if env_base_path:
                # Validate environment variable path
                try:
                    # Expand user directory first
                    expanded_path = Path(env_base_path).expanduser()

                    # Check if the original path (before expansion) was absolute
                    if not Path(
                        env_base_path
                    ).is_absolute() and not env_base_path.startswith("~"):
                        raise ValueError(
                            "DASTILL_BASE_PATH must be an absolute path or start with ~"
                        )

                    # Resolve to get canonical absolute path
                    resolved_path = expanded_path.resolve()
                    return str(resolved_path)
                except (OSError, ValueError) as e:
                    print(
                        f"Warning: Invalid DASTILL_BASE_PATH environment variable: {e}"
                    )
                    print("Falling back to configuration file value")
                    return None

        return None

    def set(self, key: str, value: Any):
        # Validate specific config values
        self._validate_config_value(key, value)

        keys = key.split(".")
        config = self.config
        for k in keys[:-1]:
            if k not in config:
                config[k] = {}
            config = config[k]
        config[keys[-1]] = value
        self._save_config()

    def _validate_config_value(self, key: str, value: Any):
        """Validate specific configuration values for security and correctness."""
        if key == "monitoring.max_recent_videos":
            if not isinstance(value, int) or value < 1 or value > 20:
                raise ValueError(
                    f"monitoring.max_recent_videos must be an integer between 1-20 (RSS feed limit), got: {value}"
                )

    def _save_config(self):
        self._save_config_data(self.config)

    def _save_config_data(self, config_data: dict[str, Any]):
        """Save config with atomic write and file locking to prevent race conditions."""
        temp_path = self.config_path.with_suffix(".tmp")

        try:
            with open(temp_path, "w", encoding="utf-8") as f:
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
            raise OSError(f"Failed to save configuration: {str(e)}") from e
