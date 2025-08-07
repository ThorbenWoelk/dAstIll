"""Browser-based transcript extraction for YouTube videos.

This module provides a fallback method for extracting transcripts when the
YouTube Transcript API is rate limited or unavailable. It uses Playwright
to interact with YouTube's web interface and extract transcript data.
"""

import re
import time
from typing import Any

try:
    from playwright.sync_api import TimeoutError as PlaywrightTimeoutError
    from playwright.sync_api import sync_playwright

    PLAYWRIGHT_AVAILABLE = True
except ImportError:
    PLAYWRIGHT_AVAILABLE = False
    sync_playwright = None
    PlaywrightTimeoutError = Exception


class BrowserTranscriptError(Exception):
    """Exception raised when browser-based transcript extraction fails."""

    pass


class BrowserTranscriptExtractor:
    """Extracts transcripts from YouTube using browser automation."""

    def __init__(self, headless: bool = True, timeout: int = 30000):
        """Initialize the browser transcript extractor.

        Args:
            headless: Whether to run browser in headless mode
            timeout: Timeout in milliseconds for page operations
        """
        if not PLAYWRIGHT_AVAILABLE:
            raise BrowserTranscriptError(
                "Playwright is required for browser-based transcript extraction. "
                "Install it with: uv add playwright && uv run playwright install chromium"
            )

        self.headless = headless
        self.timeout = timeout
        self._last_request_time = 0

    def _apply_rate_limiting(self, delay: float = 3.0):
        """Apply rate limiting to avoid overwhelming YouTube."""
        current_time = time.time()
        time_since_last = current_time - self._last_request_time

        if time_since_last < delay:
            sleep_time = delay - time_since_last
            print(f"⏳ Browser rate limiting: sleeping for {sleep_time:.1f} seconds...")
            time.sleep(sleep_time)

        self._last_request_time = time.time()

    def extract_transcript(
        self, video_id: str, languages: list[str] = None
    ) -> dict[str, Any]:
        """Extract transcript from YouTube using browser automation.

        Args:
            video_id: YouTube video ID
            languages: Preferred languages for transcript (e.g., ['en', 'de'])

        Returns:
            Dictionary containing transcript data similar to API format

        Raises:
            BrowserTranscriptError: When transcript extraction fails
        """
        if languages is None:
            languages = ["en"]

        # Apply rate limiting
        self._apply_rate_limiting()

        with sync_playwright() as p:
            try:
                browser = p.chromium.launch(headless=self.headless)
                context = browser.new_context(
                    user_agent="Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36"
                )
                page = context.new_page()

                # Navigate to YouTube video
                url = f"https://www.youtube.com/watch?v={video_id}"
                page.goto(url, timeout=self.timeout)

                # Wait for page to load
                page.wait_for_load_state("networkidle", timeout=self.timeout)

                # Check if video exists and is available
                if self._check_video_unavailable(page):
                    raise BrowserTranscriptError(
                        f"Video {video_id} is unavailable or private"
                    )

                # Try to find and click transcript button
                transcript_data = self._extract_transcript_data(
                    page, video_id, languages
                )

                return transcript_data

            except PlaywrightTimeoutError as e:
                raise BrowserTranscriptError(
                    f"Timeout while loading video {video_id}: {str(e)}"
                ) from e
            except Exception as e:
                raise BrowserTranscriptError(
                    f"Failed to extract transcript for {video_id}: {str(e)}"
                ) from e
            finally:
                try:
                    browser.close()
                except Exception:
                    pass  # Ignore cleanup errors

    def _check_video_unavailable(self, page) -> bool:
        """Check if video is unavailable, private, or removed."""
        unavailable_indicators = [
            "This video is unavailable",
            "Video unavailable",
            "This video is private",
            "This video has been removed",
        ]

        try:
            page_content = page.content()
            return any(
                indicator in page_content for indicator in unavailable_indicators
            )
        except Exception:
            return False

    def _extract_transcript_data(
        self, page, video_id: str, languages: list[str]
    ) -> dict[str, Any]:
        """Extract transcript data from the YouTube page."""
        try:
            # Look for transcript/captions button
            # YouTube uses various selectors for the transcript button
            transcript_selectors = [
                'button[aria-label*="transcript"]',
                'button[aria-label*="Transcript"]',
                'button[aria-label*="captions"]',
                'button[aria-label*="Captions"]',
                '[data-target-id="engagement-panel-transcript"]',
            ]

            transcript_button = None
            for selector in transcript_selectors:
                try:
                    transcript_button = page.wait_for_selector(selector, timeout=5000)
                    if transcript_button:
                        print(f"Found transcript button with selector: {selector}")
                        break
                except Exception as e:
                    print(f"Selector '{selector}' failed: {e}")
                    continue

            if not transcript_button:
                # Debug: print available buttons for analysis
                print("Debugging available buttons:")
                all_buttons = page.query_selector_all("button")
                for i, button in enumerate(all_buttons[:10]):  # Show first 10 buttons
                    try:
                        aria_label = button.get_attribute("aria-label")
                        text = button.inner_text()[:50]  # First 50 chars
                        print(f"  Button {i}: aria-label='{aria_label}', text='{text}'")
                    except Exception:
                        pass
                raise BrowserTranscriptError("No transcript button found")

            # Click transcript button
            transcript_button.click()

            # Wait for transcript panel to load
            page.wait_for_selector(
                '[data-target-id="engagement-panel-transcript"]', timeout=10000
            )

            # Extract transcript content
            transcript_items = self._extract_transcript_items(page)

            if not transcript_items:
                raise BrowserTranscriptError("No transcript content found")

            # Format transcript data to match API structure
            formatted_text = self._format_transcript_items(transcript_items)
            cleaned_text = self._clean_transcript_text(formatted_text)

            return {
                "video_id": video_id,
                "language": languages[0]
                if languages
                else "en",  # Default to first requested language
                "is_generated": True,  # Browser extraction typically gets auto-generated transcripts
                "raw_transcript": transcript_items,
                "formatted_text": formatted_text,
                "cleaned_text": cleaned_text,
                "languages_requested": languages,
                "extraction_method": "browser",
                "already_exists": False,
            }

        except Exception as e:
            raise BrowserTranscriptError(
                f"Failed to extract transcript content: {str(e)}"
            ) from e

    def _extract_transcript_items(self, page) -> list[dict[str, Any]]:
        """Extract individual transcript items with timestamps."""
        transcript_items = []

        try:
            # Wait for transcript content to load
            page.wait_for_selector(".ytd-transcript-segment-renderer", timeout=10000)

            # Extract transcript segments
            segments = page.query_selector_all(".ytd-transcript-segment-renderer")

            for segment in segments:
                try:
                    # Extract timestamp
                    timestamp_element = segment.query_selector(
                        ".ytd-transcript-segment-renderer .segment-timestamp"
                    )
                    timestamp = (
                        timestamp_element.inner_text().strip()
                        if timestamp_element
                        else ""
                    )

                    # Extract text content
                    text_element = segment.query_selector(
                        ".ytd-transcript-segment-renderer .segment-text"
                    )
                    text = text_element.inner_text().strip() if text_element else ""

                    if text:  # Only add non-empty text segments
                        # Convert timestamp to seconds (rough approximation)
                        start_time = self._parse_timestamp(timestamp)

                        transcript_items.append(
                            {
                                "text": text,
                                "start": start_time,
                                "duration": 2.0,  # Default duration, YouTube API doesn't always provide this
                            }
                        )

                except Exception:
                    continue  # Skip problematic segments

            return transcript_items

        except Exception as e:
            # Fallback: try to extract any text content from transcript area
            try:
                transcript_container = page.query_selector("#transcript")
                if transcript_container:
                    text_content = transcript_container.inner_text()
                    # Split into approximate segments
                    sentences = re.split(r"[.!?]+", text_content)
                    return [
                        {"text": sentence.strip(), "start": i * 3.0, "duration": 3.0}
                        for i, sentence in enumerate(sentences)
                        if sentence.strip()
                    ]
            except Exception:
                pass

            raise BrowserTranscriptError(
                f"Could not extract transcript items: {str(e)}"
            ) from e

    def _parse_timestamp(self, timestamp_str: str) -> float:
        """Parse timestamp string (e.g., '1:23') to seconds."""
        if not timestamp_str:
            return 0.0

        try:
            parts = timestamp_str.split(":")
            if len(parts) == 2:  # MM:SS
                minutes, seconds = parts
                return float(minutes) * 60 + float(seconds)
            elif len(parts) == 3:  # HH:MM:SS
                hours, minutes, seconds = parts
                return float(hours) * 3600 + float(minutes) * 60 + float(seconds)
            else:
                return 0.0
        except (ValueError, IndexError):
            return 0.0

    def _format_transcript_items(self, transcript_items: list[dict[str, Any]]) -> str:
        """Format transcript items into a readable text format."""
        return " ".join(item["text"] for item in transcript_items if item.get("text"))

    def _clean_transcript_text(self, text: str) -> str:
        """Clean transcript text by removing artifacts and normalizing whitespace."""
        # Remove common YouTube transcript artifacts
        text = re.sub(r"\[.*?\]", "", text)  # Remove [Music], [Applause], etc.
        text = re.sub(r"♪+", "", text)  # Remove music symbols
        text = re.sub(r"\s+", " ", text)  # Normalize whitespace
        text = text.strip()

        return text

    def is_available(self) -> bool:
        """Check if browser extraction is available (Playwright installed)."""
        return PLAYWRIGHT_AVAILABLE
