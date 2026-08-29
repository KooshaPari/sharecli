#!/usr/bin/env bash
# T-990: macOS DMG packaging layout builder
# Creates a .app bundle and DMG from a release build.
set -euo pipefail

BINARY="${1:-target/release/sharecli}"
APP_NAME="ShareCLI"
VERSION="${2:-0.1.0}"
DMG_NAME="sharecli-${VERSION}-universal.dmg"

echo "[dmg] Building DMG layout from ${BINARY}"

# Create .app bundle structure
APP_DIR="${APP_NAME}.app"
mkdir -p "${APP_DIR}/Contents/MacOS"
mkdir -p "${APP_DIR}/Contents/Resources"

cp "${BINARY}" "${APP_DIR}/Contents/MacOS/${APP_NAME}"
chmod +x "${APP_DIR}/Contents/MacOS/${APP_NAME}"

# Create Info.plist
cat > "${APP_DIR}/Contents/Info.plist" <<PLIST
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
  <key>CFBundleName</key>
  <string>${APP_NAME}</string>
  <key>CFBundleIdentifier</key>
  <string>org.phenotype.sharecli</string>
  <key>CFBundleVersion</key>
  <string>${VERSION}</string>
  <key>CFBundleExecutable</key>
  <string>${APP_NAME}</string>
  <key>CFBundlePackageType</key>
  <string>APPL</string>
</dict>
</plist>
PLIST

# Create DMG if on macOS
if command -v hdiutil &>/dev/null; then
  hdiutil create \
    -volname "${APP_NAME}" \
    -srcfolder "${APP_DIR}" \
    -ov \
    -format UDZO \
    "${DMG_NAME}"
  echo "[dmg] Created ${DMG_NAME}"
  hdiutil verify "${DMG_NAME}"
  echo "[dmg] Verified ${DMG_NAME}"
else
  echo "[dmg] hdiutil not available (not macOS) — skipping DMG creation"
  echo "[dmg] Layout prepared at ${APP_DIR}"
fi

# Cleanup
rm -rf "${APP_DIR}"
echo "[dmg] Done"
