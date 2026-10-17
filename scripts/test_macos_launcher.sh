#!/bin/zsh
set -euo pipefail

repo_root=${0:A:h:h}
test_binary=$(mktemp "${TMPDIR:-/tmp}/dastill-launcher-tests.XXXXXX")
launcher_binary=$(mktemp "${TMPDIR:-/tmp}/dastill-launcher.XXXXXX")
trap 'rm -f "$test_binary" "$launcher_binary"' EXIT

architecture=$(uname -m)
target="${architecture}-apple-macos13.0"

grep -Fq "NSWorkspace.didWakeNotification" "$repo_root/macos/DastillLauncher.swift"
grep -Fq "restartAfterWake" "$repo_root/macos/DastillLauncher.swift"
grep -Fq "Restarting after Mac wake" "$repo_root/macos/DastillLauncher.swift"

xcrun swiftc \
  -parse-as-library \
  -target "$target" \
  "$repo_root/macos/LocalStackRecovery.swift" \
  "$repo_root/macos/LocalStackRecoveryTests.swift" \
  -o "$test_binary"
"$test_binary"

xcrun swiftc \
  -parse-as-library \
  -target "$target" \
  "$repo_root/macos/LocalStackRecovery.swift" \
  "$repo_root/macos/DastillLauncher.swift" \
  -o "$launcher_binary"
echo "macOS launcher compile passed"
