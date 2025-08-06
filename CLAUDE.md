# CLAUDE.md

This file provides guidance to Claude Code and other AI agents when working on this repository.

## Project Context

dAstIll is a Python CLI tool for YouTube transcript management with automatic channel monitoring. It uses a stateless file-based architecture where the file system serves as the single source of truth for video status.

## Development Environment

### Package Manager and Dependencies
- Uses `uv` as package manager (not pip/poetry)
- Run commands with: `uv run python main.py <args>`
- Install dependencies: `uv sync`

### Testing
- Use pytest: `uv run python -m pytest`
- Tests are in `/tests/` directory
- Aim for high test coverage on new functionality

### Code Architecture Principles
- **Stateless Design**: File system is single source of truth, no JSON databases
- **Separation of Concerns**: Each module handles specific functionality
- **Configuration-Driven**: Behavior controlled through config files
- **Security-First**: Input sanitization and path traversal protection

## Key Development Commands

```bash
# Development workflow
uv sync                                    # Install dependencies
uv run python main.py <command>          # Run application
uv run python -m pytest                  # Run tests
uv run python -m pytest tests/test_*.py -v  # Run specific tests

# Application testing
./main.py download <youtube-url>          # Test transcript download
./main.py channel add "Test" "@test"     # Test channel monitoring
./main.py monitor status                 # Check monitoring state
```

## File System Architecture

The application uses a four-status file-based system:

```
/base_path/
├── to_be_downloaded/    # Empty placeholder files (queued)
├── downloaded/          # Downloaded transcripts awaiting processing
├── [channel-name]/      # Processed transcripts organized by channel
└── unknown/             # Processed videos with unknown channel

~/.dastill/
├── config.json         # Main application configuration
└── channels.json       # Channel monitoring configuration
```

**Important**: The actual transcript storage locations are configured in `~/.dastill/config.json` under `storage.base_path`. Use `uv run python main.py config` to check the current configuration. For testing AI integration or finding existing transcripts, always use the configured base path, not the local `./data/` directory.

## Core Modules

1. **main.py**: CLI interface with subcommands
2. **transcript_loader.py**: Core transcript fetching and processing
3. **file_manager.py**: Stateless video file management
4. **transcript_formatter.py**: Markdown formatting and file organization
5. **config.py**: Configuration management
6. **rss_monitor.py**: RSS-based channel monitoring (no API keys)
7. **channel_config.py**: Channel monitoring configuration
8. **monitoring_service.py**: Orchestrates automatic video detection and processing

## Development Rules

### Verification Requirements
- **NEVER claim feature completion without demonstration**: All features must be tested and verified to work as intended before marking complete
- **Test end-to-end workflows**: Don't just test components in isolation - verify the complete user workflow functions properly
- **Provide evidence**: Show actual output, file contents, or behavior that proves the feature works correctly
- **Fail fast on false positives**: If tests appear to pass but functionality is broken, immediately investigate and fix the root cause
- **NO MANUAL WORKAROUNDS**: Features must work through the intended automated workflow, not through manual intervention or external processes
- **VERIFY INTEGRATION**: Ensure new features integrate seamlessly with existing workflows rather than requiring separate manual steps

### Task Management
- **Primary task source**: Always check `todo.md` for current tasks and priorities
- This is the central place where development tasks are tracked
- **todo.md format**: Pure task list - each line is a task, completed tasks are REMOVED (not marked as done)
- No boilerplate, status sections, or explanatory text in todo.md - only actionable tasks

### File Management
- NEVER create files unless absolutely necessary for the goal
- ALWAYS prefer editing existing files over creating new ones
- Follow the existing stateless architecture patterns

### Testing Requirements
- Write comprehensive tests for new functionality
- **NEVER hit external APIs in tests**: All external dependencies (YouTube API, RSS feeds, HTTP requests) must be mocked
- Use mocking for external dependencies (RSS feeds, file system, API calls)
- Ensure tests are isolated and don't depend on external state
- Tests must be deterministic and not dependent on network conditions
- If a test needs to call real APIs, it doesn't belong in the test suite - remove it
- **ALWAYS run tests before pushing**: `uv run python -m pytest` must pass before `git push`

### Security Considerations
- Sanitize all user inputs, especially file paths and channel names
- Prevent path traversal attacks in file operations
- Validate video IDs and channel handles before processing

