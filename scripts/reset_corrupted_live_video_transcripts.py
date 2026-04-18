#!/usr/bin/env python3
"""Find completed livestreams with transcripts captured before the stream ended.

The cleanup rule is intentionally conservative:
- YouTube must report liveStreamingDetails.actualEndTime.
- A transcript object must exist.
- The transcript object's LastModified must be before actualEndTime.

Use --apply to delete stale transcript/summary/search artifacts and reset local
video rows to pending so the queue rebuilds them from the completed VOD.
"""

from __future__ import annotations

import argparse
import json
import os
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


def update_local_rows(db_path: Path, video_ids: list[str]) -> None:
    if not video_ids:
        return
    with sqlite3.connect(db_path) as conn:
        placeholders = ",".join("?" for _ in video_ids)
        conn.execute(
            f"DELETE FROM fts_search WHERE video_id IN ({placeholders})",
            tuple(video_ids),
        )
        conn.execute(
            f"""
            UPDATE videos
               SET transcript_status = 'pending',
                   summary_status = 'pending',
                   retry_count = 0
             WHERE id IN ({placeholders})
            """,
            tuple(video_ids),
        )
        conn.commit()


def update_video_snapshot(bucket: str, video_id: str) -> None:
    key = f"videos/{video_id}.json"
    result = run(["aws", "s3", "cp", f"s3://{bucket}/{key}", "-"], check=False)
    if result.returncode != 0 or not result.stdout.strip():
        return
    record = json.loads(result.stdout)
    record["transcript_status"] = "pending"
    record["summary_status"] = "pending"
    record["retry_count"] = 0
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
    corrupted: list[dict] = []

    for start in range(0, len(rows), DEFAULT_BATCH_SIZE):
        batch = rows[start : start + DEFAULT_BATCH_SIZE]
        by_id = {video_id: title for video_id, title in batch}
        for item in fetch_youtube_items(list(by_id)):
            video_id = item.get("id")
            end_time = item.get("liveStreamingDetails", {}).get("actualEndTime")
            if not video_id or not end_time:
                continue

            transcript_head = head_s3_key(bucket, f"transcripts/{video_id}.json")
            if not transcript_head:
                continue
            modified_raw = transcript_head.get("LastModified")
            if not modified_raw:
                continue

            stream_end = parse_dt(end_time)
            transcript_modified = parse_dt(modified_raw)
            if transcript_modified >= stream_end:
                continue

            corrupted.append(
                {
                    "id": video_id,
                    "title": by_id.get(video_id, item.get("snippet", {}).get("title", "")),
                    "actual_end": stream_end.isoformat(),
                    "transcript_modified": transcript_modified.isoformat(),
                    "duration": item.get("contentDetails", {}).get("duration"),
                }
            )

    if not corrupted:
        print(f"No corrupted completed livestream transcripts found in {db_path}")
        return 0

    print("Corrupted completed livestream transcripts:")
    for entry in corrupted:
        print(
            f"- {entry['id']} | {entry['title']} | transcript={entry['transcript_modified']} "
            f"< ended={entry['actual_end']} | duration={entry['duration']}"
        )

    if not args.apply:
        print("Dry-run only. Re-run with --apply to reset these videos.")
        return 0

    reset_ids = [entry["id"] for entry in corrupted]
    for video_id in reset_ids:
        object_keys, vector_keys = collect_cleanup_keys(bucket, video_id)
        delete_vectors(vector_bucket, vector_index, vector_keys)
        for key in object_keys:
            delete_s3_key(bucket, key)
        update_video_snapshot(bucket, video_id)

    update_local_rows(db_path, reset_ids)
    print(f"Reset {len(reset_ids)} corrupted completed livestream video(s).")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
