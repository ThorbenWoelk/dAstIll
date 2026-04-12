#!/usr/bin/env bash
set -euo pipefail

cd "$(dirname "$0")/.."

today_utc="$(date -u +%F)"

# Upstream-only advisories still open in dependency trees we do not fully control yet.
# Keep this list short and remove entries as soon as crates publish fixed releases we can adopt.
IGNORED_ADVISORIES=(
  "RUSTSEC-2025-0141|2026-07-01|bincode via libsql replication/sync stack"
  "RUSTSEC-2025-0134|2026-07-01|rustls-pemfile via libsql -> hyper-rustls 0.25"
  "RUSTSEC-2026-0097|2026-07-01|rand via libsql/aws-config/logfire transitive trees"
)

echo "cargo audit allowlist:"
for entry in "${IGNORED_ADVISORIES[@]}"; do
  IFS="|" read -r advisory review_after reason <<<"$entry"
  if [[ "$today_utc" > "$review_after" ]]; then
    echo "stale cargo audit waiver: $advisory review_after=$review_after reason=$reason" >&2
    echo "re-review upstream dependency status and either remove or extend this waiver explicitly." >&2
    exit 1
  fi

  echo "  - $advisory review_after=$review_after reason=$reason"
done

args=()
for entry in "${IGNORED_ADVISORIES[@]}"; do
  IFS="|" read -r advisory _rest <<<"$entry"
  args+=(--ignore "$advisory")
done

exec cargo audit "${args[@]}" "$@"
