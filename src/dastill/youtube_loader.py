from youtube_transcript_api import YouTubeTranscriptApi
from youtube_transcript_api.formatters import TextFormatter
from typing import List, Dict, Optional
import re
from urllib.parse import urlparse, parse_qs
from .config import Config
from .video_tracker import VideoTracker
from .markdown_storage import MarkdownStorage


class YouTubeTranscriptLoader:
    def __init__(self, config_path: str = None):
        self.formatter = TextFormatter()
        self.api = YouTubeTranscriptApi()
        self.config = Config(config_path)
        
        # Initialize tracker and storage
        tracker_path = self.config.get('tracking.database_path')
        self.tracker = VideoTracker(tracker_path)
        
        storage_path = self.config.get('storage.base_path')
        organize_by_date = self.config.get('storage.organize_by_date', True)
        self.storage = MarkdownStorage(storage_path, organize_by_date)
    
    def extract_video_id(self, url: str) -> Optional[str]:
        parsed = urlparse(url)
        
        if parsed.hostname in ['youtube.com', 'www.youtube.com']:
            if parsed.path == '/watch':
                video_id = parse_qs(parsed.query).get('v')
                return video_id[0] if video_id else None
            elif parsed.path.startswith('/embed/'):
                return parsed.path.split('/')[2]
        elif parsed.hostname in ['youtu.be', 'www.youtu.be']:
            return parsed.path[1:]
        
        return None
    
    def load_transcript(self, video_url_or_id: str, languages: List[str] = None, force: bool = False, save_markdown: bool = True) -> Dict[str, any]:
        if languages is None:
            languages = self.config.get('transcript.default_languages', ['en'])
        
        if video_url_or_id.startswith('http'):
            video_id = self.extract_video_id(video_url_or_id)
            if not video_id:
                raise ValueError(f"Could not extract video ID from URL: {video_url_or_id}")
        else:
            video_id = video_url_or_id
        
        # Check if video already processed
        if not force and self.tracker.is_video_processed(video_id):
            existing_info = self.tracker.get_video_info(video_id)
            if existing_info and existing_info.get('file_path'):
                return {
                    'video_id': video_id,
                    'language': existing_info.get('language'),
                    'is_generated': existing_info.get('is_generated'),
                    'already_exists': True,
                    'file_path': existing_info.get('file_path'),
                    'processed_at': existing_info.get('processed_at')
                }
        
        try:
            transcript_list = self.api.list(video_id)
            
            transcript = None
            for lang in languages:
                try:
                    transcript = transcript_list.find_transcript([lang])
                    break
                except:
                    continue
            
            if not transcript:
                transcript = transcript_list.find_generated_transcript(languages)
            
            raw_transcript = transcript.fetch()
            
            formatted_text = self.formatter.format_transcript(raw_transcript)
            
            cleaned_text = self.clean_transcript(formatted_text)
            
            transcript_data = {
                'video_id': video_id,
                'language': transcript.language,
                'is_generated': transcript.is_generated,
                'raw_transcript': raw_transcript,
                'formatted_text': formatted_text,
                'cleaned_text': cleaned_text,
                'languages_requested': languages,
                'already_exists': False
            }
            
            # Save as markdown if requested
            if save_markdown and self.config.get('storage.markdown_format', True):
                file_path = self.storage.save_transcript(transcript_data)
                transcript_data['file_path'] = file_path
                
                # Track the video
                self.tracker.add_video(video_id, transcript_data, file_path)
            
            return transcript_data
            
        except Exception as e:
            raise Exception(f"Failed to load transcript: {str(e)}")
    
    def clean_transcript(self, text: str) -> str:
        text = re.sub(r'\[.*?\]', '', text)
        
        text = re.sub(r'♪+', '', text)
        
        text = re.sub(r'\s+', ' ', text)
        
        text = text.strip()
        
        return text
    
    def save_transcript(self, transcript_data: Dict[str, any], filepath: str):
        with open(filepath, 'w', encoding='utf-8') as f:
            f.write(f"Video ID: {transcript_data['video_id']}\n")
            f.write(f"Language: {transcript_data['language']}\n")
            f.write(f"Auto-generated: {transcript_data['is_generated']}\n")
            f.write("-" * 50 + "\n\n")
            f.write(transcript_data['cleaned_text'])
    
    def list_processed_videos(self):
        return self.tracker.list_videos()
    
    def get_video_info(self, video_id: str):
        return self.tracker.get_video_info(video_id)
    
    def get_stats(self):
        return self.tracker.get_stats()
    
    def remove_video(self, video_id: str, delete_file: bool = False):
        video_info = self.tracker.get_video_info(video_id)
        if video_info and delete_file and video_info.get('file_path'):
            import os
            try:
                os.remove(video_info['file_path'])
            except OSError:
                pass
        
        return self.tracker.remove_video(video_id)