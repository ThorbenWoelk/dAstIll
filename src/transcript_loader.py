import re
import time
from pathlib import Path
from typing import Any
from urllib.parse import parse_qs, urlparse

from youtube_transcript_api import YouTubeTranscriptApi
from youtube_transcript_api.formatters import TextFormatter

from config.channel_config import ChannelConfigManager
from config.config import Config

from .file_manager import VideoFileManager
from .transcript_formatter import TranscriptFormatter


class RateLimitError(Exception):
    """Exception raised when rate limiting is detected."""

    pass


class YouTubeTranscriptLoader:
    def __init__(self, config_path: str = None):
        self.formatter = TextFormatter()
        self.api = YouTubeTranscriptApi()
        self.config = Config(config_path)
        self.channel_config = ChannelConfigManager()

        # Initialize file manager and formatter
        base_path = self.config.get("storage.base_path")
        self.manager = VideoFileManager(base_path)
        self.storage = TranscriptFormatter(base_path)

        # Track last API request time for rate limiting
        self._last_api_request_time = 0

    def _extract_video_id(self, url: str) -> str | None:
        """Extract video ID from YouTube URL."""
        parsed = urlparse(url)

        if parsed.hostname in ["youtube.com", "www.youtube.com"]:
            if parsed.path == "/watch":
                video_id = parse_qs(parsed.query).get("v")
                return video_id[0] if video_id else None
            elif parsed.path.startswith("/embed/"):
                return parsed.path.split("/")[2]
        elif parsed.hostname in ["youtu.be", "www.youtu.be"]:
            return parsed.path[1:]

        return None

    def _apply_rate_limiting(self, is_bulk_operation: bool = False):
        """Apply rate limiting before making API requests."""
        try:
            # Get rate limiting configuration
            rate_config = self.channel_config.global_config.rate_limiting
            if hasattr(rate_config, "api_request_delay"):
                api_delay = rate_config.api_request_delay
                bulk_delay = (
                    rate_config.bulk_operation_delay if is_bulk_operation else 0
                )
            else:
                # Fallback to default values if config doesn't exist
                api_delay = 2.0
                bulk_delay = 5.0 if is_bulk_operation else 0
        except (AttributeError, TypeError):
            # Fallback to default values if configuration is not available
            api_delay = 2.0
            bulk_delay = 5.0 if is_bulk_operation else 0

        # Calculate required delay
        required_delay = api_delay + bulk_delay
        current_time = time.time()
        time_since_last_request = current_time - self._last_api_request_time

        if time_since_last_request < required_delay:
            sleep_time = required_delay - time_since_last_request
            print(f"⏳ Rate limiting: sleeping for {sleep_time:.1f} seconds...")
            time.sleep(sleep_time)

        # Update last request time
        self._last_api_request_time = time.time()

    def load_transcript(
        self,
        video_url_or_id: str,
        languages: list[str] = None,
        force: bool = False,
        save_markdown: bool = True,
        channel: str = "unknown",
    ) -> dict[str, Any]:
        """Load transcript for a video."""
        if languages is None:
            languages = self.config.get("transcript.default_languages", ["en"])

        if video_url_or_id.startswith("http"):
            video_id = self._extract_video_id(video_url_or_id)
            if not video_id:
                raise ValueError(
                    f"Could not extract video ID from URL: {video_url_or_id}"
                )
        else:
            video_id = video_url_or_id

        # Check current status
        status, file_path = self.manager.get_video_status(video_id)

        # If already downloaded/processed and not forcing, return existing info
        if not force and status in ["downloaded", "processed"]:
            return {
                "video_id": video_id,
                "status": status,
                "already_exists": True,
                "file_path": file_path,
                "channel": self.manager._extract_channel_from_filename(
                    Path(file_path).name, video_id
                )
                if file_path
                else "unknown",
            }

        try:
            # Apply rate limiting before API request
            self._apply_rate_limiting()

            # Fetch transcript from YouTube API
            transcript_list = self.api.list(video_id)

            transcript = None
            for lang in languages:
                try:
                    transcript = transcript_list.find_transcript([lang])
                    break
                except Exception:
                    continue

            if not transcript:
                transcript = transcript_list.find_generated_transcript(languages)

            raw_transcript = transcript.fetch()
            formatted_text = self.formatter.format_transcript(raw_transcript)
            cleaned_text = self.clean_transcript(formatted_text)

            transcript_data = {
                "video_id": video_id,
                "language": transcript.language,
                "is_generated": transcript.is_generated,
                "raw_transcript": raw_transcript,
                "formatted_text": formatted_text,
                "cleaned_text": cleaned_text,
                "languages_requested": languages,
                "already_exists": False,
            }

            # Save as markdown if requested
            if save_markdown and self.config.get("storage.markdown_format", True):
                content = self.storage.format_transcript_content(transcript_data)
                file_path = self.manager.mark_downloaded(video_id, content, channel)
                transcript_data["file_path"] = file_path
                transcript_data["channel"] = channel

            return transcript_data

        except Exception as e:
            # Check if it's a rate limit error using better detection
            if self._is_rate_limit_error(e):
                raise RateLimitError(f"Rate limit hit: {str(e)}") from e
            raise Exception(f"Failed to load transcript: {str(e)}") from e

    def _is_rate_limit_error(self, exception: Exception) -> bool:
        """Detect if an exception is due to rate limiting."""
        error_msg = str(exception).lower()

        # Check for YouTube-specific rate limiting indicators
        youtube_rate_limit_indicators = [
            "too many requests",
            "quota exceeded",
            "rate limit",
            "blocked by youtube",
            "ip has been blocked",
            "requests from your ip",
            "cloud provider",
            "requestblocked",
            "ipblocked",
        ]

        # Check for HTTP status codes (in case they're mentioned in error messages)
        http_rate_limit_codes = ["429", "403", "400"]

        # Check for YouTube transcript API specific errors
        transcript_api_errors = [
            "could not retrieve a transcript",
            "youtube is blocking requests",
        ]

        return (
            any(indicator in error_msg for indicator in youtube_rate_limit_indicators)
            or any(code in error_msg for code in http_rate_limit_codes)
            or any(api_error in error_msg for api_error in transcript_api_errors)
        )

    def clean_transcript(self, text: str) -> str:
        """Clean transcript text by removing timestamps and music symbols."""
        text = re.sub(r"\[.*?\]", "", text)
        text = re.sub(r"♪+", "", text)
        text = re.sub(r"\s+", " ", text)
        text = text.strip()
        return text

    def save_transcript(self, transcript_data: dict[str, Any], filepath: str):
        """Save transcript to a custom file path."""
        with open(filepath, "w", encoding="utf-8") as f:
            f.write(f"Video ID: {transcript_data['video_id']}\n")
            f.write(f"Language: {transcript_data['language']}\n")
            f.write(f"Auto-generated: {transcript_data['is_generated']}\n")
            f.write("-" * 50 + "\n\n")
            f.write(transcript_data["cleaned_text"])

    def list_processed_videos(self):
        """List all videos across all statuses."""
        return self.manager.list_all_videos()

    def get_video_info(self, video_id: str):
        """Get information about a specific video."""
        status, file_path = self.manager.get_video_status(video_id)
        if status == "not_downloaded":
            return None

        channel = "unknown"
        if file_path:
            channel = self.manager._extract_channel_from_filename(
                Path(file_path).name, video_id
            )

        return {
            "video_id": video_id,
            "status": status,
            "file_path": file_path,
            "channel": channel,
        }

    def get_stats(self):
        """Get statistics about all videos."""
        return self.manager.get_stats()

    def remove_video(self, video_id: str, delete_file: bool = False):
        """Remove a video from tracking and optionally delete file."""
        return self.manager.remove_video(video_id, delete_file)

    def add_to_be_downloaded(self, video_id: str, channel: str = "unknown"):
        """Add a video to the download queue."""
        return self.manager.add_to_be_downloaded(video_id, channel)

    def process_video(self, video_id: str, channel: str = None):
        """Move a video from downloaded to processed status."""
        status, _ = self.manager.get_video_status(video_id)

        if status != "downloaded":
            return (
                False,
                f"Video {video_id} is not in 'downloaded' status (current: {status})",
            )

        # If no channel specified, try to extract from current filename
        if channel is None:
            _, current_path = self.manager.get_video_status(video_id)
            if current_path:
                channel = self.manager._extract_channel_from_filename(
                    Path(current_path).name, video_id
                )
            else:
                channel = "unknown"

        new_path = self.manager.mark_processed(video_id, channel)
        if new_path:
            return True, new_path
        else:
            return False, f"Failed to process {video_id}"
