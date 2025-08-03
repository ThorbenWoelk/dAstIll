#!/usr/bin/env python3
"""
Migration script to move existing Tina Huang transcript files to the downloaded folder 
and update their channel information in the database.
"""
import os
import shutil
from pathlib import Path
from src.dastill.config import Config
from src.dastill.video_tracker import VideoTracker

def main():
    # Source directory (where files currently are)
    source_dir = Path("/Users/thorben.woelk/Documents/totos-vault/AI Memory/youtube library/Tina Huang")
    
    # Destination directory (where they should go)
    dest_dir = Path("/Users/thorben.woelk/Documents/totos-vault/AI Memory/youtube library/downloaded")
    
    # Ensure destination directory exists
    dest_dir.mkdir(parents=True, exist_ok=True)
    
    # Initialize tracker
    config = Config()
    tracker_path = config.get('tracking.database_path')
    tracker = VideoTracker(tracker_path)
    
    if not source_dir.exists():
        print(f"Source directory does not exist: {source_dir}")
        return
    
    moved_count = 0
    updated_count = 0
    
    # Get all .md files in the source directory
    for file_path in source_dir.glob("*.md"):
        video_id = file_path.stem  # Get filename without extension
        
        # Check if this video is in our tracker
        video_info = tracker.get_video_info(video_id)
        if not video_info:
            print(f"⚠ Video {video_id} not found in tracker, skipping file: {file_path.name}")
            continue
        
        # Move the file
        dest_file = dest_dir / file_path.name
        try:
            shutil.move(str(file_path), str(dest_file))
            print(f"✓ Moved {file_path.name} to downloaded folder")
            moved_count += 1
            
            # Update the tracker with the new file path and channel
            # Update both channel and file path
            video_info['channel'] = 'tina huang'
            video_info['file_path'] = str(dest_file)
            tracker._save_database()
            updated_count += 1
            
        except Exception as e:
            print(f"✗ Error moving {file_path.name}: {str(e)}")
    
    print(f"\nMigration complete:")
    print(f"Files moved: {moved_count}")
    print(f"Database entries updated: {updated_count}")

if __name__ == "__main__":
    main()