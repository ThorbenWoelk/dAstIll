# Tasks: dAstIll Dev Launcher

## Current State
`dAstIll Dev.app` installs into `~/Applications` with custom `.icns` icon and successfully launches the local frontend/backend via `start_app.sh`.

## Steps
- [x] Create spec and tasks tracking files for dev launcher feature
- [x] Add installer script that builds `~/Applications/dAstIll Dev.app`
- [x] Add custom icon asset and `.icns` generation path
- [x] Run installer and verify `.app` exists with icon
- [x] Launch app and verify local stack + URL open behavior

## Decisions Made During Implementation
- Use a user-level app install location (`~/Applications`) to avoid sudo.
- Generate `.icns` from an SVG asset via macOS built-ins (`sips`, `iconutil`).
- Launch app process detached via `nohup ./start_app.sh` so Finder launch does not block.
