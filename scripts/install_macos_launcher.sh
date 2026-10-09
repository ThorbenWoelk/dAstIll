#!/bin/zsh
set -euo pipefail

repo_root=${0:A:h:h}
app_root="$HOME/Applications/dAstIll Local.app"
mkdir -p "$HOME/Applications"
build_dir=$(mktemp -d "$HOME/Applications/.dastill-local.XXXXXX")
trap 'rm -rf "$build_dir"' EXIT

candidate="$build_dir/dAstIll Local.app"
contents="$candidate/Contents"
mkdir -p "$contents/MacOS" "$contents/Resources"

echo "Building dAstIll Local for macOS..."
architecture=$(uname -m)
xcrun swiftc \
  -parse-as-library \
  -O \
  -target "${architecture}-apple-macos13.0" \
  "$repo_root/macos/DastillLauncher.swift" \
  -o "$contents/MacOS/dastill-local"

cp "$repo_root/src-tauri/icons/icon.icns" "$contents/Resources/dastill.icns"

python3 - "$contents/Info.plist" "$repo_root" <<'PY'
import plistlib
import sys

with open(sys.argv[1], "wb") as plist:
    plistlib.dump(
        {
            "CFBundleDevelopmentRegion": "en",
            "CFBundleDisplayName": "dAstIll Local",
            "CFBundleExecutable": "dastill-local",
            "CFBundleIconFile": "dastill.icns",
            "CFBundleIdentifier": "local.dastill.launcher",
            "CFBundleInfoDictionaryVersion": "6.0",
            "CFBundleName": "dAstIll Local",
            "CFBundlePackageType": "APPL",
            "CFBundleShortVersionString": "1.0",
            "CFBundleVersion": "1",
            "DastillRepositoryPath": sys.argv[2],
            "LSMinimumSystemVersion": "13.0",
            "NSAppTransportSecurity": {"NSAllowsLocalNetworking": True},
            "NSHighResolutionCapable": True,
        },
        plist,
    )
PY

codesign --force --sign - "$candidate" >/dev/null

hook_source="$repo_root/scripts/githooks/post-commit"
hook_target=$(git -C "$repo_root" rev-parse --path-format=absolute --git-path hooks/post-commit)
if [[ "$hook_source" != "$hook_target" ]]; then
  if [[ -f "$hook_target" ]] && ! grep -Fq '# dAstIll macOS launcher post-commit hook' "$hook_target"; then
    echo "Refusing to replace an existing post-commit hook at $hook_target" >&2
    exit 1
  fi
  mkdir -p "${hook_target:h}"
  install -m 755 "$hook_source" "$hook_target"
fi

was_running=0
if pgrep -x dastill-local >/dev/null 2>&1; then
  was_running=1
fi

if [[ -e "$app_root" ]]; then
  mv "$app_root" "$build_dir/previous.app"
fi
if ! mv "$candidate" "$app_root"; then
  if [[ -e "$build_dir/previous.app" ]]; then
    mv "$build_dir/previous.app" "$app_root"
  fi
  echo "Could not replace dAstIll Local; restored the previous app." >&2
  exit 1
fi

# The former script launcher always started the stack and then exited.
legacy_app="$HOME/Applications/dAstIll Dev.app"
if [[ -f "$legacy_app/Contents/Info.plist" ]]; then
  legacy_id=$(/usr/libexec/PlistBuddy -c 'Print CFBundleIdentifier' "$legacy_app/Contents/Info.plist" 2>/dev/null || true)
  if [[ "$legacy_id" == "local.dastill.devlauncher" ]]; then
    rm -rf "$legacy_app"
  fi
fi

touch "$app_root"
if (( was_running )); then
  pkill -x dastill-local || true
  for _ in {1..20}; do
    if ! pgrep -x dastill-local >/dev/null 2>&1; then
      break
    fi
    sleep 0.1
  done
  open -g "$app_root"
fi

echo "Installed $app_root"
echo "Installed post-commit hook at $hook_target"
echo "Open dAstIll Local from Spotlight, Launchpad, or Applications."
