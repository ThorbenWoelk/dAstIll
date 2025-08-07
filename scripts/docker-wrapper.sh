#!/bin/bash

# Secure Docker Wrapper for dAstIll
# Uses Python configuration system instead of direct Docker environment mounting
# Addresses security issues from direct path interpolation

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

info() {
    echo -e "${BLUE}[INFO]${NC} $1"
}

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

# Validate we're in the correct directory
if [[ ! -f "docker-compose.yml" ]]; then
    error "docker-compose.yml not found. Please run from the dAstIll project root directory."
fi

# Validate Python configuration first
info "Validating dAstIll configuration..."
if ! uv run python -c "from config.config import Config; c = Config(); print('✓ Configuration valid')"; then
    error "Python configuration validation failed. Please check your .env file and configuration."
fi

# Get the validated base path from Python configuration
BASE_PATH=$(uv run python -c "from config.config import Config; c = Config(); print(c.get('storage.base_path'))" 2>/dev/null)

if [[ -z "$BASE_PATH" ]]; then
    error "Could not determine storage base path from Python configuration"
fi

success "Using validated base path: $BASE_PATH"

# Security check: Ensure the path is safe
if [[ "$BASE_PATH" == "/" ]] || [[ "$BASE_PATH" =~ ^/(etc|usr|bin|sbin|boot|dev|proc|sys|var|root)(/|$) ]]; then
    error "Refusing to use dangerous system directory: $BASE_PATH"
fi

# Check if path exists and is accessible
if [[ ! -d "$BASE_PATH" ]]; then
    warning "Directory $BASE_PATH does not exist. Creating it..."
    mkdir -p "$BASE_PATH" || error "Failed to create directory: $BASE_PATH"
fi

if [[ ! -w "$BASE_PATH" ]]; then
    error "Directory $BASE_PATH is not writable"
fi

# Create a temporary docker-compose override file with the validated path
TEMP_OVERRIDE=$(mktemp)
cat > "$TEMP_OVERRIDE" << EOF
services:
  dastill-monitor:
    volumes:
      - "${BASE_PATH}:/app/data"
  dastill-cli:
    volumes:
      - "${BASE_PATH}:/app/data"
EOF

info "Created temporary Docker Compose override with validated paths"

# Cleanup function
cleanup() {
    rm -f "$TEMP_OVERRIDE"
}
trap cleanup EXIT

# Execute docker-compose with the validated configuration
info "Executing: docker-compose -f docker-compose.yml -f $TEMP_OVERRIDE $*"
exec docker-compose -f docker-compose.yml -f "$TEMP_OVERRIDE" "$@"
