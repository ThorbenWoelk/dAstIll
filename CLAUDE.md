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

### Task Management
- **Primary task source**: Always check `todo.md` for current tasks and priorities
- This is the central place where development tasks are tracked
- Update todo.md when completing tasks or discovering new ones

### File Management
- NEVER create files unless absolutely necessary for the goal
- ALWAYS prefer editing existing files over creating new ones
- Follow the existing stateless architecture patterns

### Testing Requirements
- Write comprehensive tests for new functionality
- Use mocking for external dependencies (RSS feeds, file system)
- Ensure tests are isolated and don't depend on external state

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

## Common Pitfalls to Avoid

1. **Don't introduce JSON databases** - use file system for state
2. **Don't require API keys** - use RSS feeds for monitoring
3. **Don't skip input sanitization** - especially for file paths
4. **Don't forget to update README.md** - it's the primary documentation

## Testing Strategy

- Unit tests for individual modules
- Integration tests for CLI commands
- Mock external dependencies (RSS feeds, file system)
- Test error conditions and edge cases
- Maintain high coverage for new functionality