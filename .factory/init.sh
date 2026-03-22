#!/usr/bin/env bash
set -euo pipefail

cd "$(dirname "$0")/.."

# Install frontend dependencies
cd frontend
bun install --frozen-lockfile 2>/dev/null || bun install
cd ..

# Verify backend compiles
cd backend
cargo check 2>/dev/null || echo "Warning: cargo check failed, may need manual intervention"
cd ..
