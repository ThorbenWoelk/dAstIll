"""RSS-based YouTube channel monitoring without API keys."""

import re
import time
import xml.etree.ElementTree as ET
from dataclasses import dataclass
from typing import Any

import requests


@dataclass
class VideoInfo:
    """Data structure for video information from RSS feed."""
    video_id: str
    title: str
    published: str
    channel_name: str
    channel_id: str
    url: str

    def __post_init__(self):
        """Ensure URL is properly formatted."""
        if not self.url:
            self.url = f"https://www.youtube.com/watch?v={self.video_id}"


class RSSChannelMonitor:
    """RSS-based channel monitoring - no API key required."""

    def __init__(self):
        self.session = requests.Session()
        # Set a reasonable user agent to avoid being blocked
        self.session.headers.update({
            'User-Agent': 'Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/91.0.4472.124 Safari/537.36'
        })

    def _request_with_backoff(self, url: str, max_retries: int = 3, initial_delay: float = 1.0) -> requests.Response | None:
        """Make HTTP request with exponential backoff on failure."""
        for attempt in range(max_retries):
            try:
                response = self.session.get(url, timeout=10)
                if response.status_code == 200:
                    return response
                elif response.status_code in [429, 500, 502, 503, 504]:  # Retry on server errors
                    if attempt < max_retries - 1:  # Don't sleep on last attempt
                        delay = initial_delay * (2 ** attempt)  # Exponential backoff
                        time.sleep(delay)
                        continue
                return response  # Return non-retryable response
            except (requests.RequestException, requests.Timeout):
                if attempt < max_retries - 1:  # Don't sleep on last attempt
                    delay = initial_delay * (2 ** attempt)  # Exponential backoff
                    time.sleep(delay)
                    continue
                return None  # All retries failed
        return None

    def get_latest_videos(self, channel_id: str, limit: int = 10) -> list[VideoInfo]:
        """Get latest videos from a channel using RSS feed."""
        try:
            rss_url = f"https://www.youtube.com/feeds/videos.xml?channel_id={channel_id}"
            response = self._request_with_backoff(rss_url)

            if not response or response.status_code != 200:
                # HTTP error or network failure - return empty list and let caller handle
                return []

            root = ET.fromstring(response.content)
            entries = root.findall('{http://www.w3.org/2005/Atom}entry')

            videos = []
            for entry in entries[:limit]:
                try:
                    video_id = entry.find('{http://www.youtube.com/xml/schemas/2015}videoId').text
                    title = entry.find('{http://www.w3.org/2005/Atom}title').text
                    published = entry.find('{http://www.w3.org/2005/Atom}published').text
                    channel_name = entry.find('{http://www.w3.org/2005/Atom}author/{http://www.w3.org/2005/Atom}name').text

                    videos.append(VideoInfo(
                        video_id=video_id,
                        title=title,
                        published=published,
                        channel_name=channel_name,
                        channel_id=channel_id,
                        url=f"https://www.youtube.com/watch?v={video_id}"
                    ))
                except AttributeError as e:
                    # Skip malformed entries and continue
                    continue

            return videos

        except Exception as e:
            # Network or parsing error - return empty list
            return []

    def resolve_channel_id(self, handle: str) -> str | None:
        """Extract channel ID from channel page by scraping."""
        try:
            # Clean the handle
            clean_handle = handle.replace('@', '')
            
            # Try different URL formats with exponential backoff
            url_patterns = [
                f"https://www.youtube.com/@{clean_handle}",
                f"https://www.youtube.com/c/{clean_handle}",
                f"https://www.youtube.com/user/{clean_handle}"
            ]
            
            response = None
            for url in url_patterns:
                response = self._request_with_backoff(url)
                if response and response.status_code == 200:
                    break
            
            if not response or response.status_code != 200:
                return None

            content = response.text

            # Multiple patterns to find channel ID
            patterns = [
                r'"channelId":"(UC[a-zA-Z0-9_-]+)"',
                r'"externalId":"(UC[a-zA-Z0-9_-]+)"',
                r'<meta property="og:url" content="https://www\.youtube\.com/channel/(UC[a-zA-Z0-9_-]+)"',
                r'youtube\.com/channel/(UC[a-zA-Z0-9_-]+)',
                r'"browseId":"(UC[a-zA-Z0-9_-]+)"',
                r'channel/(UC[a-zA-Z0-9_-]+)',
                r'"canonicalChannelUrl":"[^"]*/(UC[a-zA-Z0-9_-]+)"'
            ]

            for pattern in patterns:
                match = re.search(pattern, content)
                if match:
                    channel_id = match.group(1)
                    return channel_id

            # Could not find channel ID - return None for caller to handle
            return None

        except Exception as e:
            # Network or parsing error - return None for caller to handle
            return None

    def test_rss_feed(self, channel_id: str) -> bool:
        """Test if RSS feed is accessible for a channel."""
        try:
            rss_url = f"https://www.youtube.com/feeds/videos.xml?channel_id={channel_id}"
            response = self._request_with_backoff(rss_url, max_retries=2, initial_delay=0.5)
            return response is not None and response.status_code == 200
        except Exception:
            return False

    def verify_channel_exists(self, handle: str) -> dict[str, Any]:
        """Verify that a channel exists and return basic info."""
        try:
            clean_handle = handle.replace('@', '')
            channel_url = f"https://www.youtube.com/@{clean_handle}"

            response = self.session.get(channel_url, timeout=10)
            if response.status_code != 200:
                return {"exists": False, "error": f"HTTP {response.status_code}"}

            content = response.text

            # Extract channel name from page title or meta tags
            title_match = re.search(r'<title>([^<]+)</title>', content)
            name = title_match.group(1).strip() if title_match else clean_handle

            # Remove " - YouTube" suffix if present
            name = re.sub(r'\s*-\s*YouTube$', '', name)

            return {
                "exists": True,
                "name": name,
                "handle": handle,
                "url": channel_url
            }

        except Exception as e:
            return {"exists": False, "error": str(e)}
