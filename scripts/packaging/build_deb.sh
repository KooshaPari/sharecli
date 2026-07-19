#!/usr/bin/env bash
# Build an unsigned .deb for sharecli (C11 L108 phase 3 — no L112 signing).
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
cd "$ROOT"

VERSION="${SHARECLI_DEB_VERSION:-$(grep '^version' Cargo.toml | head -1 | sed -E 's/.*"([^"]+)".*/\1/')}"
BINARY="${SHARECLI_DEB_BINARY:-target/release/sharecli}"
ARCH="${SHARECLI_DEB_ARCH:-amd64}"
OUT_DIR="${SHARECLI_DEB_OUT:-dist}"

if [[ ! -f "$BINARY" ]]; then
  echo "error: release binary not found at $BINARY (run cargo build --release first)" >&2
  exit 1
fi

PKG_ROOT="$(mktemp -d)"
trap 'rm -rf "$PKG_ROOT"' EXIT

install -d "$PKG_ROOT/DEBIAN"
install -d "$PKG_ROOT/usr/bin"
install -d "$PKG_ROOT/lib/systemd/system"
install -m 755 "$BINARY" "$PKG_ROOT/usr/bin/sharecli"
install -m 0644 "$ROOT/docs/deploy/systemd/sharecli.service" "$PKG_ROOT/lib/systemd/system/sharecli.service"

cat >"$PKG_ROOT/DEBIAN/control" <<EOF
Package: sharecli
Version: ${VERSION}
Section: utils
Priority: optional
Architecture: ${ARCH}
Maintainer: Phenotype <noreply@phenotype.dev>
Description: Shared CLI process manager for multi-project agent orchestration
 Homepage: https://github.com/KooshaPari/sharecli
EOF

install -d "$OUT_DIR"
DEB_PATH="$OUT_DIR/sharecli_${VERSION}_${ARCH}.deb"
dpkg-deb --build --root-owner-group "$PKG_ROOT" "$DEB_PATH"

echo "built $DEB_PATH"
dpkg-deb -I "$DEB_PATH" | head -20
