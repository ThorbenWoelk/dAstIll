#!/usr/bin/env python3
"""Find completed livestreams with corrupted transcript artifacts.

The cleanup rules are intentionally conservative:
- YouTube must report liveStreamingDetails.actualEndTime.
- A transcript object must exist.
- A transcript is stale when its LastModified is before actualEndTime.
- A transcript is description-like when a long completed stream's transcript is
  short, untimed, and mostly overlaps the YouTube description.
- A transcript is a tiny duplicate when another video from the same channel has
  the same actual start/end/duration and a much larger transcript.

Use --apply to delete stale transcript/summary/search artifacts. Stale
before-end transcripts are reset to pending so the queue rebuilds them from the
completed VOD. Tiny duplicates are quarantined with retries exhausted, because
rebuilding that alternate duplicate can keep returning the same tiny transcript.
"""

from __future__ import annotations

import argparse
import json
import os
import re
import sqlite3
import subprocess
import sys
import tempfile
import urllib.parse
from datetime import datetime, timezone
from pathlib import Path


REPO_ROOT = Path(__file__).resolve().parents[1]
BACKEND_ENV = REPO_ROOT / "backend" / ".env"
DEFAULT_BATCH_SIZE = 50
MIN_DUPLICATE_GOOD_WORDS = 1_000
DUPLICATE_TINY_RATIO = 0.10
STREAM_TIME_MATCH_TOLERANCE_SECONDS = 5
MAX_DISTILLATION_RETRIES = 3
LONG_LIVE_MIN_DURATION_SECONDS = 30 * 60
DESCRIPTION_LIKE_MAX_TRANSCRIPT_WORDS = 1_000
DESCRIPTION_LIKE_MIN_WORDS = 40
DESCRIPTION_LIKE_OVERLAP_RATIO = 0.75


def transcript_word_count(payload: dict) -> int:
    text = payload.get("raw_text") or payload.get("formatted_markdown") or ""
    return len(text.split())


def normalized_word_tokens(text: str) -> list[str]:
    return [token.lower() for token in re.split(r"[^0-9A-Za-z]+", text) if token]


def token_overlap_ratio(needle_tokens: list[str], haystack_tokens: list[str]) -> float:
    if not needle_tokens:
        return 0.0
    counts: dict[str, int] = {}
    for token in haystack_tokens:
        counts[token] = counts.get(token, 0) + 1

    matches = 0
    for token in needle_tokens:
        count = counts.get(token, 0)
        if count:
            counts[token] = count - 1
            matches += 1
    return matches / len(needle_tokens)


def parse_iso8601_duration_seconds(value: str | None) -> int | None:
    if not value or not value.startswith("P"):
        return None
    match = re.fullmatch(
        r"P(?:(?P<days>\d+)D)?(?:T(?:(?P<hours>\d+)H)?(?:(?P<minutes>\d+)M)?(?:(?P<seconds>\d+)S)?)?",
        value,
    )
    if not match:
        return None
    total = (
        int(match.group("days") or 0) * 86_400
        + int(match.group("hours") or 0) * 3_600
        + int(match.group("minutes") or 0) * 60
        + int(match.group("seconds") or 0)
    )
    return total or None


def transcript_looks_like_description(
    payload: dict,
    description: str,
    duration_iso8601: str | None,
) -> bool:
    if payload.get("timed_text"):
        return False

    duration_seconds = parse_iso8601_duration_seconds(duration_iso8601)
    if duration_seconds is None or duration_seconds < LONG_LIVE_MIN_DURATION_SECONDS:
        return False

    text = payload.get("raw_text") or payload.get("formatted_markdown") or ""
    transcript_tokens = normalized_word_tokens(text)
    if (
        len(transcript_tokens) < DESCRIPTION_LIKE_MIN_WORDS
        or len(transcript_tokens) > DESCRIPTION_LIKE_MAX_TRANSCRIPT_WORDS
    ):
        return False

    description_tokens = normalized_word_tokens(description)
    if len(description_tokens) < DESCRIPTION_LIKE_MIN_WORDS:
        return False

    return (
        token_overlap_ratio(transcript_tokens, description_tokens)
        >= DESCRIPTION_LIKE_OVERLAP_RATIO
    )


