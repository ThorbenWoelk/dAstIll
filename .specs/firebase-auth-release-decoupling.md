# Firebase Auth Release Decoupling

## Problem

The normal `Release` workflow currently runs `firebase deploy --only auth` on every frontend deploy. That makes regular application releases depend on Firebase Auth admin access and Firebase app listing even when no auth configuration changed. In the new `dastill` project, this extra dependency is now failing the release pipeline and blocking frontend deploys.

## Goal

Return Firebase Auth management to an explicit setup or maintenance operation rather than part of every routine release, while keeping the repository documentation aligned with that operating model.

## Requirements

- The normal `Release` workflow must no longer run Firebase Auth deployment as a prerequisite for frontend deploys.
- Frontend deploys must proceed without depending on a Firebase Auth job result.
- Deployment documentation must clearly state that Firebase Auth config is set up separately from routine releases.
- Repo task tracking must capture the change and its verification status.

## Non-Goals

- Replacing Firebase Auth or changing the app's login flow.
- Automating a new standalone Firebase Auth workflow in this scope.
- Changing Terraform-managed Firebase project, web app, or secret wiring.

## Design Considerations

- The migration bug was about establishing the correct project-local Google sign-in config for `dastill`, not about requiring auth redeploy on every app release.
- Keeping Firebase Auth out of the hot path reduces deploy coupling and avoids blocking routine frontend releases on Firebase admin permissions.
- The repo should still document where Firebase Auth config lives and when it should be deployed.

## Open Questions

- Whether Firebase Auth setup should later move into a separate manual GitHub Actions workflow or remain a local/operator command.
