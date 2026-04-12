#!/usr/bin/env bash
set -euo pipefail

cd "$(dirname "$0")/.."

# Upstream-only advisories still open in dependency trees we do not fully control yet.
# Keep this list short and remove entries as soon as crates publish fixed releases we can adopt.
IGNORED_ADVISORIES=(
  "RUSTSEC-2025-0141" # bincode via libsql replication/sync stack
  "RUSTSEC-2025-0134" # rustls-pemfile via libsql -> hyper-rustls 0.25
  "RUSTSEC-2026-0097" # rand via libsql/aws-config/reqwest transitive trees
)

echo "cargo audit allowlist:"
for advisory in "${IGNORED_ADVISORIES[@]}"; do
  echo "  - $advisory"
done

args=()
for advisory in "${IGNORED_ADVISORIES[@]}"; do
  args+=(--ignore "$advisory")
done

exec cargo audit "${args[@]}" "$@"