def load_env_file(path: Path) -> None:
    if not path.exists():
        return
    for raw_line in path.read_text().splitlines():
        line = raw_line.strip()
        if not line or line.startswith("#") or "=" not in line:
            continue
        key, value = line.split("=", 1)
        key = key.strip()
        value = value.strip().strip('"').strip("'")
        os.environ.setdefault(key, value)


def run(args: list[str], *, capture: bool = True, check: bool = True) -> subprocess.CompletedProcess[str]:
    return subprocess.run(
        args,
        text=True,
        capture_output=capture,
        check=check,
    )


def find_default_db() -> Path:
    explicit_dir = os.environ.get("DASTILL_LIBSQL_DIR")
    if explicit_dir:
        candidate = Path(explicit_dir) / "search-fts.db"
        if candidate.exists():
            return candidate

    matches = list(Path(tempfile.gettempdir()).glob("dastill-search-index-*/search-fts.db"))
    if not matches:
        raise SystemExit("No local search-fts.db found. Pass --db PATH.")
    return max(matches, key=lambda path: path.stat().st_mtime)


def parse_dt(value: str) -> datetime:
    normalized = value.replace("Z", "+00:00")
    parsed = datetime.fromisoformat(normalized)
    if parsed.tzinfo is None:
        parsed = parsed.replace(tzinfo=timezone.utc)
    return parsed.astimezone(timezone.utc)


def fetch_youtube_items(video_ids: list[str]) -> list[dict]:
    api_key = os.environ.get("YOUTUBE_API_KEY")
    if not api_key:
        raise SystemExit("YOUTUBE_API_KEY is required")

    query = urllib.parse.urlencode(
        {
            "part": "snippet,contentDetails,liveStreamingDetails",
            "id": ",".join(video_ids),
            "maxResults": "50",
            "key": api_key,
        }
    )
    url = f"https://www.googleapis.com/youtube/v3/videos?{query}"
    result = run(["curl", "-fsS", url])
    payload = json.loads(result.stdout)
    return payload.get("items", [])


def aws_json(args: list[str], *, default=None):
    env = os.environ.copy()
    result = run(["aws", *args, "--output", "json"], check=False)
    if result.returncode != 0:
        if default is not None:
            return default
        sys.stderr.write(result.stderr)
        raise SystemExit(result.returncode)
    if not result.stdout.strip():
        return default
    return json.loads(result.stdout)


def head_s3_key(bucket: str, key: str) -> dict | None:
    result = run(
        ["aws", "s3api", "head-object", "--bucket", bucket, "--key", key, "--output", "json"],
        check=False,
    )
    if result.returncode != 0:
        if "Not Found" in result.stderr or "404" in result.stderr or "NoSuchKey" in result.stderr:
            return None
        sys.stderr.write(result.stderr)
        raise SystemExit(result.returncode)
    return json.loads(result.stdout)


def get_s3_json(bucket: str, key: str) -> dict | None:
    result = run(["aws", "s3", "cp", f"s3://{bucket}/{key}", "-"], check=False)
    if result.returncode != 0 or not result.stdout.strip():
        return None
    return json.loads(result.stdout)


def list_s3_keys(bucket: str, prefix: str) -> list[str]:
    payload = aws_json(
        [
            "s3api",
            "list-objects-v2",
            "--bucket",
            bucket,
            "--prefix",
            prefix,
        ],
        default={},
    )
    return [entry["Key"] for entry in payload.get("Contents", []) if entry.get("Key")]


