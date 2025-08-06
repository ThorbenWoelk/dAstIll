🔄 **CH-141 UPDATE**: Claude Code SDK Integration Limitations Found

## Implementation Status
✅ **SDK Installed**: Added `claude-code-sdk` to project dependencies
✅ **Integration Rewritten**: Modified claude_integration.py to use SDK
⚠️ **Limitation Discovered**: SDK designed for in-session use only

## Technical Findings

### SDK Architecture Issue
The Claude Code SDK is designed to work **within** an active Claude Code session, not as a standalone library that can be called from external Python scripts. When invoked externally:
- SDK returns system messages but no actual agent processing
- Cannot properly communicate with transcript-education-curator agent
- Requires active Claude Code CLI process context

### Alternative Approaches Needed
1. **Docker Volume Mounting**: Mount Claude CLI + auth from host into container
2. **Direct API Integration**: Use Anthropic API directly (requires separate implementation)
3. **Manual Workflow**: Keep current manual process with documentation

## Current Workaround
For now, the workflow remains semi-automated:
1. App downloads and monitors transcripts automatically
2. User manually runs Claude Code to process transcripts
3. User runs `process` command to organize files

## Next Investigation Areas
- Explore Docker compose volumes for Claude CLI mounting
- Research direct Anthropic API integration options
- Consider webhook-based integration for automated triggers