#!/usr/bin/env bash
set -euo pipefail

cd "$(dirname "$0")/.."

today_utc="$(date -u +%F)"

exec cargo audit "$@"
