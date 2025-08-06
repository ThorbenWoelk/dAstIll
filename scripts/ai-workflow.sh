#!/bin/bash

# AI Workflow Automation Script
# 
# This script implements CH-141: Claude Code integration for automated transcript processing
# 
# Workflow:
# 1. Start Docker container for automated video monitoring/downloading
# 2. Use Claude Code to process downloaded transcripts with the professor agent
# 3. Run dAstIll process command to organize enhanced transcripts

set -e  # Exit on any error

# Configuration
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_DIR="$(dirname "$SCRIPT_DIR")"
DOCKER_COMPOSE_FILE="$PROJECT_DIR/docker-compose.yml"
LOG_FILE="$PROJECT_DIR/logs/ai-workflow.log"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Logging function
log() {
    echo -e "${BLUE}[$(date +'%Y-%m-%d %H:%M:%S')]${NC} $1" | tee -a "$LOG_FILE"
}

error() {
    echo -e "${RED}[ERROR]${NC} $1" | tee -a "$LOG_FILE"
    exit 1
}

success() {
    echo -e "${GREEN}[SUCCESS]${NC} $1" | tee -a "$LOG_FILE"
}

warning() {
    echo -e "${YELLOW}[WARNING]${NC} $1" | tee -a "$LOG_FILE"
}

# Check prerequisites
check_prerequisites() {
    log "Checking prerequisites..."
    
    # Check if Docker is available
    if ! command -v docker &> /dev/null; then
        error "Docker is not installed or not in PATH"
    fi
    
    # Check if docker-compose is available
    if ! command -v docker-compose &> /dev/null; then
        error "docker-compose is not installed or not in PATH"
    fi
    
    # Check if Claude Code is available
    if ! command -v claude &> /dev/null && ! command -v claude-code &> /dev/null; then
        error "Claude Code CLI is not installed or not in PATH"
    fi
    
    # Check if docker-compose.yml exists
    if [[ ! -f "$DOCKER_COMPOSE_FILE" ]]; then
        error "docker-compose.yml not found at $DOCKER_COMPOSE_FILE"
    fi
    
    # Create logs directory
    mkdir -p "$(dirname "$LOG_FILE")"
    
    success "Prerequisites check passed"
}

# Start Docker container for monitoring
start_monitoring() {
    log "Starting Docker container for automated monitoring..."
    
    cd "$PROJECT_DIR"
    
    # Start the container in detached mode
    if docker-compose up -d; then
        success "Docker container started successfully"
        
        # Wait a moment for container to initialize
        sleep 5
        
        # Check container status
        if docker-compose ps | grep -q "Up"; then
            success "Container is running and monitoring channels"
        else
            error "Container failed to start properly"
        fi
    else
        error "Failed to start Docker container"
    fi
}

# Process transcripts with Claude Code
process_transcripts() {
    log "Processing downloaded transcripts with Claude Code..."
    
    # Get the configured storage path from dAstIll
    cd "$PROJECT_DIR"
    STORAGE_INFO=$(uv run python main.py config 2>/dev/null | grep "base_path" || echo "")
    
    if [[ -z "$STORAGE_INFO" ]]; then
        warning "Could not determine storage path, using default ~/.dastill/transcripts"
        BASE_PATH="$HOME/.dastill/transcripts"
    else
        BASE_PATH=$(echo "$STORAGE_INFO" | sed 's/.*: //' | tr -d '"')
    fi
    
    DOWNLOADED_DIR="$BASE_PATH/downloaded"
    
    log "Looking for transcripts in: $DOWNLOADED_DIR"
    
    # Check if downloaded directory exists and has files
    if [[ ! -d "$DOWNLOADED_DIR" ]]; then
        warning "Downloaded directory does not exist: $DOWNLOADED_DIR"
        return 0
    fi
    
    # Count files in downloaded directory
    FILE_COUNT=$(find "$DOWNLOADED_DIR" -name "*.md" -type f | wc -l)
    
    if [[ "$FILE_COUNT" -eq 0 ]]; then
        log "No transcript files found in downloaded directory"
        return 0
    fi
    
    success "Found $FILE_COUNT transcript files to process"
    
    # Use Claude Code to process transcripts
    log "Launching Claude Code for transcript processing..."
    
    # Determine Claude Code command
    CLAUDE_CMD="claude"
    if command -v claude-code &> /dev/null; then
        CLAUDE_CMD="claude-code"
    fi
    
    # Create automated prompt for Claude Code
    PROMPT="Use the transcript-education-curator agent to process all transcript files in $DOWNLOADED_DIR. For each .md file, transform it into a well-structured educational summary with key concepts, insights, and actionable takeaways. Replace each original file with the enhanced version."
    
    # Launch Claude Code in non-interactive mode with Task tool
    # Using --dangerously-skip-permissions and --add-dir for full automation
    if echo "$PROMPT" | timeout 600 "$CLAUDE_CMD" --print --dangerously-skip-permissions --add-dir "$BASE_PATH" > /tmp/claude-processing.log 2>&1; then
        success "Claude Code transcript processing completed"
        log "Processing log saved to /tmp/claude-processing.log"
        # Show last few lines of processing for confirmation
        tail -10 /tmp/claude-processing.log || true
    else
        EXIT_CODE=$?
        if [ $EXIT_CODE -eq 124 ]; then
            error "Claude Code processing timed out after 10 minutes"
        else
            error "Claude Code transcript processing failed. Check log at /tmp/claude-processing.log"
        fi
        cat /tmp/claude-processing.log || true
    fi
}

