#!/usr/bin/env bash
# Stage an unsigned macOS .app layout for sharecli (C11 L108 soft — no notarize/L112).
# Produces dist/sharecli.app (Contents/{MacOS,Info.plist,Resources}) ready for create-dmg.
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
cd "$ROOT"

VERSION="${SHARECLI_DMG_VERSION:-$(grep '^version' Cargo.toml | head -1 | sed -E 's/.*"([^"]+)".*/\1/')}"
BINARY="${SHARECLI_DMG_BINARY:-target/release/sharecli}"
OUT_DIR="${SHARECLI_DMG_OUT:-dist}"
APP_NAME="${SHARECLI_DMG_APP_NAME:-sharecli.app}"
BUNDLE_ID="${SHARECLI_DMG_BUNDLE_ID:-phenotype.sharecli}"

if [[ ! -f "$BINARY" ]]; then
  echo "error: binary not found at $BINARY (pass SHARECLI_DMG_BINARY=… or build release first)" >&2
  exit 1
fi

APP_ROOT="$OUT_DIR/$APP_NAME"
MACOS_DIR="$APP_ROOT/Contents/MacOS"
RES_DIR="$APP_ROOT/Contents/Resources"

rm -rf "$APP_ROOT"
install -d "$MACOS_DIR" "$RES_DIR"
install -m 755 "$BINARY" "$MACOS_DIR/sharecli"

cat >"$APP_ROOT/Contents/Info.plist" <<EOF
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
  <key>CFBundleExecutable</key>
  <string>sharecli</string>
  <key>CFBundleIdentifier</key>
  <string>${BUNDLE_ID}</string>
  <key>CFBundleName</key>
  <string>ShareCLI</string>
  <key>CFBundleVersion</key>
  <string>${VERSION}</string>
  <key>CFBundleShortVersionString</key>
  <string>${VERSION}</string>
  <key>CFBundlePackageType</key>
  <string>APPL</string>
  <key>LSMinimumSystemVersion</key>
  <string>11.0</string>
  <key>CFBundleInfoDictionaryVersion</key>
  <string>6.0</string>
</dict>
</plist>
EOF

# Marker for soft CI / operators: layout only — create-dmg + L112 still deferred.
cat >"$APP_ROOT/UNSIGNED_SOFT.txt" <<EOF
sharecli unsigned .app layout (C11 L108 soft)
version=${VERSION}
notarize=deferred-L112
next=create-dmg on macOS runner after L112 secrets
EOF

echo "staged $APP_ROOT"
test -x "$MACOS_DIR/sharecli"
test -f "$APP_ROOT/Contents/Info.plist"
