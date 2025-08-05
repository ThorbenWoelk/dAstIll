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
        # Set comprehensive headers to avoid being blocked
        self.session.headers.update(
            {
                "User-Agent": "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36",
                "Accept": "text/html,application/xhtml+xml,application/xml;q=0.9,image/avif,image/webp,*/*;q=0.8",
                "Accept-Language": "en-US,en;q=0.5",
                "Accept-Encoding": "gzip, deflate, br",
                "DNT": "1",
                "Connection": "keep-alive",
                "Upgrade-Insecure-Requests": "1",
            }
        )

    def _request_with_backoff(
        self, url: str, max_retries: int = 3, initial_delay: float = 1.0
    ) -> requests.Response | None:
        """Make HTTP request with exponential backoff on failure."""
        for attempt in range(max_retries):
            try:
                response = self.session.get(url, timeout=10)
                if response.status_code == 200:
                    return response
                elif response.status_code in [
                    429,
                    500,
                    502,
                    503,
                    504,
                ]:  # Retry on server errors
                    if attempt < max_retries - 1:  # Don't sleep on last attempt
                        delay = initial_delay * (2**attempt)  # Exponential backoff
                        time.sleep(delay)
                        continue
                return response  # Return non-retryable response
            except (requests.RequestException, requests.Timeout):
                if attempt < max_retries - 1:  # Don't sleep on last attempt
                    delay = initial_delay * (2**attempt)  # Exponential backoff
                    time.sleep(delay)
                    continue
                return None  # All retries failed
        return None

    def get_latest_videos(self, channel_id: str, limit: int = 10) -> list[VideoInfo]:
        """Get latest videos from a channel using RSS feed."""
        try:
            rss_url = (
                f"https://www.youtube.com/feeds/videos.xml?channel_id={channel_id}"
            )
            response = self._request_with_backoff(rss_url)

            if not response or response.status_code != 200:
                # HTTP error or network failure - return empty list and let caller handle
                return []

            root = ET.fromstring(response.content)
            entries = root.findall("{http://www.w3.org/2005/Atom}entry")

            videos = []
            for entry in entries[:limit]:
                try:
                    video_id = entry.find(
                        "{http://www.youtube.com/xml/schemas/2015}videoId"
                    ).text
                    title = entry.find("{http://www.w3.org/2005/Atom}title").text
                    published = entry.find(
                        "{http://www.w3.org/2005/Atom}published"
                    ).text
                    channel_name = entry.find(
                        "{http://www.w3.org/2005/Atom}author/{http://www.w3.org/2005/Atom}name"
                    ).text

                    videos.append(
                        VideoInfo(
                            video_id=video_id,
                            title=title,
                            published=published,
                            channel_name=channel_name,
                            channel_id=channel_id,
                            url=f"https://www.youtube.com/watch?v={video_id}",
                        )
                    )
                except AttributeError:
                    # Skip malformed entries and continue
                    continue

            return videos

        except Exception:
            # Network or parsing error - return empty list
            return []

    def test_rss_feed(self, channel_id: str) -> bool:
        """Test if RSS feed is accessible for a channel."""
        try:
            rss_url = (
                f"https://www.youtube.com/feeds/videos.xml?channel_id={channel_id}"
            )
            response = self._request_with_backoff(
                rss_url, max_retries=2, initial_delay=0.5
            )
            return response is not None and response.status_code == 200
        except Exception:
            return False

    def verify_channel_exists(self, handle: str) -> dict[str, Any]:
        """Verify that a channel exists and return basic info."""
        try:
            clean_handle = handle.replace("@", "")
            channel_url = f"https://www.youtube.com/@{clean_handle}"

            response = self.session.get(channel_url, timeout=10)
            if response.status_code != 200:
                return {"exists": False, "error": f"HTTP {response.status_code}"}

            content = response.text

            # Extract channel name from page title or meta tags
            title_match = re.search(r"<title>([^<]+)</title>", content)
            name = title_match.group(1).strip() if title_match else clean_handle

            # Remove " - YouTube" suffix if present
            name = re.sub(r"\s*-\s*YouTube$", "", name)

            return {"exists": True, "name": name, "handle": handle, "url": channel_url}

        except Exception as e:
            return {"exists": False, "error": str(e)}

    def resolve_channel_id_from_handle(self, handle: str) -> str | None:
        """Resolve YouTube channel ID from handle without API key.

        Args:
            handle: YouTube channel handle (e.g., '@username' or 'username')

        Returns:
            Channel ID if found, None otherwise
        """
        # Ensure handle starts with @
        if not handle.startswith("@"):
            handle = f"@{handle}"

        # Construct channel URL
        channel_url = f"https://www.youtube.com/{handle}"

        try:
            response = self._request_with_backoff(channel_url)
            if not response or response.status_code != 200:
                return None

            content = response.text

            # Try multiple patterns to extract channel ID
            # Pattern 1: meta tag with itemprop="channelId"
            channel_id_match = re.search(
                r'<meta\s+itemprop="channelId"\s+content="([^"]+)"', content
            )
            if channel_id_match:
                return channel_id_match.group(1)

            # Pattern 2: externalId in JSON-LD or page data
            external_id_match = re.search(r'"externalId"\s*:\s*"([^"]+)"', content)
            if external_id_match:
                return external_id_match.group(1)

            # Pattern 3: browseId in ytInitialData
            browse_id_match = re.search(r'"browseId"\s*:\s*"(UC[^"]+)"', content)
            if browse_id_match:
                return browse_id_match.group(1)

            # Pattern 4: channelId in various contexts
            channel_id_alt_match = re.search(r'"channelId"\s*:\s*"(UC[^"]+)"', content)
            if channel_id_alt_match:
                return channel_id_alt_match.group(1)

            return None

        except Exception:
            return None
