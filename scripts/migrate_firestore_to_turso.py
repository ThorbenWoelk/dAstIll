#!/usr/bin/env python3
"""One-time migration: Firestore → Turso.

Reads dastill_videos, dastill_preferences, and dastill_tts_stats from
Firestore via the REST API and inserts them into the Turso database.

Requirements:
  pip install requests
  gcloud auth print-access-token   (must work)

Usage:
  python scripts/migrate_firestore_to_turso.py
"""

import json
import os
import subprocess
import sys
import requests

GCP_PROJECT = "dastill"
FIRESTORE_BASE = f"https://firestore.googleapis.com/v1/projects/{GCP_PROJECT}/databases/(default)/documents"


# Read Turso config from backend/.env
def load_env(path: str) -> dict[str, str]:
    env = {}
    if not os.path.exists(path):
        return env
    with open(path) as f:
        for line in f:
            line = line.strip()
            if not line or line.startswith("#"):
                continue
            if "=" in line:
                k, v = line.split("=", 1)
                env[k.strip()] = v.strip()
    return env


def get_access_token() -> str:
    result = subprocess.run(
        ["gcloud", "auth", "print-access-token"],
        capture_output=True,
        text=True,
        check=True,
    )
    return result.stdout.strip()


def firestore_get_all(collection: str, token: str) -> list[dict]:
    """Paginate through all documents in a Firestore collection."""
    docs = []
    url = f"{FIRESTORE_BASE}/{collection}?pageSize=300"
    while url:
        resp = requests.get(url, headers={"Authorization": f"Bearer {token}"})
        resp.raise_for_status()
        data = resp.json()
        docs.extend(data.get("documents", []))
        next_token = data.get("nextPageToken")
        if next_token:
            base = f"{FIRESTORE_BASE}/{collection}?pageSize=300"
            url = f"{base}&pageToken={next_token}"
        else:
            url = None
    return docs


def fs_str(fields: dict, key: str, default: str = "") -> str:
    f = fields.get(key)
    if f is None:
        return default
    return f.get("stringValue", default)


def fs_bool(fields: dict, key: str, default: bool = False) -> bool:
    f = fields.get(key)
    if f is None:
        return default
    return f.get("booleanValue", default)


def fs_int(fields: dict, key: str, default: int = 0) -> int:
    f = fields.get(key)
    if f is None:
        return default
    return int(f.get("integerValue", default))


def fs_float(fields: dict, key: str, default: float = 0.0) -> float:
    f = fields.get(key)
    if f is None:
        return default
    if "doubleValue" in f:
        return float(f["doubleValue"])
    if "integerValue" in f:
        return float(f["integerValue"])
    return default


def turso_execute(db_url: str, auth_token: str, statements: list[dict]) -> dict:
    """Execute statements via Turso HTTP API (pipeline)."""
    # Convert libsql:// to https://
    http_url = db_url.replace("libsql://", "https://")
    url = f"{http_url}/v3/pipeline"

    requests_payload = []
    for stmt in statements:
        requests_payload.append(
            {
                "type": "execute",
                "stmt": stmt,
            }
        )
    requests_payload.append({"type": "close"})

    resp = requests.post(
        url,
        headers={
            "Authorization": f"Bearer {auth_token}",
            "Content-Type": "application/json",
        },
        json={"requests": requests_payload},
    )
    resp.raise_for_status()
    return resp.json()


