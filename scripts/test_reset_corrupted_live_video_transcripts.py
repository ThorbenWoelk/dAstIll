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


if __name__ == "__main__":
    unittest.main()
