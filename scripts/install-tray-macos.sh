#!/usr/bin/env bash
# Install ShareCLITray.app to ~/Applications with bundled FFI dylib and IPC sidecar.
set -euo pipefail

REPO_ROOT="$(cd "$(dirname "$0")/.." && pwd)"
PROFILE=release
TARGET="$REPO_ROOT/target/$PROFILE"
TRAY_PKG="$REPO_ROOT/desktop/ShareCLITray"
BIN_DIR="$(cd "$TRAY_PKG" && swift build -c release --show-bin-path)"
TRAY_BIN="$BIN_DIR/ShareCLITray"
FFI_DYLIB="$TARGET/libsharecli_ffi.dylib"
IPC_BIN="$TARGET/sharecli-ipc"
APP_NAME="ShareCLITray.app"
INSTALL_DIR="${HOME}/Applications"
APP_PATH="$INSTALL_DIR/$APP_NAME"

for f in "$TRAY_BIN" "$FFI_DYLIB" "$IPC_BIN"; do
  if [[ ! -f "$f" ]]; then
    echo "Missing build artifact: $f (run 'just build-tray-macos' first)" >&2
    exit 1
  fi
done

ICNS="$REPO_ROOT/assets/icons/sharecli.icns"
if [[ -d "$REPO_ROOT/assets/icons/sharecli.iconset" && ! -f "$ICNS" ]]; then
  iconutil -c icns "$REPO_ROOT/assets/icons/sharecli.iconset" -o "$ICNS" 2>/dev/null || true
fi

rm -rf "$APP_PATH"
mkdir -p "$APP_PATH/Contents/MacOS" "$APP_PATH/Contents/Frameworks" "$APP_PATH/Contents/Resources/bin"

cat > "$APP_PATH/Contents/Info.plist" <<'PLIST'
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
  <key>CFBundleDevelopmentRegion</key>
  <string>en</string>
  <key>CFBundleExecutable</key>
  <string>ShareCLITray</string>
  <key>CFBundleIdentifier</key>
  <string>phenotype.sharecli.tray</string>
  <key>CFBundleName</key>
  <string>ShareCLI</string>
  <key>CFBundleDisplayName</key>
  <string>ShareCLI</string>
  <key>CFBundlePackageType</key>
  <string>APPL</string>
  <key>CFBundleShortVersionString</key>
  <string>0.3.0</string>
  <key>CFBundleVersion</key>
  <string>1</string>
  <key>LSMinimumSystemVersion</key>
  <string>14.0</string>
  <key>LSUIElement</key>
  <true/>
  <key>NSHighResolutionCapable</key>
  <true/>
</dict>
</plist>
PLIST

if [[ -f "$ICNS" ]]; then
  cp "$ICNS" "$APP_PATH/Contents/Resources/AppIcon.icns"
  /usr/libexec/PlistBuddy -c "Add :CFBundleIconFile string AppIcon" "$APP_PATH/Contents/Info.plist" 2>/dev/null \
    || /usr/libexec/PlistBuddy -c "Set :CFBundleIconFile AppIcon" "$APP_PATH/Contents/Info.plist"
fi

cp "$TRAY_BIN" "$APP_PATH/Contents/MacOS/ShareCLITray"
cp "$FFI_DYLIB" "$APP_PATH/Contents/Frameworks/"
cp "$IPC_BIN" "$APP_PATH/Contents/Resources/bin/sharecli-ipc"
chmod +x "$APP_PATH/Contents/MacOS/ShareCLITray" "$APP_PATH/Contents/Resources/bin/sharecli-ipc"

install_name_tool -change "@rpath/libsharecli_ffi.dylib" "@executable_path/../Frameworks/libsharecli_ffi.dylib" \
  "$APP_PATH/Contents/MacOS/ShareCLITray" 2>/dev/null || true

echo "Installed: $APP_PATH"
