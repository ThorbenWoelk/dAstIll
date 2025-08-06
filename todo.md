- The bash workflow lacks proper error recovery and partial failure handling. If Claude Code fails on one file, the entire batch fails.
- remove all references of ~/.dastill and data/config folders. we don't want to use this anywhere. instead, the configs are in config/ folder
- increase Test Coverage for Critical Paths
- remove the 600 seconds timeout for Claude Code i rather imagine a flow where claude summarizes whatever is in the downloaded folder while the docker process happily downloads more transcripts. simultaneously.

