#!/bin/zsh
set -euo pipefail

script_dir="$(cd "$(dirname "$0")" && pwd)"
repo_root="$(cd "$script_dir/.." && pwd)"

source_icon="$repo_root/assets/dastill-dev-icon.svg"
target_dir="$repo_root/src-tauri/icons"
tmp_dir="$(mktemp -d)"

cleanup() {
  rm -rf "$tmp_dir"
}

trap cleanup EXIT

if [[ ! -f "$source_icon" ]]; then
  printf "Source icon not found: %s\n" "$source_icon" >&2
  exit 1
fi

if ! command -v cargo >/dev/null 2>&1; then
  printf "cargo is required to generate the Tauri icon set.\n" >&2
  exit 1
fi

mkdir -p "$target_dir"

cargo tauri icon "$source_icon" -o "$tmp_dir"

files=(
  "32x32.png"
  "128x128.png"
  "128x128@2x.png"
  "icon.png"
  "icon.icns"
  "icon.ico"
  "Square107x107Logo.png"
  "Square142x142Logo.png"
  "Square150x150Logo.png"
  "Square284x284Logo.png"
  "Square30x30Logo.png"
  "Square310x310Logo.png"
  "Square44x44Logo.png"
  "Square71x71Logo.png"
  "Square89x89Logo.png"
  "StoreLogo.png"
)

for file in "${files[@]}"; do
  cp "$tmp_dir/$file" "$target_dir/$file"
done

printf "Synced Tauri icons into %s\n" "$target_dir"
