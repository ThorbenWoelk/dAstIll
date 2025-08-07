#!/bin/bash

# Docker Wrapper for dAstIll
# Provides secure path validation and environment setup

set -e

# Validate we're in the correct directory
if [[ ! -f "docker-compose.yml" ]]; then
    echo "Error: docker-compose.yml not found. Please run from the dAstIll project root directory." >&2
    exit 1
fi

# Get the base path from configuration (without verbose output)
BASE_PATH=$(uv run python -c "from config.config import Config; c = Config(); print(c.get('storage.base_path'))" 2>/dev/null)

if [[ -z "$BASE_PATH" ]]; then
    echo "Error: Could not determine storage base path from configuration" >&2
    echo "Please ensure your .env file or configuration is properly set up." >&2
    exit 1
fi

# Security check: Ensure the path is safe
if [[ "$BASE_PATH" == "/" ]] || [[ "$BASE_PATH" =~ ^/(etc|usr|bin|sbin|boot|dev|proc|sys|var|root)(/|$) ]]; then
    echo "Error: Refusing to use dangerous system directory: $BASE_PATH" >&2
    exit 1
fi

# Check if path exists and is accessible
if [[ ! -d "$BASE_PATH" ]]; then
    mkdir -p "$BASE_PATH" 2>/dev/null || {
        echo "Error: Failed to create directory: $BASE_PATH" >&2
        exit 1
    }
fi

if [[ ! -w "$BASE_PATH" ]]; then
    echo "Error: Directory $BASE_PATH is not writable" >&2
    exit 1
fi

# Set environment variables for docker-compose
export DASTILL_HOST_DATA_PATH="$BASE_PATH"
export DASTILL_CONTAINER_DATA_PATH="/app/data"

# Execute docker-compose with environment variables
exec docker-compose "$@"