def delete_s3_key(bucket: str, key: str) -> None:
    run(["aws", "s3api", "delete-object", "--bucket", bucket, "--key", key], capture=True)


def delete_vectors(vector_bucket: str, vector_index: str, keys: list[str]) -> None:
    for start in range(0, len(keys), 500):
        batch = keys[start : start + 500]
        if not batch:
            continue
        run(
            [
                "aws",
                "s3vectors",
                "delete-vectors",
                "--vector-bucket-name",
                vector_bucket,
                "--index-name",
                vector_index,
                "--keys",
                *batch,
            ],
            capture=True,
            check=False,
        )


def load_video_rows(db_path: Path, limit_ids: set[str] | None) -> list[tuple[str, str]]:
    with sqlite3.connect(db_path) as conn:
        if limit_ids:
            placeholders = ",".join("?" for _ in limit_ids)
            return conn.execute(
                f"SELECT id, title FROM videos WHERE id IN ({placeholders}) ORDER BY published_at DESC",
                tuple(sorted(limit_ids)),
            ).fetchall()
        return conn.execute("SELECT id, title FROM videos ORDER BY published_at DESC").fetchall()


def update_local_rows(
    db_path: Path,
    reset_ids: list[str],
    quarantine_ids: list[str],
) -> None:
    if not reset_ids and not quarantine_ids:
        return
    with sqlite3.connect(db_path) as conn:
        all_ids = [*reset_ids, *quarantine_ids]
        placeholders = ",".join("?" for _ in all_ids)
        conn.execute(f"DELETE FROM fts_search WHERE video_id IN ({placeholders})", tuple(all_ids))
        if reset_ids:
            reset_placeholders = ",".join("?" for _ in reset_ids)
            conn.execute(
                f"""
                UPDATE videos
                   SET transcript_status = 'pending',
                       summary_status = 'pending',
                       retry_count = 0
                 WHERE id IN ({reset_placeholders})
                """,
                tuple(reset_ids),
            )
        if quarantine_ids:
            quarantine_placeholders = ",".join("?" for _ in quarantine_ids)
            conn.execute(
                f"""
                UPDATE videos
                   SET transcript_status = 'failed',
                       summary_status = 'pending',
                       retry_count = ?
                 WHERE id IN ({quarantine_placeholders})
                """,
                (MAX_DISTILLATION_RETRIES, *quarantine_ids),
            )
        conn.commit()


def update_video_snapshot(bucket: str, video_id: str, *, quarantine: bool) -> None:
    key = f"videos/{video_id}.json"
    result = run(["aws", "s3", "cp", f"s3://{bucket}/{key}", "-"], check=False)
    if result.returncode != 0 or not result.stdout.strip():
        return
    record = json.loads(result.stdout)
    record["transcript_status"] = "failed" if quarantine else "pending"
    record["summary_status"] = "pending"
    record["retry_count"] = MAX_DISTILLATION_RETRIES if quarantine else 0
    payload = json.dumps(record, separators=(",", ":")).encode()
    proc = subprocess.run(
        ["aws", "s3", "cp", "-", f"s3://{bucket}/{key}", "--content-type", "application/json"],
        input=payload,
        check=True,
    )


def collect_cleanup_keys(bucket: str, video_id: str) -> tuple[list[str], list[str]]:
    object_keys = [
        f"transcripts/{video_id}.json",
        f"summaries/{video_id}.json",
        f"video-info/{video_id}.json",
    ]
    object_keys.extend(list_s3_keys(bucket, f"search-sources/{video_id}/"))
    object_keys.extend(list_s3_keys(bucket, f"search-chunks/{video_id}_"))
    object_keys.extend(list_s3_keys(bucket, f"search-bundles/{video_id}_"))
    object_keys = sorted(set(object_keys))
    vector_keys = [
        key.removeprefix("search-chunks/").removesuffix(".json")
        for key in object_keys
        if key.startswith("search-chunks/")
    ]
    return object_keys, vector_keys