def migrate_videos(docs: list[dict], db_url: str, auth_token: str):
    print(f"Migrating {len(docs)} videos...")

    # Batch in groups of 50
    batch_size = 50
    for i in range(0, len(docs), batch_size):
        batch = docs[i : i + batch_size]
        stmts = []
        for doc in batch:
            fields = doc["fields"]
            video_id = fs_str(fields, "id")
            if not video_id:
                # Extract from document path
                video_id = doc["name"].split("/")[-1]

            quality_score = fields.get("quality_score")
            qs_val = None
            if quality_score and "integerValue" in quality_score:
                qs_val = int(quality_score["integerValue"])

            args = [
                {"type": "text", "value": video_id},
                {"type": "text", "value": fs_str(fields, "channel_id")},
                {"type": "text", "value": fs_str(fields, "title")},
                {"type": "text", "value": fs_str(fields, "thumbnail_url")}
                if fs_str(fields, "thumbnail_url")
                else {"type": "null"},
                {"type": "text", "value": fs_str(fields, "published_at")},
                {
                    "type": "integer",
                    "value": str(1 if fs_bool(fields, "is_short") else 0),
                },
                {
                    "type": "text",
                    "value": fs_str(fields, "transcript_status", "pending"),
                },
                {"type": "text", "value": fs_str(fields, "summary_status", "pending")},
                {
                    "type": "integer",
                    "value": str(1 if fs_bool(fields, "acknowledged") else 0),
                },
                {"type": "integer", "value": str(fs_int(fields, "retry_count", 0))},
            ]
            if qs_val is not None:
                args.append({"type": "integer", "value": str(qs_val)})
            else:
                args.append({"type": "null"})

            stmts.append(
                {
                    "sql": """INSERT OR REPLACE INTO videos
                    (id, channel_id, title, thumbnail_url, published_at,
                     is_short, transcript_status, summary_status,
                     acknowledged, retry_count, quality_score)
                    VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)""",
                    "args": args,
                }
            )

        turso_execute(db_url, auth_token, stmts)
        end = min(i + batch_size, len(docs))
        print(f"  inserted {end}/{len(docs)}")

    print(f"  done: {len(docs)} videos migrated")


def migrate_preferences(docs: list[dict], db_url: str, auth_token: str):
    print(f"Migrating {len(docs)} preference documents...")
    stmts = []
    for doc in docs:
        user_id = doc["name"].split("/")[-1]
        fields = doc["fields"]

        # Build preferences JSON matching the Rust model
        prefs = {
            "channel_sort_mode": fs_str(fields, "channel_sort_mode", "custom"),
        }

        # Extract channel_order array
        co = fields.get("channel_order", {})
        if "arrayValue" in co and "values" in co["arrayValue"]:
            prefs["channel_order"] = [
                v["stringValue"]
                for v in co["arrayValue"]["values"]
                if "stringValue" in v
            ]
        else:
            prefs["channel_order"] = []

        # Extract vocabulary_replacements array
        vr = fields.get("vocabulary_replacements", {})
        if "arrayValue" in vr and "values" in vr["arrayValue"]:
            replacements = []
            for v in vr["arrayValue"]["values"]:
                if "mapValue" in v and "fields" in v["mapValue"]:
                    mf = v["mapValue"]["fields"]
                    replacements.append(
                        {
                            "from": fs_str(mf, "from"),
                            "to": fs_str(mf, "to"),
                            "added_at": fs_str(mf, "added_at"),
                        }
                    )
            prefs["vocabulary_replacements"] = replacements
        else:
            prefs["vocabulary_replacements"] = []

        stmts.append(
            {
                "sql": "INSERT OR REPLACE INTO preferences (user_id, data) VALUES (?, ?)",
                "args": [
                    {"type": "text", "value": user_id},
                    {"type": "text", "value": json.dumps(prefs)},
                ],
            }
        )

    turso_execute(db_url, auth_token, stmts)
    print(f"  done: {len(docs)} preference docs migrated")


def migrate_tts_stats(docs: list[dict], db_url: str, auth_token: str):
    print(f"Migrating {len(docs)} TTS stats documents...")
    stmts = []
    for doc in docs:
        doc_id = doc["name"].split("/")[-1]
        fields = doc["fields"]
        stmts.append(
            {
                "sql": """INSERT OR REPLACE INTO tts_stats
                (id, sample_count, total_words, total_duration_secs)
                VALUES (?, ?, ?, ?)""",
                "args": [
                    {"type": "text", "value": doc_id},
                    {
                        "type": "integer",
                        "value": str(fs_int(fields, "sample_count", 0)),
                    },
                    {"type": "integer", "value": str(fs_int(fields, "total_words", 0))},
                    {
                        "type": "text",
                        "value": str(fs_float(fields, "total_duration_secs", 0.0)),
                    },
                ],
            }
        )

    turso_execute(db_url, auth_token, stmts)
    print(f"  done: {len(docs)} TTS stats migrated")


