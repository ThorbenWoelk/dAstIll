#!/usr/bin/env bash

set -euo pipefail

mode="${1:-}"

if [[ "$mode" != "validation" && "$mode" != "deploy" ]]; then
  echo "usage: $0 <validation|deploy>" >&2
  exit 1
fi

backend=false
frontend=false
docs=false

write_outputs() {
  local any=false

  if [[ "$backend" == "true" || "$frontend" == "true" || "$docs" == "true" ]]; then
    any=true
  fi

  if [[ -n "${GITHUB_OUTPUT:-}" ]]; then
    {
      echo "backend=$backend"
      echo "frontend=$frontend"
      echo "docs=$docs"
      echo "any=$any"
    } >> "$GITHUB_OUTPUT"
    return
  fi

  printf 'backend=%s\n' "$backend"
  printf 'frontend=%s\n' "$frontend"
  printf 'docs=%s\n' "$docs"
  printf 'any=%s\n' "$any"
}

set_all() {
  backend=true
  frontend=true
  docs=true
}

set_from_dispatch_target() {
  case "${TARGET:-all}" in
    all)
      set_all
      ;;
    backend)
      backend=true
      ;;
    frontend)
      frontend=true
      ;;
    docs)
      docs=true
      ;;
    *)
      echo "unknown TARGET: ${TARGET:-}" >&2
      exit 1
      ;;
  esac
}

set_from_workflow_call_inputs() {
  if [[ "${RUN_ALL:-false}" == "true" ]]; then
    set_all
    return
  fi

  backend="${RUN_BACKEND:-false}"
  frontend="${RUN_FRONTEND:-false}"
  docs="${RUN_DOCS:-false}"
}

classify_validation_file() {
  local file="$1"

  case "$file" in
    backend/.gitignore|backend/README.md)
      ;;
    backend/*)
      backend=true
      ;;
    frontend/.gitignore|frontend/README.md)
      ;;
    frontend/*)
      frontend=true
      ;;
    docs/.gitignore|docs/README.md)
      ;;
    docs/*)
      docs=true
      ;;
  esac
}

classify_deploy_file() {
  local file="$1"

  case "$file" in
    backend/tests/*|backend/scripts/*|backend/.cargo/audit.toml|backend/openapi.postman.yaml|backend/package.json|backend/bun.lock|backend/.env.example|backend/.gitignore|backend/README.md)
      ;;
    backend/*)
      backend=true
      ;;
    frontend/tests/*|frontend/e2e/*|frontend/scripts/*|frontend/playwright.config.ts|frontend/firebase.json|frontend/.env.example|frontend/.gitignore|frontend/.prettierignore|frontend/.prettierrc|frontend/README.md|frontend/eslint.config.js)
      ;;
    frontend/*)
      frontend=true
      ;;
    docs/.gitignore|docs/README.md)
      ;;
    docs/*)
      docs=true
      ;;
  esac
}

classify_file() {
  local file="$1"

  if [[ "$mode" == "validation" ]]; then
    classify_validation_file "$file"
    return
  fi

  classify_deploy_file "$file"
}

if [[ "${GITHUB_EVENT_NAME:-}" == "workflow_call" ]]; then
  set_from_workflow_call_inputs
  write_outputs
  exit 0
fi

if [[ "$mode" == "validation" ]]; then
  case "${GITHUB_EVENT_NAME:-}" in
    schedule|workflow_dispatch)
      set_all
      write_outputs
      exit 0
      ;;
  esac
fi

if [[ "$mode" == "deploy" && "${GITHUB_EVENT_NAME:-}" == "workflow_dispatch" ]]; then
  set_from_dispatch_target
  write_outputs
  exit 0
fi

head_sha="${GITHUB_SHA:-$(git rev-parse HEAD)}"
before_sha="${BEFORE_SHA:-}"
base_ref="${GITHUB_BASE_REF:-}"
default_branch="${DEFAULT_BRANCH:-main}"
base_sha=""

if [[ -n "$before_sha" && "$before_sha" != "0000000000000000000000000000000000000000" ]]; then
  base_sha="$before_sha"
elif [[ -n "$base_ref" ]]; then
  git fetch --no-tags --depth=1 origin "$base_ref"
  base_sha="$(git merge-base HEAD "origin/$base_ref")"
else
  git fetch --no-tags --depth=1 origin "$default_branch"
  base_sha="$(git merge-base HEAD "origin/$default_branch")"
fi

if [[ -z "$base_sha" ]]; then
  set_all
  write_outputs
  exit 0
fi

while IFS= read -r file; do
  classify_file "$file"
done < <(git diff --name-only "$base_sha" "$head_sha")

write_outputs
