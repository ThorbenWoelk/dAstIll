# Repo Validation Sweep

# Status
Accepted

# Context
The repository is on `main` with a broad set of existing changes across backend, frontend, docs, workflows, Terraform, and repo hygiene. Before committing and pushing the full working tree, the repo needs a full local verification sweep so compile errors, warnings, lint failures, and test regressions are resolved against the current state.

# Decision
Run the backend and frontend verification commands required by the repo guide, fix blocking issues in the working tree until the checks pass, then run the local pre-commit hook and commit the full requested diff with plain `git`.

# Consequences
This keeps the validation work scoped to the current working tree instead of reopening individual feature specs. It may require touching code outside the most recent changes when repo-wide checks expose older breakage.
