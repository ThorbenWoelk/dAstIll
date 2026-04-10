#!/bin/zsh
set -euo pipefail

script_dir="$(cd "$(dirname "$0")" && pwd)"
repo_root="$(cd "$script_dir/.." && pwd)"

source_icon="$repo_root/frontend/static/icon-512.png"
target_dir="$repo_root/src-tauri/icons"
android_res_dir="$repo_root/src-tauri/gen/android/app/src/main/res"
tmp_dir="$(mktemp -d)"
manifest_path="$(mktemp "$repo_root/.tauri-icon-manifest.XXXXXX.json")"

cleanup() {
  rm -rf "$tmp_dir"
  rm -f "$manifest_path"
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

cat > "$manifest_path" <<'JSON'
{
  "default": "frontend/static/icon-512.png",
  "bg_color": "#faf9f6"
}
JSON

(
  cd "$repo_root/src-tauri"
  cargo tauri icon "$manifest_path" -o "$tmp_dir"
)

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

if [[ -d "$android_res_dir" ]]; then
  # Keep Android on the raster launcher assets so the native home-screen icon
  # matches the web/PWA install artwork instead of Android's adaptive treatment.
  rm -f \
    "$android_res_dir/mipmap-anydpi-v26/ic_launcher.xml" \
    "$android_res_dir/values/ic_launcher_background.xml"

  for density in mdpi hdpi xhdpi xxhdpi xxxhdpi; do
    density_dir="$android_res_dir/mipmap-$density"
    mkdir -p "$density_dir"

    for file in ic_launcher.png ic_launcher_foreground.png ic_launcher_round.png; do
      cp "$tmp_dir/android/mipmap-$density/$file" "$density_dir/$file"
    done
  done

  printf "Synced Android launcher icons into %s\n" "$android_res_dir"
fi

printf "Synced Tauri icons into %s\n" "$target_dir"
