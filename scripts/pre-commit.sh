#!/bin/bash
# Pre-commit hook for dAstIll project
# Install with: cp scripts/pre-commit.sh .git/hooks/pre-commit
# Or run manually: ./scripts/pre-commit.sh

set -euo pipefail

echo "🔍 Running pre-commit checks..."

# Check if we're in a git repository
if ! git rev-parse --git-dir > /dev/null 2>&1; then
    echo "❌ Not in a git repository"
    exit 1
fi

# Frontend checks
FRONTEND_STAGED=$(git diff --cached --name-only | grep -E '^frontend/.*\.(ts|js|svelte|css|html)$' || true)
FRONTEND_PKGS=$(git diff --cached --name-only | grep -E '^frontend/(package\.json|bun\.lock|bun\.lockb)$' || true)

if [[ -n "$FRONTEND_STAGED" ]]; then
    echo "📝 Frontend files staged, running frontend checks..."
    if ! (cd frontend && bun run format:check); then
        echo "❌ Frontend format check failed"
        exit 1
    fi
    if ! (cd frontend && bun run lint); then
        echo "❌ Frontend lint failed"
        exit 1
    fi
    if ! (cd frontend && bun run check); then
        echo "❌ Frontend typecheck failed"
        exit 1
    fi
    if ! (cd frontend && bun run test); then
        echo "❌ Frontend tests failed"
        exit 1
    fi
fi

if [[ -n "$FRONTEND_STAGED" || -n "$FRONTEND_PKGS" ]]; then
    echo "🔒 Running frontend bun audit..."
    if ! (cd frontend && bun audit --production); then
        echo "❌ Frontend audit failed"
        exit 1
    fi
fi

# Backend checks
BACKEND_STAGED=$(git diff --cached --name-only | grep -E '^backend/' || true)
BACKEND_PKGS=$(git diff --cached --name-only | grep -E '^backend/Cargo\.(toml|lock)$' || true)

if [[ -n "$BACKEND_STAGED" ]]; then
    echo "🦀 Backend files staged, running backend checks..."
    if ! (cd backend && cargo check); then
        echo "❌ Backend check failed"
        exit 1
    fi
    if ! (cd backend && cargo test); then
        echo "❌ Backend tests failed"
        exit 1
    fi
fi

if [[ -n "$BACKEND_STAGED" || -n "$BACKEND_PKGS" ]]; then
    echo "🔒 Running backend cargo audit..."
    if ! (cd backend && ./scripts/cargo_audit.sh); then
        echo "❌ Backend audit failed"
        exit 1
    fi
fi

# Startup smoke test for backend (only if backend source changed)
if [[ -n "$BACKEND_STAGED" ]] && git diff --cached --name-only | grep -qE 'backend/src/'; then
    echo "🔥 Running backend startup smoke test..."
    
    # Build release binary if not present or source changed
    if [[ ! -f backend/target/release/dastill ]] || \
       git diff --cached --name-only | grep -qE 'backend/(src|Cargo\.toml|Cargo\.lock)'; then
        echo "Building release binary..."
        if ! (cd backend && cargo build --release --bin dastill); then
            echo "❌ Failed to build backend binary"
            exit 1
        fi
    fi
    
    # Minimal env for startup check
    export BACKEND_PROXY_TOKEN="test"
    export GCP_PROJECT_ID="test"
    export YOUTUBE_API_KEY="test"
    export OLLAMA_URL="http://localhost:11434"
    export OLLAMA_API_KEY="test"
    export OLLAMA_SUMMARY_MODEL="test"
    export SUMMARY_EVALUATOR_MODEL="test"
    export BACKEND_CORS_ALLOWED_ORIGINS="http://localhost:3000"
    export OBJECT_STORE_PROVIDER="memory"
    export ALLOW_IN_MEMORY_OBJECT_STORE="true"
    export SEARCH_SEMANTIC_ENABLED="false"
    
    # Run with timeout - must exceed the libSQL builder inner timeout (30s) so the binary
    # can fail cleanly with exit 1 rather than being killed with exit 124.
    exit_code=0
    timeout 45s ./backend/target/release/dastill 2>&1 || exit_code=$?

    # Exit codes:
    # 124 = timeout (process hung past 45s) - FAIL
    # 1 = early error (config validation, libSQL/open startup failure) - PASS (expected)
    # 0 = started successfully - PASS
    if [[ "$exit_code" == "124" ]]; then
        echo "❌ Backend startup hung for over 45s"
        exit 1
    fi
    
    echo "✅ Backend startup smoke test passed (early failure as expected)"
fi

echo "✅ All pre-commit checks passed!"
exit 0
