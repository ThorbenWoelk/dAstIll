---
description: deploy changes from feature branch to main using Graphite
---

1. Sync workspace
```bash
gt sync --no-interactive
```

2. Stage all changes
```bash
gt add .
```

3. Update current branch or create new one
```bash
gt modify --no-edit --no-interactive || gt branch create -m "feat: deploy changes" --no-interactive
```

4. Submit stack to Graphite (creates/updates PRs)
```bash
gt submit --stack --no-edit --publish --no-interactive
```

5. Merge the stack to main
```bash
gt merge --no-interactive
```

6. Final sync and cleanup
```bash
gt sync --no-interactive
```
