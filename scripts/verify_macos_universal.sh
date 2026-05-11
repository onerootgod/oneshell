#!/usr/bin/env bash
set -euo pipefail

if [[ $# -lt 1 ]]; then
  echo "Usage: $0 <path-to-OneShell.app>"
  exit 1
fi

APP_PATH="$1"
EXPECTED_MIN_MACOS_VERSION="${EXPECTED_MIN_MACOS_VERSION:-12.0}"
INFO_PLIST="$APP_PATH/Contents/Info.plist"
EXECUTABLE_PATH="$APP_PATH/Contents/MacOS/OneShell"

[[ -d "$APP_PATH" ]] || { echo "App bundle missing: $APP_PATH"; exit 1; }
[[ -f "$INFO_PLIST" ]] || { echo "Info.plist missing: $INFO_PLIST"; exit 1; }
[[ -f "$EXECUTABLE_PATH" ]] || { echo "Executable missing: $EXECUTABLE_PATH"; exit 1; }

bundle_name="$(/usr/libexec/PlistBuddy -c 'Print :CFBundleName' "$INFO_PLIST")"
bundle_executable="$(/usr/libexec/PlistBuddy -c 'Print :CFBundleExecutable' "$INFO_PLIST")"
bundle_min_macos_version="$(
  /usr/libexec/PlistBuddy -c 'Print :LSMinimumSystemVersion' "$INFO_PLIST" 2>/dev/null \
    || /usr/libexec/PlistBuddy -c 'Print :MinimumOSVersion' "$INFO_PLIST" 2>/dev/null \
    || true
)"

[[ "$bundle_name" == "OneShell" ]] || {
  echo "Unexpected bundle name: $bundle_name"
  exit 1
}

[[ "$bundle_executable" == "OneShell" ]] || {
  echo "Unexpected bundle executable: $bundle_executable"
  exit 1
}

[[ "$bundle_min_macos_version" == "$EXPECTED_MIN_MACOS_VERSION" ]] || {
  echo "Unexpected minimum macOS version: ${bundle_min_macos_version:-<missing>}"
  exit 1
}

binary_arches="$(lipo -archs "$EXECUTABLE_PATH" 2>/dev/null || true)"
echo "$binary_arches" | grep -qw "arm64" || {
  echo "Missing arm64 architecture: ${binary_arches:-<unknown>}"
  exit 1
}
echo "$binary_arches" | grep -qw "x86_64" || {
  echo "Missing x86_64 architecture: ${binary_arches:-<unknown>}"
  exit 1
}

echo "Universal bundle verified:"
echo "  App: $APP_PATH"
echo "  Architectures: $binary_arches"
echo "  Minimum macOS: $bundle_min_macos_version"
