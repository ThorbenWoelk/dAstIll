import os
from pathlib import Path
from typing import Dict, List, Optional, Any, Tuple
from datetime import datetime


class StatelessVideoManager:
    def __init__(self, base_path: str):
        self.base_path = Path(base_path)
        self.to_be_downloaded_path = self.base_path / "to_be_downloaded"
        self.downloaded_path = self.base_path / "downloaded"
        
        # Create necessary directories
        self.to_be_downloaded_path.mkdir(parents=True, exist_ok=True)
        self.downloaded_path.mkdir(parents=True, exist_ok=True)
        (self.base_path / "unknown").mkdir(parents=True, exist_ok=True)
    
    def get_video_status(self, video_id: str) -> Tuple[str, Optional[str]]:
        """
        Get the current status of a video based on file system.
        Returns (status, file_path) where status is one of:
        'not_downloaded', 'to_be_downloaded', 'downloaded', 'processed'
        """
        # Check for processed files in channel folders
        for channel_dir in self.base_path.iterdir():
            if channel_dir.is_dir() and channel_dir.name not in ['to_be_downloaded', 'downloaded']:
                for pattern in [f"{video_id}.md", f"{video_id}_*.md"]:
                    matches = list(channel_dir.glob(pattern))
                    if matches:
                        return 'processed', str(matches[0])
        
        # Check downloaded folder
        for pattern in [f"{video_id}.md", f"{video_id}_*.md"]:
            matches = list(self.downloaded_path.glob(pattern))
            if matches:
                return 'downloaded', str(matches[0])
        
        # Check to_be_downloaded folder
        for pattern in [f"{video_id}.md", f"{video_id}_*.md"]:
            matches = list(self.to_be_downloaded_path.glob(pattern))
            if matches:
                return 'to_be_downloaded', str(matches[0])
        
        return 'not_downloaded', None
    
    def add_to_be_downloaded(self, video_id: str, channel: str = 'unknown') -> bool:
        """Create an empty placeholder file for a video to be downloaded."""
        status, _ = self.get_video_status(video_id)
        if status != 'not_downloaded':
            return False
        
        # Create empty file with channel info in filename for later reference
        filename = f"{video_id}_{channel}.md" if channel != 'unknown' else f"{video_id}.md"
        placeholder_file = self.to_be_downloaded_path / filename
        
        try:
            with open(placeholder_file, 'w', encoding='utf-8') as f:
                f.write(f"# Placeholder for {video_id}\n\nChannel: {channel}\nAdded: {datetime.now().isoformat()}\n")
            return True
        except Exception:
            return False
    
    def mark_downloaded(self, video_id: str, transcript_content: str, channel: str = 'unknown') -> Optional[str]:
        """Move from to_be_downloaded to downloaded and write actual content."""
        status, current_path = self.get_video_status(video_id)
        
        if status == 'to_be_downloaded':
            # Remove placeholder
            if current_path:
                Path(current_path).unlink()
        
        # Create file in downloaded folder
        filename = f"{video_id}_{channel}.md" if channel != 'unknown' else f"{video_id}.md"
        downloaded_file = self.downloaded_path / filename
        
        try:
            with open(downloaded_file, 'w', encoding='utf-8') as f:
                f.write(transcript_content)
            return str(downloaded_file)
        except Exception:
            return None
    
    def mark_processed(self, video_id: str, channel: str = 'unknown') -> Optional[str]:
        """Move from downloaded to channel folder."""
        status, current_path = self.get_video_status(video_id)
        
        if status != 'downloaded':
            return None
        
        if not current_path or not Path(current_path).exists():
            return None
        
        # Create channel folder if needed
        channel_path = self.base_path / channel
        channel_path.mkdir(parents=True, exist_ok=True)
        
        # Determine new filename
        current_file = Path(current_path)
        if channel != 'unknown':
            new_filename = f"{video_id}_{channel}.md"
        else:
            new_filename = f"{video_id}.md"
        
        new_path = channel_path / new_filename
        
        try:
            # Read content from downloaded file
            with open(current_path, 'r', encoding='utf-8') as f:
                content = f.read()
            
            # Write to channel folder
            with open(new_path, 'w', encoding='utf-8') as f:
                f.write(content)
            
            # Remove from downloaded folder
            current_file.unlink()
            
            return str(new_path)
        except Exception:
            return None
    
    def list_videos_by_status(self, status: str) -> List[Dict[str, str]]:
        """List all videos with a specific status."""
        videos = []
        
        if status == 'to_be_downloaded':
            for file_path in self.to_be_downloaded_path.glob("*.md"):
                video_id = self._extract_video_id_from_filename(file_path.name)
                channel = self._extract_channel_from_filename(file_path.name, video_id)
                videos.append({
                    'video_id': video_id,
                    'status': status,
                    'channel': channel,
                    'file_path': str(file_path)
                })
        
        elif status == 'downloaded':
            for file_path in self.downloaded_path.glob("*.md"):
                video_id = self._extract_video_id_from_filename(file_path.name)
                channel = self._extract_channel_from_filename(file_path.name, video_id)
                videos.append({
                    'video_id': video_id,
                    'status': status,
                    'channel': channel,
                    'file_path': str(file_path)
                })
        
        elif status == 'processed':
            for channel_dir in self.base_path.iterdir():
                if channel_dir.is_dir() and channel_dir.name not in ['to_be_downloaded', 'downloaded']:
                    for file_path in channel_dir.glob("*.md"):
                        video_id = self._extract_video_id_from_filename(file_path.name)
                        videos.append({
                            'video_id': video_id,
                            'status': status,
                            'channel': channel_dir.name,
                            'file_path': str(file_path)
                        })
        
        return videos
    
    def list_all_videos(self) -> List[Dict[str, str]]:
        """List all videos across all statuses."""
        all_videos = []
        for status in ['to_be_downloaded', 'downloaded', 'processed']:
            all_videos.extend(self.list_videos_by_status(status))
        return all_videos
    
    def get_stats(self) -> Dict[str, Any]:
        """Get statistics about videos."""
        stats = {
            'to_be_downloaded': 0,
            'downloaded': 0,
            'processed': 0,
            'channels': {}
        }
        
        for video in self.list_all_videos():
            status = video['status']
            channel = video['channel']
            
            stats[status] += 1
            if channel not in stats['channels']:
                stats['channels'][channel] = 0
            stats['channels'][channel] += 1
        
        stats['total'] = sum([stats['to_be_downloaded'], stats['downloaded'], stats['processed']])
        return stats
    
    def remove_video(self, video_id: str, delete_file: bool = False) -> Dict[str, Any]:
        """Remove a video from tracking (and optionally delete file)."""
        status, file_path = self.get_video_status(video_id)
        
        if status == 'not_downloaded':
            return {'found': False, 'file_deleted': False, 'error': None}
        
        file_deleted = False
        error = None
        
        if delete_file and file_path:
            try:
                Path(file_path).unlink()
                file_deleted = True
            except Exception as e:
                error = str(e)
        
        return {
            'found': True,
            'file_deleted': file_deleted,
            'error': error,
            'previous_status': status
        }
    
    def _extract_video_id_from_filename(self, filename: str) -> str:
        """Extract video ID from filename (first 11 characters typically for YouTube IDs)."""
        # Remove .md extension
        name_part = filename[:-3]
        
        # YouTube video IDs are typically 11 characters
        # Look for common patterns
        if len(name_part) >= 11:
            # Try to find video ID at start of filename
            potential_id = name_part[:11]
            if self._is_valid_video_id(potential_id):
                return potential_id
        
        # Fallback: take everything before first underscore, or whole name
        parts = name_part.split('_')
        return parts[0] if parts else name_part
    
    def _is_valid_video_id(self, video_id: str) -> bool:
        """Basic validation for YouTube video ID format."""
        return len(video_id) == 11 and video_id.replace('-', '').replace('_', '').isalnum()
    
    def _extract_channel_from_filename(self, filename: str, video_id: str) -> str:
        """Extract channel from filename, default to 'unknown'."""
        # Remove .md extension
        name_part = filename[:-3]
        
        # If filename is just video_id.md, channel is unknown
        if name_part == video_id:
            return 'unknown'
        
        # If filename is video_id_channel.md, extract channel
        if name_part.startswith(f"{video_id}_"):
            return name_part[len(f"{video_id}_"):]
        
        return 'unknown'