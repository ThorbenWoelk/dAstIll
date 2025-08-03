"""Channel monitoring configuration management."""

import json
import os
from dataclasses import asdict, dataclass
from pathlib import Path
from typing import Any


@dataclass
class MonitoringSettings:
    """Settings for transcript monitoring."""
    languages: list[str] = None
    enabled: bool = True
    auto_download: bool = True
    auto_process: bool = False

    def __post_init__(self):
        if self.languages is None:
            self.languages = ["en"]


@dataclass
class ChannelConfig:
    """Configuration for a monitored channel."""
    name: str
    handle: str
    channel_id: str | None = None
    last_video_id: str | None = None
    monitoring: MonitoringSettings = None

    def __post_init__(self):
        if self.monitoring is None:
            self.monitoring = MonitoringSettings()


@dataclass
class GlobalMonitoringConfig:
    """Global monitoring configuration."""
    enabled: bool = False
    check_interval: int = 300  # seconds
    max_videos_per_check: int = 5
    notifications: dict[str, Any] = None

    def __post_init__(self):
        if self.notifications is None:
            self.notifications = {
                "enabled": True,
                "console": True,
                "log_file": True
            }


class ChannelConfigManager:
    """Manages channel monitoring configuration."""

    def __init__(self, config_dir: str | None = None):
        if config_dir is None:
            config_dir = os.path.expanduser("~/.dastill")

        self.config_dir = Path(config_dir)
        self.channels_file = self.config_dir / "channels.json"
        self.config_dir.mkdir(parents=True, exist_ok=True)

        self.channels: dict[str, ChannelConfig] = {}
        self.global_config = GlobalMonitoringConfig()

        self._load_configuration()

    def _load_configuration(self):
        """Load channel configuration from file."""
        if not self.channels_file.exists():
            self._create_default_config()
            return

        try:
            with open(self.channels_file, encoding='utf-8') as f:
                data = json.load(f)

            # Load global config
            global_data = data.get('monitoring', {})
            self.global_config = GlobalMonitoringConfig(**global_data)

            # Load channels
            channels_data = data.get('channels', {})
            self.channels = {}

            for handle, channel_data in channels_data.items():
                monitoring_data = channel_data.get('monitoring', {})
                monitoring = MonitoringSettings(**monitoring_data)

                channel = ChannelConfig(
                    name=channel_data['name'],
                    handle=handle,
                    channel_id=channel_data.get('channel_id'),
                    last_video_id=channel_data.get('last_video_id'),
                    monitoring=monitoring
                )
                self.channels[handle] = channel

        except Exception as e:
            print(f"Error loading channel configuration: {e}")
            self._create_default_config()

    def _create_default_config(self):
        """Create default configuration file."""
        self.channels = {}
        self.global_config = GlobalMonitoringConfig()
        self.save_configuration()

    def save_configuration(self):
        """Save current configuration to file."""
        try:
            data = {
                'monitoring': asdict(self.global_config),
                'channels': {}
            }

            for handle, channel in self.channels.items():
                data['channels'][handle] = asdict(channel)

            # Atomic write for safety
            temp_file = self.channels_file.with_suffix('.tmp')
            with open(temp_file, 'w', encoding='utf-8') as f:
                json.dump(data, f, indent=2)

            temp_file.replace(self.channels_file)

        except Exception as e:
            print(f"Error saving channel configuration: {e}")

    def add_channel(self, name: str, handle: str, **kwargs) -> bool:
        """Add a new channel to monitor."""
        # Normalize handle format
        if not handle.startswith('@'):
            handle = '@' + handle

        if handle in self.channels:
            return False  # Channel already exists

        # Create monitoring settings
        monitoring_kwargs = {}
        if 'languages' in kwargs:
            monitoring_kwargs['languages'] = kwargs.pop('languages')
        if 'auto_download' in kwargs:
            monitoring_kwargs['auto_download'] = kwargs.pop('auto_download')
        if 'auto_process' in kwargs:
            monitoring_kwargs['auto_process'] = kwargs.pop('auto_process')

        monitoring = MonitoringSettings(**monitoring_kwargs)

        channel = ChannelConfig(
            name=name,
            handle=handle,
            monitoring=monitoring,
            **kwargs
        )

        self.channels[handle] = channel
        self.save_configuration()
        return True

    def remove_channel(self, handle: str) -> bool:
        """Remove a channel from monitoring."""
        if not handle.startswith('@'):
            handle = '@' + handle

        if handle in self.channels:
            del self.channels[handle]
            self.save_configuration()
            return True
        return False

    def get_channel(self, handle: str) -> ChannelConfig | None:
        """Get channel configuration by handle."""
        if not handle.startswith('@'):
            handle = '@' + handle
        return self.channels.get(handle)

    def get_enabled_channels(self) -> list[ChannelConfig]:
        """Get all enabled channels."""
        return [ch for ch in self.channels.values() if ch.monitoring.enabled]

    def update_channel_id(self, handle: str, channel_id: str) -> bool:
        """Update the channel ID for a handle."""
        if not handle.startswith('@'):
            handle = '@' + handle

        if handle in self.channels:
            self.channels[handle].channel_id = channel_id
            self.save_configuration()
            return True
        return False

    def update_last_video_id(self, handle: str, video_id: str) -> bool:
        """Update the last processed video ID for a channel."""
        if not handle.startswith('@'):
            handle = '@' + handle

        if handle in self.channels:
            self.channels[handle].last_video_id = video_id
            self.save_configuration()
            return True
        return False

    def enable_channel(self, handle: str, enabled: bool = True) -> bool:
        """Enable or disable monitoring for a channel."""
        if not handle.startswith('@'):
            handle = '@' + handle

        if handle in self.channels:
            self.channels[handle].monitoring.enabled = enabled
            self.save_configuration()
            return True
        return False

    def set_global_monitoring(self, enabled: bool):
        """Enable or disable global monitoring."""
        self.global_config.enabled = enabled
        self.save_configuration()

    def set_check_interval(self, interval: int):
        """Set the check interval in seconds."""
        if interval >= 60:  # Minimum 1 minute
            self.global_config.check_interval = interval
            self.save_configuration()

    def list_channels(self, enabled_only: bool = False) -> list[ChannelConfig]:
        """List all channels, optionally filtering to enabled only."""
        channels = list(self.channels.values())
        if enabled_only:
            channels = [ch for ch in channels if ch.monitoring.enabled]
        return channels

    def get_stats(self) -> dict[str, Any]:
        """Get monitoring statistics."""
        total_channels = len(self.channels)
        enabled_channels = len(self.get_enabled_channels())

        channels_with_ids = len([ch for ch in self.channels.values() if ch.channel_id])
        channels_with_last_video = len([ch for ch in self.channels.values() if ch.last_video_id])

        return {
            'total_channels': total_channels,
            'enabled_channels': enabled_channels,
            'channels_with_ids': channels_with_ids,
            'channels_with_last_video': channels_with_last_video,
            'global_monitoring_enabled': self.global_config.enabled,
            'check_interval': self.global_config.check_interval
        }
