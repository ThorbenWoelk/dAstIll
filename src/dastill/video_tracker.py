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
    
    def add_video(self, video_id: str, transcript_data: Dict[str, Any], file_path: str):
        self.videos[video_id] = {
            'video_id': video_id,
            'language': transcript_data.get('language'),
            'is_generated': transcript_data.get('is_generated'),
            'processed_at': datetime.now().isoformat(),
            'file_path': file_path,
            'title': transcript_data.get('title', ''),
            'duration': transcript_data.get('duration', ''),
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
        
        for video in self.videos.values():
            lang = video.get('language', 'unknown')
            languages[lang] = languages.get(lang, 0) + 1
            if video.get('is_generated', False):
                generated_count += 1
        
        return {
            'total_videos': total_videos,
            'languages': languages,
            'auto_generated_count': generated_count,
            'manual_transcript_count': total_videos - generated_count
        }