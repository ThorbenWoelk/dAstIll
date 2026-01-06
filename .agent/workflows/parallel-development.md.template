---
description: Workflow for parallel development using git worktrees to avoid branch conflicts
---

# Parallel Agent Development Workflow

This workflow enables multiple AI agents or developers to work on the same repository simultaneously using `git worktree`.

## The Solution: Nested Worktrees

We use a hidden directory `.worktrees/` to host isolated environments.

### 1. Create a Worktree
```bash
# git worktree add .worktrees/<folder-name> -b <new-branch-name>
git worktree add .worktrees/feature-xyz -b feature/xyz
```

### 2. Run the Agent in Isolation
**Crucial**: Open the workspace window specifically for the folder `.worktrees/feature-xyz`.

```bash
cd .worktrees/feature-xyz
uv sync
```

### 3. Cleanup
When done, simply remove the worktree from the root.

```bash
git worktree remove .worktrees/feature-xyz
```

## Important Considerations
- **Ignored Directory**: Ensure `.worktrees/` is in your `.gitignore`.
- **Port Conflicts**: Running full stacks in parallel will lead to port fighting. Use specific logic runs or distinct ports.