# Organize processed transcripts
organize_transcripts() {
    log "Organizing processed transcripts by channel..."
    
    cd "$PROJECT_DIR"
    
    if uv run python main.py process; then
        success "Transcripts organized successfully"
    else
        error "Failed to organize transcripts"
    fi
}

# Main workflow function
run_workflow() {
    log "=== Starting AI Workflow Automation ==="
    log "Implementing CH-141: Claude Code integration for automated transcript processing"
    
    check_prerequisites
    start_monitoring
    
    log "Docker container is now monitoring channels and downloading transcripts..."
    log "Waiting 30 seconds for initial downloads, then processing with Claude Code..."
    
    # Wait for some downloads to occur
    sleep 30
    
    # Process any available transcripts
    process_transcripts
    organize_transcripts
    
    success "=== AI Workflow completed successfully ==="
    log "The Docker container continues monitoring. Run this script periodically to process new downloads."
}

# Script usage
usage() {
    echo "Usage: $0 [COMMAND]"
    echo ""
    echo "Commands:"
    echo "  start     - Start the full AI workflow (monitoring + processing)"
    echo "  process   - Only process downloaded transcripts with Claude Code"
    echo "  status    - Check the status of Docker container and Claude Code"
    echo "  stop      - Stop the Docker container"
    echo "  help      - Show this help message"
    echo ""
    echo "Default: start"
}

# Status check
check_status() {
    log "=== AI Workflow Status ==="
    
    cd "$PROJECT_DIR"
    
    # Check Docker status
    if docker-compose ps | grep -q "Up"; then
        success "Docker container: Running"
    else
        warning "Docker container: Not running"
    fi
    
    # Check Claude Code
    CLAUDE_CMD="claude"
    if command -v claude-code &> /dev/null; then
        CLAUDE_CMD="claude-code"
    fi
    
    if "$CLAUDE_CMD" --version &> /dev/null; then
        success "Claude Code: Available and authenticated"
    else
        warning "Claude Code: Not available or not authenticated"
    fi
    
    # Check for pending transcripts
    STORAGE_INFO=$(uv run python main.py config 2>/dev/null | grep "base_path" || echo "")
    if [[ -n "$STORAGE_INFO" ]]; then
        BASE_PATH=$(echo "$STORAGE_INFO" | sed 's/.*: //' | tr -d '"')
        DOWNLOADED_DIR="$BASE_PATH/downloaded"
        
        if [[ -d "$DOWNLOADED_DIR" ]]; then
            FILE_COUNT=$(find "$DOWNLOADED_DIR" -name "*.md" -type f | wc -l)
            if [[ "$FILE_COUNT" -gt 0 ]]; then
                log "Pending transcripts: $FILE_COUNT files ready for processing"
            else
                log "Pending transcripts: None"
            fi
        fi
    fi
}

# Stop monitoring
stop_monitoring() {
    log "Stopping Docker container..."
    
    cd "$PROJECT_DIR"
    
    if docker-compose down; then
        success "Docker container stopped"
    else
        error "Failed to stop Docker container"
    fi
}

# Main script logic
case "${1:-start}" in
    start)
        run_workflow
        ;;
    process)
        check_prerequisites
        process_transcripts
        organize_transcripts
        ;;
    status)
        check_status
        ;;
    stop)
        stop_monitoring
        ;;
    help|--help|-h)
        usage
        ;;
    *)
        echo "Unknown command: $1"
        usage
        exit 1
        ;;
esac