#!/bin/bash

# Docker Configuration Security Validator
# Validates DASTILL_BASE_PATH before Docker Compose execution to prevent path injection

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

error() {
    echo -e "${RED}[ERROR]${NC} $1" >&2
    exit 1
}

warning() {
    echo -e "${YELLOW}[WARNING]${NC} $1" >&2
}

success() {
    echo -e "${GREEN}[SUCCESS]${NC} $1"
}

# Check if .env file exists
if [[ ! -f .env ]]; then
    error ".env file not found. Please create one based on .env.example"
fi

# Source .env file to get DASTILL_BASE_PATH
source .env

# Validate DASTILL_BASE_PATH is set
if [[ -z "$DASTILL_BASE_PATH" ]]; then
    error "DASTILL_BASE_PATH is not set in .env file"
fi

# Security validation: Ensure path is absolute
if [[ ! "$DASTILL_BASE_PATH" =~ ^/ ]] && [[ ! "$DASTILL_BASE_PATH" =~ ^~ ]]; then
    error "DASTILL_BASE_PATH must be an absolute path (start with / or ~), got: $DASTILL_BASE_PATH"
fi

# Expand ~ if present
EXPANDED_PATH="${DASTILL_BASE_PATH/#\~/$HOME}"

# Security validation: Prevent dangerous paths
DANGEROUS_PATHS=(
    "/"
    "/etc"
    "/usr"
    "/bin"
    "/sbin"
    "/boot"
    "/dev"
    "/proc"
    "/sys"
    "/var"
    "/root"
)

for dangerous_path in "${DANGEROUS_PATHS[@]}"; do
    if [[ "$EXPANDED_PATH" == "$dangerous_path" ]] || [[ "$EXPANDED_PATH" == "$dangerous_path/"* ]]; then
        error "DASTILL_BASE_PATH points to dangerous system directory: $EXPANDED_PATH"
    fi
done

# Security validation: Prevent directory traversal
if [[ "$DASTILL_BASE_PATH" =~ \.\. ]]; then
    error "DASTILL_BASE_PATH contains directory traversal (..), which is not allowed: $DASTILL_BASE_PATH"
fi

# Check if path exists and is accessible
if [[ ! -d "$EXPANDED_PATH" ]]; then
    warning "Directory $EXPANDED_PATH does not exist. Creating it..."
    mkdir -p "$EXPANDED_PATH" || error "Failed to create directory: $EXPANDED_PATH"
fi

# Check if path is writable
if [[ ! -w "$EXPANDED_PATH" ]]; then
    error "Directory $EXPANDED_PATH is not writable"
fi

# Check if path is owned by current user (security best practice)
if [[ "$(stat -f "%Su" "$EXPANDED_PATH" 2>/dev/null || stat -c "%U" "$EXPANDED_PATH" 2>/dev/null)" != "$(whoami)" ]]; then
    warning "Directory $EXPANDED_PATH is not owned by current user. This may cause permission issues."
fi

success "DASTILL_BASE_PATH validation passed: $EXPANDED_PATH"

# Export the validated path for Docker Compose
export DASTILL_BASE_PATH="$EXPANDED_PATH"

# Execute docker-compose with validated configuration
exec docker-compose "$@"
