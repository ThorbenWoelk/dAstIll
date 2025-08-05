"""RSS-based YouTube channel monitoring without API keys."""

import logging
import re
import time
import xml.etree.ElementTree as ET
from dataclasses import dataclass
from typing import Any

import requests

# Set up logger for pattern success tracking
logger = logging.getLogger(__name__)


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

    def get_latest_videos(self, channel_id: str, limit: int = None) -> list[VideoInfo]:
        """Get latest videos from a channel using RSS feed. If limit is None, returns all available videos."""
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
            # If no limit specified, process all entries; otherwise slice to limit
            entries_to_process = entries if limit is None else entries[:limit]
            for entry in entries_to_process:
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
        # Input validation - ensure handle is safe
        if not handle or not isinstance(handle, str):
            return None

        # Remove any leading/trailing whitespace
        handle = handle.strip()

        # Validate handle format - only allow alphanumeric, underscore, dash, and @
        # This prevents URL injection and other malicious inputs
        import string

        allowed_chars = string.ascii_letters + string.digits + "_-@"
        if not all(c in allowed_chars for c in handle):
            return None

        # Ensure handle starts with @
        if not handle.startswith("@"):
            handle = f"@{handle}"

        # Additional validation: handle should have reasonable length
        if len(handle) < 2 or len(handle) > 50:  # @ + at least 1 char, max 50 total
            return None

        # Construct channel URL with validated handle
        channel_url = f"https://www.youtube.com/{handle}"

        try:
            response = self._request_with_backoff(channel_url)
            if not response or response.status_code != 200:
                return None

            content = response.text

            # Try multiple patterns to extract channel ID
            patterns = [
                (
                    "meta_tag",
                    r'<meta\s+itemprop="channelId"\s+content="([^"]+)"',
                    "meta tag with itemprop='channelId'",
                ),
                (
                    "external_id",
                    r'"externalId"\s*:\s*"([^"]+)"',
                    "externalId in JSON data",
                ),
                (
                    "browse_id",
                    r'"browseId"\s*:\s*"(UC[^"]+)"',
                    "browseId in ytInitialData",
                ),
                (
                    "channel_id_alt",
                    r'"channelId"\s*:\s*"(UC[^"]+)"',
                    "channelId in various contexts",
                ),
            ]

            for pattern_name, pattern, description in patterns:
                match = re.search(pattern, content)
                if match:
                    channel_id = match.group(1)
                    logger.debug(
                        f"Channel ID resolved using {pattern_name} pattern ({description}) for handle {handle}: {channel_id}"
                    )
                    return channel_id

            # Log pattern failure for monitoring
            logger.warning(
                f"All channel ID patterns failed for handle {handle}. YouTube may have changed their HTML structure."
            )
            return None

        except Exception:
            return None