def completed_livestream_key(item: dict) -> tuple[str, str, str, str] | None:
    live_details = item.get("liveStreamingDetails", {})
    start_time = live_details.get("actualStartTime")
    end_time = live_details.get("actualEndTime")
    channel_id = item.get("snippet", {}).get("channelId")
    duration = item.get("contentDetails", {}).get("duration")
    if not channel_id or not start_time or not end_time or not duration:
        return None
    return (channel_id, start_time, end_time, duration)


def same_completed_livestream(left: dict, right: dict) -> bool:
    left_key = left.get("duplicate_key")
    right_key = right.get("duplicate_key")
    if left_key is None or right_key is None:
        return False

    left_channel, left_start, left_end, left_duration = left_key
    right_channel, right_start, right_end, right_duration = right_key
    if left_channel != right_channel or left_duration != right_duration:
        return False

    start_delta = abs((parse_dt(left_start) - parse_dt(right_start)).total_seconds())
    end_delta = abs((parse_dt(left_end) - parse_dt(right_end)).total_seconds())
    return (
        start_delta <= STREAM_TIME_MATCH_TOLERANCE_SECONDS
        and end_delta <= STREAM_TIME_MATCH_TOLERANCE_SECONDS
    )


def find_duplicate_tiny_transcripts(entries: list[dict]) -> list[dict]:
    coarse_groups: dict[tuple[str, str], list[dict]] = {}
    for entry in entries:
        duplicate_key = entry.get("duplicate_key")
        if duplicate_key is not None:
            channel_id, _, _, duration = duplicate_key
            coarse_groups.setdefault((channel_id, duration), []).append(entry)

    groups: list[list[dict]] = []
    for group_entries in coarse_groups.values():
        for entry in group_entries:
            for group in groups:
                if same_completed_livestream(group[0], entry):
                    group.append(entry)
                    break
            else:
                groups.append([entry])

    corrupted = []
    for group_entries in groups:
        if len(group_entries) < 2:
            continue

        largest = max(entry["word_count"] for entry in group_entries)
        if largest < MIN_DUPLICATE_GOOD_WORDS:
            continue

        threshold = max(1, int(largest * DUPLICATE_TINY_RATIO))
        for entry in group_entries:
            if entry["word_count"] <= threshold:
                richer = [
                    other["id"]
                    for other in group_entries
                    if other["id"] != entry["id"] and other["word_count"] == largest
                ]
                corrupted.append(
                    {
                        **entry,
                        "reason": "tiny_duplicate",
                        "duplicate_best_words": largest,
                        "duplicate_best_ids": richer,
                    }
                )
    return corrupted


