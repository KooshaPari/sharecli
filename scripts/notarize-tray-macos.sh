#!/usr/bin/env bash
# Notarize + staple a ShareCLITray.app build for distribution outside the App Store.
#
# Prerequisites:
#   - Xcode command-line tools (xcrun notarytool)
#   - An Apple Developer ID Application certificate installed in the keychain
#   - An App Store Connect API key (.p8) for notarytool authentication
#     (recommended) — OR an Apple ID + app-specific password via keychain
#   - A previously-built and code-signed .app bundle
#
# Usage:
#   ./scripts/notarize-tray-macos.sh \
#     --key-id KEY_ID \
#     --issuer ISSUER_ID \
#     --key-path /path/to/AuthKey_KEY_ID.p8 \
#     --bundle-id com.phenotype.sharecli.tray \
#     [--app-path /Applications/ShareCLITray.app] \
#     [--primary-bundle-id com.phenotype.sharecli] \
#     [--signing-identity "Developer ID Application: Your Name (TEAMID)"]
#
# Environment fallback (no flags):
#   SHARECLI_NOTARY_KEY_ID, SHARECLI_NOTARY_ISSUER, SHARECLI_NOTARY_KEY_PATH
#   SHARECLI_BUNDLE_ID (defaults to com.phenotype.sharecli.tray)
#   SHARECLI_SIGNING_IDENTITY (defaults to "-" for ad-hoc)
#
# Output:
#   Notarized + stapled .app at the same path (in place).
set -euo pipefail

REPO_ROOT="$(cd "$(dirname "$0")/.." && pwd)"
APP_NAME="ShareCLITray.app"

# ---- arg parsing -----------------------------------------------------------
KEY_ID="${SHARECLI_NOTARY_KEY_ID:-}"
ISSUER_ID="${SHARECLI_NOTARY_ISSUER:-}"
KEY_PATH="${SHARECLI_NOTARY_KEY_PATH:-}"
BUNDLE_ID="${SHARECLI_BUNDLE_ID:-com.phenotype.sharecli.tray}"
APP_PATH=""
SIGNING_IDENTITY="${SHARECLI_SIGNING_IDENTITY:--}"
PRIMARY_BUNDLE_ID=""

usage() {
  sed -n '2,28p' "$0"
  exit "${1:-0}"
}

while [[ $# -gt 0 ]]; do
  case "$1" in
    --key-id)            KEY_ID="$2"; shift 2 ;;
    --issuer)            ISSUER_ID="$2"; shift 2 ;;
    --key-path)          KEY_PATH="$2"; shift 2 ;;
    --bundle-id)         BUNDLE_ID="$2"; shift 2 ;;
    --app-path)          APP_PATH="$2"; shift 2 ;;
    --signing-identity)  SIGNING_IDENTITY="$2"; shift 2 ;;
    --primary-bundle-id) PRIMARY_BUNDLE_ID="$2"; shift 2 ;;
    -h|--help)           usage 0 ;;
    *) echo "Unknown arg: $1" >&2; usage 2 ;;
  esac
done

if [[ -z "$APP_PATH" ]]; then
  if [[ -d "/Applications/$APP_NAME" ]]; then
    APP_PATH="/Applications/$APP_NAME"
  elif [[ -d "$HOME/Applications/$APP_NAME" ]]; then
    APP_PATH="$HOME/Applications/$APP_NAME"
  else
    echo "No .app found in /Applications or ~/Applications; pass --app-path" >&2
    exit 2
  fi
fi

if [[ ! -d "$APP_PATH" ]]; then
  echo "App bundle not found at $APP_PATH" >&2
  exit 2
fi

if [[ -z "$KEY_ID" || -z "$ISSUER_ID" || -z "$KEY_PATH" ]]; then
  echo "Missing notarization credentials. Pass --key-id, --issuer, --key-path" >&2
  echo "(or set SHARECLI_NOTARY_KEY_ID / SHARECLI_NOTARY_ISSUER / SHARECLI_NOTARY_KEY_PATH)" >&2
  exit 2
fi

if [[ ! -f "$KEY_PATH" ]]; then
  echo "Key file not found: $KEY_PATH" >&2
  exit 2
fi

echo "==> Notarizing $APP_PATH"
echo "    Bundle ID:  $BUNDLE_ID"
echo "    Signing ID: $SIGNING_IDENTITY"
echo "    Key ID:     $KEY_ID"
echo "    Issuer:     $ISSUER_ID"

# ---- step 1: deep sign the embedded dylib ---------------------------------
FFI_DYLIB="$APP_PATH/Contents/Frameworks/libsharecli_ffi.dylib"
if [[ -f "$FFI_DYLIB" ]]; then
  echo "==> Signing dylib: $FFI_DYLIB"
  codesign --force --sign "$SIGNING_IDENTITY" \
           --options runtime --timestamp \
           "$FFI_DYLIB"
fi

# ---- step 2: deep sign the .app bundle -------------------------------------
echo "==> Signing app bundle: $APP_PATH"
codesign --force --deep --sign "$SIGNING_IDENTITY" \
         --options runtime --timestamp \
         --entitlements "$REPO_ROOT/desktop/ShareCLITray/Sources/ShareCLITray/ShareCLITray.entitlements" \
         "$APP_PATH" 2>/dev/null || \
codesign --force --deep --sign "$SIGNING_IDENTITY" \
         --options runtime --timestamp \
         "$APP_PATH"

# ---- step 3: zip into a temporary submission bundle ------------------------
TMPDIR=$(mktemp -d -t sharecli-notary.XXXXXX)
trap 'rm -rf "$TMPDIR"' EXIT
ZIP_PATH="$TMPDIR/$APP_NAME.zip"
echo "==> Zipping for submission: $ZIP_PATH"
/usr/bin/ditto -c -k --keepParent "$APP_PATH" "$ZIP_PATH"

# ---- step 4: submit to notarytool -----------------------------------------
echo "==> Submitting to Apple notary service (this may take 1-5 minutes)..."
xcrun notarytool submit "$ZIP_PATH" \
  --key-id "$KEY_ID" \
  --issuer "$ISSUER_ID" \
  --key "$KEY_PATH" \
  --wait

# ---- step 5: staple the ticket back onto the .app -------------------------
echo "==> Stapling notarization ticket onto $APP_PATH"
xcrun stapler staple "$APP_PATH"
xcrun stapler validate "$APP_PATH"

# ---- step 6: gatekeeper assessment (optional but recommended) ------------
echo "==> Running Gatekeeper assessment"
spctl --assess --type execute --verbose=2 "$APP_PATH" || true

echo "✓ Done. Notarized + stapled bundle at: $APP_PATH"
echo "  Users on macOS 10.14.5+ can launch it without security warnings."
