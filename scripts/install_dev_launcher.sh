#!/bin/zsh
set -euo pipefail

script_dir="$(cd "$(dirname "$0")" && pwd)"
repo_root="$(cd "$script_dir/.." && pwd)"

app_name="dAstIll Dev.app"
app_root="$HOME/Applications/$app_name"
contents_dir="$app_root/Contents"
macos_dir="$contents_dir/MacOS"
resources_dir="$contents_dir/Resources"
launcher_bin="$macos_dir/dastill-dev"
plist_path="$contents_dir/Info.plist"

icon_png="$repo_root/assets/dastill-dev-icon.png"
icon_icns="$resources_dir/dastill-dev.icns"

mkdir -p "$HOME/Applications"
rm -rf "$app_root"
mkdir -p "$macos_dir" "$resources_dir"

cat > "$launcher_bin" <<LAUNCHER
#!/bin/zsh
set -euo pipefail

REPO_ROOT="$repo_root"
FRONTEND_URL="\${FRONTEND_URL:-http://localhost:3543}"

export PATH="/opt/homebrew/bin:/usr/local/bin:\$HOME/.cargo/bin:\$HOME/.bun/bin:\$PATH"

if [[ ! -x "\$REPO_ROOT/start_app.sh" ]]; then
  osascript -e 'display alert "dAstIll Dev" message "start_app.sh is missing or not executable." as critical'
  exit 1
fi

cd "\$REPO_ROOT"
nohup ./start_app.sh > start.log 2>&1 &
open "\$FRONTEND_URL"
LAUNCHER

chmod +x "$launcher_bin"

cat > "$plist_path" <<PLIST
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
  <key>CFBundleDevelopmentRegion</key>
  <string>en</string>
  <key>CFBundleDisplayName</key>
  <string>dAstIll Dev</string>
  <key>CFBundleExecutable</key>
  <string>dastill-dev</string>
  <key>CFBundleIconFile</key>
  <string>dastill-dev.icns</string>
  <key>CFBundleIdentifier</key>
  <string>local.dastill.devlauncher</string>
  <key>CFBundleInfoDictionaryVersion</key>
  <string>6.0</string>
  <key>CFBundleName</key>
  <string>dAstIll Dev</string>
  <key>CFBundlePackageType</key>
  <string>APPL</string>
  <key>CFBundleShortVersionString</key>
  <string>1.0</string>
  <key>CFBundleVersion</key>
  <string>1</string>
  <key>LSMinimumSystemVersion</key>
  <string>12.0</string>
  <key>NSHighResolutionCapable</key>
  <true/>
</dict>
</plist>
PLIST

if [[ -f "$icon_png" ]]; then
  tmp_dir="$(mktemp -d)"
  trap 'rm -rf "$tmp_dir"' EXIT
  base_png="$tmp_dir/base.png"
  iconset_dir="$tmp_dir/dastill.iconset"

  cp "$icon_png" "$base_png"
  mkdir -p "$iconset_dir"

  sips -z 16 16 "$base_png" --out "$iconset_dir/icon_16x16.png" >/dev/null
  sips -z 32 32 "$base_png" --out "$iconset_dir/icon_16x16@2x.png" >/dev/null
  sips -z 32 32 "$base_png" --out "$iconset_dir/icon_32x32.png" >/dev/null
  sips -z 64 64 "$base_png" --out "$iconset_dir/icon_32x32@2x.png" >/dev/null
  sips -z 128 128 "$base_png" --out "$iconset_dir/icon_128x128.png" >/dev/null
  sips -z 256 256 "$base_png" --out "$iconset_dir/icon_128x128@2x.png" >/dev/null
  sips -z 256 256 "$base_png" --out "$iconset_dir/icon_256x256.png" >/dev/null
  sips -z 512 512 "$base_png" --out "$iconset_dir/icon_256x256@2x.png" >/dev/null
  sips -z 512 512 "$base_png" --out "$iconset_dir/icon_512x512.png" >/dev/null
  sips -z 1024 1024 "$base_png" --out "$iconset_dir/icon_512x512@2x.png" >/dev/null

  if command -v iconutil >/dev/null 2>&1; then
    iconutil -c icns "$iconset_dir" -o "$icon_icns"
  fi
fi

# Ensure macOS refreshes metadata for Finder and Dock.
touch "$app_root"

printf "Installed %s\n" "$app_root"
printf "Launch it from Finder, Spotlight, or Dock.\n"