def dedupe_corruption_entries(entries: list[dict]) -> list[dict]:
    by_id: dict[str, dict] = {}
    for entry in entries:
        current = by_id.get(entry["id"])
        if current is None:
            by_id[entry["id"]] = entry
            continue
        reasons = set(str(current.get("reason", "")).split("+"))
        reasons.update(str(entry.get("reason", "")).split("+"))
        current.update(entry)
        current["reason"] = "+".join(sorted(reason for reason in reasons if reason))
    return list(by_id.values())


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--apply", action="store_true", help="apply cleanup; otherwise dry-run")
    parser.add_argument("--db", type=Path, default=None, help="path to local search-fts.db")
    parser.add_argument("--id", action="append", dest="ids", help="limit to a video id; repeatable")
    args = parser.parse_args()

    load_env_file(BACKEND_ENV)
    db_path = args.db or find_default_db()
    bucket = os.environ.get("S3_DATA_BUCKET")
    vector_bucket = os.environ.get("S3_VECTOR_BUCKET")
    vector_index = os.environ.get("S3_VECTOR_INDEX", "search-chunks")
    if not bucket or not vector_bucket:
        raise SystemExit("S3_DATA_BUCKET and S3_VECTOR_BUCKET are required")

    rows = load_video_rows(db_path, set(args.ids) if args.ids else None)
    completed_entries: list[dict] = []
    stale_transcripts: list[dict] = []

    for start in range(0, len(rows), DEFAULT_BATCH_SIZE):
        batch = rows[start : start + DEFAULT_BATCH_SIZE]
        by_id = {video_id: title for video_id, title in batch}
        for item in fetch_youtube_items(list(by_id)):
            video_id = item.get("id")
            duplicate_key = completed_livestream_key(item)
            if not video_id or duplicate_key is None:
                continue
            end_time = duplicate_key[2]

            transcript_head = head_s3_key(bucket, f"transcripts/{video_id}.json")
            if not transcript_head:
                continue
            modified_raw = transcript_head.get("LastModified")
            if not modified_raw:
                continue
            transcript_payload = get_s3_json(bucket, f"transcripts/{video_id}.json")
            if not transcript_payload:
                continue

            stream_end = parse_dt(end_time)
            transcript_modified = parse_dt(modified_raw)
            duration = item.get("contentDetails", {}).get("duration")
            description = item.get("snippet", {}).get("description") or ""
            entry = {
                "id": video_id,
                "title": by_id.get(video_id, item.get("snippet", {}).get("title", "")),
                "actual_end": stream_end.isoformat(),
                "transcript_modified": transcript_modified.isoformat(),
                "duration": duration,
                "word_count": transcript_word_count(transcript_payload),
                "duplicate_key": duplicate_key,
                "description_like": transcript_looks_like_description(
                    transcript_payload,
                    description,
                    duration,
                ),
            }
            completed_entries.append(entry)

            if transcript_modified < stream_end:
                stale_transcripts.append({**entry, "reason": "stale_before_stream_end"})

    description_like_transcripts = [
        {**entry, "reason": "description_like"}
        for entry in completed_entries
        if entry.get("description_like")
    ]
    corrupted = dedupe_corruption_entries(
        [
            *stale_transcripts,
            *description_like_transcripts,
            *find_duplicate_tiny_transcripts(completed_entries),
        ]
    )
    if not corrupted:
        print(f"No corrupted completed livestream transcripts found in {db_path}")
        return 0

    print("Corrupted completed livestream transcripts:")
    for entry in corrupted:
        details = [
            f"reason={entry['reason']}",
            f"words={entry['word_count']}",
            f"transcript={entry['transcript_modified']}",
            f"ended={entry['actual_end']}",
            f"duration={entry['duration']}",
        ]
        if entry.get("duplicate_best_ids"):
            details.append(
                f"duplicate_best={','.join(entry['duplicate_best_ids'])}"
                f"({entry['duplicate_best_words']} words)"
            )
        print(f"- {entry['id']} | {entry['title']} | " + " | ".join(details))

    if not args.apply:
        print("Dry-run only. Re-run with --apply to reset these videos.")
        return 0

    reset_ids = [
        entry["id"]
        for entry in corrupted
        if "tiny_duplicate" not in str(entry.get("reason", "")).split("+")
    ]
    quarantine_ids = [
        entry["id"]
        for entry in corrupted
        if "tiny_duplicate" in str(entry.get("reason", "")).split("+")
    ]

    for video_id in [*reset_ids, *quarantine_ids]:
        object_keys, vector_keys = collect_cleanup_keys(bucket, video_id)
        delete_vectors(vector_bucket, vector_index, vector_keys)
        for key in object_keys:
            delete_s3_key(bucket, key)
        update_video_snapshot(bucket, video_id, quarantine=video_id in quarantine_ids)

    update_local_rows(db_path, reset_ids, quarantine_ids)
    print(
        f"Reset {len(reset_ids)} stale completed livestream video(s); "
        f"quarantined {len(quarantine_ids)} tiny duplicate(s)."
    )
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
