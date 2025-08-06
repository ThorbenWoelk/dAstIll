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
        sleep 3
        
        # Show initial container logs
        log "Initial container logs:"
        docker-compose logs --tail=10 dastill-monitor || true
        
        # Check container status
        if docker-compose ps | grep -q "Up"; then
            success "Container is running and monitoring channels"
            
            # Show live logs for a few seconds to see initial activity
            log "Monitoring initial activity (showing 15 seconds of logs)..."
            timeout 15 docker-compose logs -f dastill-monitor || true
            log "Container is now running in the background. Use 'docker-compose logs -f dastill-monitor' to see ongoing activity."
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
    
    # List files to be processed
    log "Files to be processed:"
    find "$DOWNLOADED_DIR" -name "*.md" -type f -exec basename {} \; | head -10 | while read filename; do
        log "  → $filename"
    done
    if [[ $FILE_COUNT -gt 10 ]]; then
        log "  ... and $((FILE_COUNT - 10)) more files"
    fi
    
    # Use Claude Code to process transcripts
    log "Launching Claude Code for transcript processing..."
    log "This may take several minutes depending on the number and size of files..."
    
    # Determine Claude Code command
    CLAUDE_CMD="claude"
    if command -v claude-code &> /dev/null; then
        CLAUDE_CMD="claude-code"
    fi
    
    # Create automated prompt for Claude Code
    PROMPT="Use the transcript-education-curator agent to process all transcript files in $DOWNLOADED_DIR. For each .md file, transform it into a well-structured educational summary with key concepts, insights, and actionable takeaways. Replace each original file with the enhanced version."
    
    # Launch Claude Code in non-interactive mode with Task tool
    # Using --dangerously-skip-permissions and --add-dir for full automation
    # Use stdbuf to ensure real-time output buffering
    log "Starting Claude Code processing (output will appear in real-time)..."
    if echo "$PROMPT" | timeout 600 stdbuf -oL -eL "$CLAUDE_CMD" --print --dangerously-skip-permissions --add-dir "$BASE_PATH" 2>&1 | tee /tmp/claude-processing.log; then
        success "Claude Code transcript processing completed"
        log "Processing log saved to /tmp/claude-processing.log"
        
        # Show processing summary
        PROCESSED_COUNT=$(find "$DOWNLOADED_DIR" -name "*.md" -type f | wc -l)
        if [[ $PROCESSED_COUNT -eq 0 ]]; then
            success "All $FILE_COUNT files have been processed and moved"
        else
            warning "$PROCESSED_COUNT files remain in downloaded directory (may need manual review)"
        fi
    else
        EXIT_CODE=$?
        if [ $EXIT_CODE -eq 124 ]; then
            error "Claude Code processing timed out after 10 minutes"
            log "Files that may have been partially processed:"
            find "$DOWNLOADED_DIR" -name "*.md" -type f -exec basename {} \; || true
        else
            error "Claude Code transcript processing failed (exit code: $EXIT_CODE)"
            log "Last 20 lines of processing log:"
            tail -20 /tmp/claude-processing.log || true
        fi
        
        # Show which files are still pending
        REMAINING_COUNT=$(find "$DOWNLOADED_DIR" -name "*.md" -type f | wc -l)
        if [[ $REMAINING_COUNT -gt 0 ]]; then
            warning "$REMAINING_COUNT files still need processing"
        fi
        return 1
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
        
        # Show recent container activity
        log "Recent container activity (last 10 lines):"
        docker-compose logs --tail=10 dastill-monitor || true
    else
        warning "Docker container: Not running"
        log "Use 'ai-workflow start' to start the container"
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