### Code Style
- Follow existing patterns in the codebase
- Use type hints consistently
- Handle exceptions gracefully with user-friendly error messages

## Documentation Rules

### README.md (Primary Documentation)
- README.md is the authoritative documentation for users and developers
- Always update README.md when adding new features or changing behavior
- Include comprehensive usage examples and architecture overview

### CLAUDE.md (AI Agent Guidance)
- This file is specifically for AI agents working on the codebase
- Focus on development rules, architecture context, and workflow guidance
- Do not duplicate user-facing documentation from README.md

### Documentation Style
- **Avoid temporal language** that becomes outdated:
  - ❌ "NEW feature", "Recently added", "Latest update"
  - ✅ "Feature", "The application includes", "Available functionality"
- Write for long-term validity until features actually change
- Never reference issue numbers or PRs in user documentation

## Common Development Patterns

### Adding New CLI Commands
1. Add argument parser in `main.py`
2. Create handler function following existing pattern
3. Update README.md with usage examples
4. Add comprehensive tests

### Extending File Operations
1. Use `VideoFileManager` for all file status operations
2. Follow four-status system: not_downloaded → to_be_downloaded → downloaded → processed
3. Sanitize all file paths and names
4. Update tests to cover new file operations

### Adding Monitoring Features
1. Use `ChannelConfigManager` for configuration
2. Extend `MonitoringService` for new monitoring logic
3. Use callback patterns for event handling
4. Test with mocked RSS feeds and file operations

## Integration Points

### External Dependencies
- YouTube Transcript API: For fetching transcripts
- RSS Feeds: For channel monitoring (no API key required)
- File System: Primary data storage (stateless design)

### Configuration System
- Main config: `~/.dastill/config.json`
- Channel config: `~/.dastill/channels.json`
- Both use atomic writes for safety

## AI Agent Integration

### Available Claude Code Agents
The project has access to specialized Claude Code agents for different tasks:

1. **transcript-education-curator** - Primary agent for this project
   - Analyzes YouTube transcripts and transforms them into educational summaries
   - Extracts key concepts, insights, and actionable takeaways
   - Used in automated workflow: `claude --print transcript-education-curator`

2. **git-agent** - For version control workflows
   - Creates conventional commit messages following v1.0.0 specification
   - Handles proper branching workflows with PR creation
   - Usage: For commit message automation and git workflow management

3. **linear-product-owner** - For project management
   - Manages projects and tasks in Linear
   - Creates/organizes projects, manages issues, maintains structure
   - Usage: Project planning and issue tracking

4. **obsidian-journal-writer** - For documentation
   - Creates journal entries in Obsidian
   - Documents experiences and reflections
   - Usage: Development journaling and note-taking

5. **ai-app-architect** - For system design
   - Helps design AI-powered applications
   - System architecture, model selection, integration patterns
   - Usage: Technical architecture decisions

6. **general-purpose** - For research and complex tasks
   - General-purpose agent for complex, multi-step tasks
   - Code searching and research
   - Usage: When other specialized agents don't fit the task

### CH-141: Bash Workflow Automation ✅ (COMPLETED)
The application now includes **fully automated AI processing** through bash script orchestration with zero user interaction required:

#### Architecture
- **Docker Container**: Runs dAstIll monitoring as a long-running process
- **Bash Script Orchestration**: `scripts/ai-workflow.sh` coordinates Docker + Claude Code
- **Claude Code CLI**: Processes transcripts with transcript-education-curator agent in non-interactive mode
- **Automated Organization**: Files are organized by channel after AI processing

#### Key Automation Breakthrough: Non-Interactive Claude Code
**Problem Solved**: Claude Code CLI was requiring user prompts, breaking full automation.

**Solution Implemented**: 
```bash
# Before (interactive - broke automation)
claude < prompt_file

# After (fully automated)
claude --print --dangerously-skip-permissions --add-dir "$BASE_PATH" "$PROMPT"
```

**Critical Flags for Automation**:
- `--print`: Non-interactive output mode
- `--dangerously-skip-permissions`: Bypass all permission prompts
- `--add-dir`: Grant directory access automatically
- `timeout 600`: Prevent hanging with 10-minute timeout
- Comprehensive error handling and logging

