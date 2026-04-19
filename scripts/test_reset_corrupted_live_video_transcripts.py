#!/usr/bin/env python3

import importlib.util
import unittest
from pathlib import Path


SCRIPT_PATH = Path(__file__).with_name("reset_corrupted_live_video_transcripts.py")
SPEC = importlib.util.spec_from_file_location("live_cleanup", SCRIPT_PATH)
assert SPEC and SPEC.loader
live_cleanup = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(live_cleanup)


class DuplicateLivestreamDetectionTest(unittest.TestCase):
    def test_flags_tiny_duplicate_transcript(self) -> None:
        duplicate_key = (
            "channel-1",
            "2026-04-17T20:11:50Z",
            "2026-04-18T00:56:19Z",
            "PT4H44M33S",
        )

        corrupted = live_cleanup.find_duplicate_tiny_transcripts(
            [
                {
                    "id": "small",
                    "title": "Small duplicate",
                    "duplicate_key": duplicate_key,
                    "word_count": 463,
                },
                {
                    "id": "large",
                    "title": "Large duplicate",
                    "duplicate_key": (
                        "channel-1",
                        "2026-04-17T20:11:51Z",
                        "2026-04-18T00:56:19Z",
                        "PT4H44M33S",
                    ),
                    "word_count": 39_469,
                },
            ]
        )

        self.assertEqual([entry["id"] for entry in corrupted], ["small"])
        self.assertEqual(corrupted[0]["reason"], "tiny_duplicate")
        self.assertEqual(corrupted[0]["duplicate_best_ids"], ["large"])

    def test_ignores_small_transcripts_without_better_duplicate(self) -> None:
        duplicate_key = (
            "channel-1",
            "2026-04-17T20:11:50Z",
            "2026-04-18T00:56:19Z",
            "PT4H44M33S",
        )

        corrupted = live_cleanup.find_duplicate_tiny_transcripts(
            [
                {
                    "id": "small-a",
                    "title": "Small duplicate A",
                    "duplicate_key": duplicate_key,
                    "word_count": 463,
                },
                {
                    "id": "small-b",
                    "title": "Small duplicate B",
                    "duplicate_key": duplicate_key,
                    "word_count": 700,
                },
            ]
        )

        self.assertEqual(corrupted, [])

    def test_keeps_different_completed_streams_separate(self) -> None:
        corrupted = live_cleanup.find_duplicate_tiny_transcripts(
            [
                {
                    "id": "small",
                    "title": "Small stream",
                    "duplicate_key": (
                        "channel-1",
                        "2026-04-17T20:11:50Z",
                        "2026-04-18T00:56:19Z",
                        "PT4H44M33S",
                    ),
                    "word_count": 463,
                },
                {
                    "id": "large",
                    "title": "Different stream",
                    "duplicate_key": (
                        "channel-1",
                        "2026-04-18T20:11:50Z",
                        "2026-04-19T00:56:19Z",
                        "PT4H44M33S",
                    ),
                    "word_count": 39_469,
                },
            ]
        )

        self.assertEqual(corrupted, [])

    def test_detects_description_like_transcript_for_long_stream(self) -> None:
        description = (
            "Yesterday I tested Claude Design against v0, Grok, Google Stitch, "
            "Cursor, Droid, and ChatGPT Pro on the same blog redesign. Claude "
            "Design produced the visual system. Droid turned it into a working "
            "prototype with Opus on max thinking. It looked great locally. "
            "Today I am closing the loop and getting my blog live on the internet."
        )

        self.assertTrue(
            live_cleanup.transcript_looks_like_description(
                {"raw_text": description, "timed_text": None},
                description,
                "PT3H28M39S",
            )
        )

    def test_description_like_detector_accepts_real_long_transcript(self) -> None:
        description = "Today I am getting my blog live on the internet."
        transcript = " ".join(f"caption{index}" for index in range(1_500))

        self.assertFalse(
            live_cleanup.transcript_looks_like_description(
                {"raw_text": transcript, "timed_text": None},
                description,
                "PT3H28M39S",
            )
        )


if __name__ == "__main__":
    unittest.main()