def main():
    repo_root = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))

    # Load Turso config - check shared env first, then backend/.env
    shared_env = os.path.expanduser("~/.config/dastill/backend.env")
    local_env = os.path.join(repo_root, "backend", ".env")

    env = {}
    if os.path.exists(shared_env):
        env.update(load_env(shared_env))
    env.update(load_env(local_env))

    db_url = env.get("TURSO_DB_URL")
    auth_token = env.get("TURSO_AUTH_TOKEN")
    if not db_url or not auth_token:
        print("Error: TURSO_DB_URL and TURSO_AUTH_TOKEN must be set in backend/.env")
        sys.exit(1)

    print(f"Turso target: {db_url}")

    # Verify Turso is reachable
    try:
        result = turso_execute(db_url, auth_token, [{"sql": "SELECT 1", "args": []}])
        print("Turso connection: ok")
    except Exception as e:
        print(f"Error connecting to Turso: {e}")
        sys.exit(1)

    # Ensure schema exists
    turso_execute(
        db_url,
        auth_token,
        [
            {
                "sql": """CREATE TABLE IF NOT EXISTS videos (
            id TEXT PRIMARY KEY,
            channel_id TEXT NOT NULL,
            title TEXT NOT NULL,
            thumbnail_url TEXT,
            published_at TEXT NOT NULL,
            is_short INTEGER NOT NULL DEFAULT 0,
            transcript_status TEXT NOT NULL DEFAULT 'pending',
            summary_status TEXT NOT NULL DEFAULT 'pending',
            acknowledged INTEGER NOT NULL DEFAULT 0,
            retry_count INTEGER NOT NULL DEFAULT 0,
            quality_score INTEGER
        )""",
                "args": [],
            },
            {
                "sql": "CREATE INDEX IF NOT EXISTS idx_videos_channel_published ON videos(channel_id, published_at DESC)",
                "args": [],
            },
            {
                "sql": "CREATE INDEX IF NOT EXISTS idx_videos_transcript_status ON videos(transcript_status)",
                "args": [],
            },
            {
                "sql": "CREATE INDEX IF NOT EXISTS idx_videos_summary_status ON videos(summary_status)",
                "args": [],
            },
            {
                "sql": "CREATE TABLE IF NOT EXISTS preferences (user_id TEXT PRIMARY KEY, data TEXT NOT NULL)",
                "args": [],
            },
            {
                "sql": """CREATE TABLE IF NOT EXISTS tts_stats (
            id TEXT PRIMARY KEY DEFAULT 'global',
            sample_count INTEGER NOT NULL DEFAULT 0,
            total_words INTEGER NOT NULL DEFAULT 0,
            total_duration_secs REAL NOT NULL DEFAULT 0.0
        )""",
                "args": [],
            },
        ],
    )
    print("Schema: ok")

    # Get GCP access token
    print("Authenticating with GCP...")
    gcp_token = get_access_token()
    print("GCP auth: ok")

    # Export and migrate each collection
    print("\n--- Videos ---")
    video_docs = firestore_get_all("dastill_videos", gcp_token)
    print(f"Exported {len(video_docs)} video documents from Firestore")
    if video_docs:
        migrate_videos(video_docs, db_url, auth_token)

    print("\n--- Preferences ---")
    pref_docs = firestore_get_all("dastill_preferences", gcp_token)
    print(f"Exported {len(pref_docs)} preference documents from Firestore")
    if pref_docs:
        migrate_preferences(pref_docs, db_url, auth_token)

    print("\n--- TTS Stats ---")
    tts_docs = firestore_get_all("dastill_tts_stats", gcp_token)
    print(f"Exported {len(tts_docs)} TTS stats documents from Firestore")
    if tts_docs:
        migrate_tts_stats(tts_docs, db_url, auth_token)

    # Verify
    print("\n--- Verification ---")
    result = turso_execute(
        db_url,
        auth_token,
        [
            {"sql": "SELECT COUNT(*) as c FROM videos", "args": []},
            {"sql": "SELECT COUNT(*) as c FROM preferences", "args": []},
            {"sql": "SELECT COUNT(*) as c FROM tts_stats", "args": []},
        ],
    )

    for i, table in enumerate(["videos", "preferences", "tts_stats"]):
        rows = result["results"][i]["response"]["result"]["rows"]
        count = rows[0][0]["value"] if rows else "?"
        print(f"  {table}: {count} rows")

    print("\nMigration complete.")


if __name__ == "__main__":
    main()
