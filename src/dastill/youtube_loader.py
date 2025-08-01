from youtube_transcript_api import YouTubeTranscriptApi
from youtube_transcript_api.formatters import TextFormatter
from typing import List, Dict, Optional
import re
from urllib.parse import urlparse, parse_qs


class YouTubeTranscriptLoader:
    def __init__(self):
        self.formatter = TextFormatter()
        self.api = YouTubeTranscriptApi()
    
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
    
    def load_transcript(self, video_url_or_id: str, languages: List[str] = ['en']) -> Dict[str, any]:
        if video_url_or_id.startswith('http'):
            video_id = self.extract_video_id(video_url_or_id)
            if not video_id:
                raise ValueError(f"Could not extract video ID from URL: {video_url_or_id}")
        else:
            video_id = video_url_or_id
        
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
            
            return {
                'video_id': video_id,
                'language': transcript.language,
                'is_generated': transcript.is_generated,
                'raw_transcript': raw_transcript,
                'formatted_text': formatted_text,
                'cleaned_text': cleaned_text
            }
            
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