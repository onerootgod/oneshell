#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "$0")/.." && pwd)"
TARGET_DIR="$ROOT_DIR/src-tauri/target/universal-apple-darwin/release/bundle"

if [[ "$(uname -s)" != "Darwin" ]]; then
  echo "This script must run on macOS."
  exit 1
fi

for tool in xcodebuild lipo rustup npm; do
  if ! command -v "$tool" >/dev/null 2>&1; then
    echo "Required tool missing: $tool"
    exit 1
  fi
done

echo "==> Installing required Rust macOS targets"
rustup target add aarch64-apple-darwin x86_64-apple-darwin

cd "$ROOT_DIR"

if [[ ! -d node_modules ]]; then
  echo "==> Installing Node dependencies"
  npm install
fi

echo "==> Building universal macOS app + DMG"
npm run tauri:build:universal

APP_PATH="$(find "$TARGET_DIR" -type d -name 'OneShell.app' | head -n 1)"

if [[ -z "$APP_PATH" ]]; then
  echo "Universal app bundle not found under $TARGET_DIR"
  exit 1
fi

echo "==> Verifying universal bundle"
bash "$ROOT_DIR/scripts/verify_macos_universal.sh" "$APP_PATH"

echo "==> Build completed"
echo "App: $APP_PATH"
find "$TARGET_DIR" -type f \( -name '*.dmg' -o -name '*.app.tar.gz' \) -print