#### Usage (100% Automated)
```bash
# Setup configuration for Docker (one-time)
mkdir -p ./data/config && cp config/channels.json ./data/config/

# Start full AI workflow (zero user interaction)
uv run python main.py ai-workflow start

# Check status
uv run python main.py ai-workflow status

# Process only downloaded transcripts (automated)
uv run python main.py ai-workflow process

# Stop the Docker container
uv run python main.py ai-workflow stop
```

#### Processing Flow (100% Automated)
```
Channel RSS → Docker Monitor → downloaded/ → Claude AI (--print) → [channel-name]/
```

#### Key Benefits
- **100% Automated**: Absolutely zero manual intervention after setup
- **Clean Architecture**: Each tool used in its optimal environment with proper automation flags
- **Reliable**: Comprehensive error handling, timeouts, logging, and recovery mechanisms
- **Actually Works**: Leverages Claude Code's native CLI interface with proper automation flags
- **Production Ready**: Handles edge cases, timeouts, permission bypassing, and logging

#### Critical Lessons Learned
1. **Docker Configuration**: Container needs channel config copied to `./data/config/` directory
2. **Claude Code Automation**: `--print --dangerously-skip-permissions` essential for non-interactive operation
3. **Directory Access**: `--add-dir` flag required for Claude Code to access transcript directories
4. **Error Handling**: Timeout handling (124 exit code) and comprehensive logging critical for reliability
5. **Bash vs SDK**: Simple bash orchestration more reliable than complex SDK integration attempts

#### Troubleshooting AI Workflow
**Problem**: "Docker container: Not running" in status check
- **Solution**: Check if Docker daemon is running (`docker --version`)
- **Fix**: Start Docker Desktop application or run `open -a Docker`

**Problem**: "Global monitoring is disabled" in container logs
- **Solution**: Channel configuration missing in container
- **Fix**: Copy config with `mkdir -p ./data/config && cp config/channels.json ./data/config/`

**Problem**: Claude Code processing hangs or times out
- **Solution**: Use proper automation flags and timeout
- **Fix**: Ensure `--print --dangerously-skip-permissions --add-dir` flags are used with `timeout 600`

**Problem**: Permission denied errors during Claude Code processing
- **Solution**: Claude Code needs explicit directory access
- **Fix**: Add `--add-dir "$BASE_PATH"` flag to Claude Code command

**Problem**: Container constantly restarting
- **Solution**: Configuration files not properly mounted or accessible
- **Fix**: Verify `./data/config/channels.json` exists and contains valid channel data

### Legacy: Manual Processing Workflow
For manual processing without the automated workflow:

1. **Technical Processing**: The CLI application downloads transcripts and organizes them by channel
2. **Educational Processing**: External AI agents (specifically the transcript-education-curator) analyze and summarize the content

**Manual Processing Flow**: After using the transcript-education-curator agent manually, run:
```bash
uv run python main.py process
```

## Common Pitfalls to Avoid

1. **Don't introduce JSON databases** - use file system for state
2. **Don't require API keys** - use RSS feeds for monitoring
3. **Don't skip input sanitization** - especially for file paths
4. **Don't forget to update README.md** - it's the primary documentation
5. **Don't assume Docker auto-starts** - implement Docker daemon startup checks
6. **Don't use interactive Claude Code in automation** - always use `--print --dangerously-skip-permissions`
7. **Don't forget Docker config setup** - container needs `./data/config/channels.json`

## Testing Strategy

### Core Testing Principles
- **Zero external dependencies**: Never hit real APIs, RSS feeds, or external services
- **Deterministic results**: Tests must pass consistently regardless of network/system state
- **Fast execution**: Tests should run quickly without network delays
- **Isolated testing**: Each test should be completely independent

### Testing Patterns
- **Unit tests**: Test individual modules with all dependencies mocked
- **Integration tests**: Test component interactions without external calls
- **Mock everything external**: YouTube API, RSS feeds, HTTP requests, file system operations
- **Use realistic test data**: Test with valid video IDs, channel handles, but never call real APIs
- **Test error conditions**: Network failures, invalid responses, edge cases

### What NOT to Test
- Real API responses (these change and cause flaky tests)
- Network connectivity or external service availability
- Rate limiting or API quotas
- Real file system operations (use temp directories)

### Mocking Guidelines
- Mock at the service boundary (e.g., `self.loader.api = MagicMock()`)
- Provide realistic mock data that matches API response formats
- Test both success and failure scenarios with mocks
- Ensure mocks are properly isolated between tests