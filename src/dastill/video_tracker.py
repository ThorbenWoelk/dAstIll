import json
import os
import tempfile
from pathlib import Path
from datetime import datetime
from typing import Dict, List, Optional, Any

# Import fcntl only on Unix systems
try:
    import fcntl
    HAS_FCNTL = True
except ImportError:
    HAS_FCNTL = False


class VideoTracker:
    def __init__(self, database_path: str):
        self.database_path = Path(database_path)
        self.database_path.parent.mkdir(parents=True, exist_ok=True)
        self.videos = self._load_database()
    
    def _load_database(self) -> Dict[str, Any]:
        if self.database_path.exists():
            with open(self.database_path, 'r', encoding='utf-8') as f:
                return json.load(f)
        else:
            return {}
    
    def _save_database(self):
        """Save database with atomic write and file locking to prevent race conditions."""
        # Use atomic write with temporary file to prevent corruption
        temp_path = self.database_path.with_suffix('.tmp')
        
        try:
            with open(temp_path, 'w', encoding='utf-8') as f:
                # Lock the file to prevent concurrent writes (Unix only)
                if HAS_FCNTL:
                    fcntl.flock(f.fileno(), fcntl.LOCK_EX)
                json.dump(self.videos, f, indent=2, default=str)
                f.flush()
                os.fsync(f.fileno())  # Ensure data is written to disk
            
            # Atomic rename - this is atomic on most filesystems
            temp_path.replace(self.database_path)
            
        except Exception as e:
            # Clean up temp file if something went wrong
            if temp_path.exists():
                temp_path.unlink()
            raise IOError(f"Failed to save video database: {str(e)}")
    
    def is_video_processed(self, video_id: str) -> bool:
        return video_id in self.videos
    
    def migrate_legacy_videos(self):
        """Migrate videos without status and channel fields."""
        migrated_status = 0
        migrated_channel = 0
        
        for video_id, video_data in self.videos.items():
            if 'status' not in video_data:
                video_data['status'] = 'downloaded'
                migrated_status += 1
            if 'channel' not in video_data:
                video_data['channel'] = 'unknown'
                migrated_channel += 1
        
        if migrated_status > 0 or migrated_channel > 0:
            self._save_database()
            if migrated_status > 0:
                print(f"Migrated {migrated_status} videos to include status field")
            if migrated_channel > 0:
                print(f"Migrated {migrated_channel} videos to include channel field")
        
        return migrated_status + migrated_channel
    
    def add_video(self, video_id: str, transcript_data: Dict[str, Any], file_path: str, status: str = 'downloaded', channel: str = 'unknown'):
        self.videos[video_id] = {
            'video_id': video_id,
            'language': transcript_data.get('language'),
            'is_generated': transcript_data.get('is_generated'),
            'processed_at': datetime.now().isoformat(),
            'file_path': file_path,
            'title': transcript_data.get('title', ''),
            'duration': transcript_data.get('duration', ''),
            'status': status,
            'channel': channel,
            'metadata': {
                'languages_requested': transcript_data.get('languages_requested', []),
                'file_size': os.path.getsize(file_path) if os.path.exists(file_path) else 0
            }
        }
        self._save_database()
    
    def get_video_info(self, video_id: str) -> Optional[Dict[str, Any]]:
        return self.videos.get(video_id)
    
    def list_videos(self) -> List[Dict[str, Any]]:
        return list(self.videos.values())
    
    def remove_video(self, video_id: str) -> bool:
        if video_id in self.videos:
            del self.videos[video_id]
            self._save_database()
            return True
        return False
    
    def get_stats(self) -> Dict[str, Any]:
        total_videos = len(self.videos)
        languages = {}
        generated_count = 0
        status_counts = {'to_be_downloaded': 0, 'downloaded': 0, 'processed': 0}
        
        for video in self.videos.values():
            lang = video.get('language', 'unknown')
            languages[lang] = languages.get(lang, 0) + 1
            if video.get('is_generated', False):
                generated_count += 1
            status = video.get('status', 'downloaded')
            if status in status_counts:
                status_counts[status] += 1
        
        return {
            'total_videos': total_videos,
            'languages': languages,
            'auto_generated_count': generated_count,
            'manual_transcript_count': total_videos - generated_count,
            'status_counts': status_counts
        }
    
    def update_status(self, video_id: str, new_status: str, new_file_path: str = None) -> bool:
        """Update the status of a video and optionally its file path."""
        if video_id not in self.videos:
            return False
        
        if new_status not in ['to_be_downloaded', 'downloaded', 'processed']:
            raise ValueError(f"Invalid status: {new_status}")
        
        self.videos[video_id]['status'] = new_status
        if new_file_path:
            self.videos[video_id]['file_path'] = new_file_path
        
        # Update timestamp when status changes
        self.videos[video_id][f'{new_status}_at'] = datetime.now().isoformat()
        
        self._save_database()
        return True
    
    def list_videos_by_status(self, status: str) -> List[Dict[str, Any]]:
        """List all videos with a specific status."""
        return [v for v in self.videos.values() if v.get('status', 'downloaded') == status]
    
    def add_to_be_downloaded(self, video_id: str, title: str = '', channel: str = 'unknown') -> bool:
        """Add a video ID to be downloaded later."""
        if video_id in self.videos:
            return False
        
        self.videos[video_id] = {
            'video_id': video_id,
            'status': 'to_be_downloaded',
            'added_at': datetime.now().isoformat(),
            'title': title,
            'channel': channel
        }
        self._save_database()
        return True