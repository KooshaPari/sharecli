#!/usr/bin/env bash
# T-1010: Linux DEB packaging builder
# Creates a .deb package from a release build.
set -euo pipefail

BINARY="${1:-target/release/sharecli}"
PKG_NAME="sharecli"
VERSION="${2:-0.1.0}"
ARCH="amd64"
DEB_NAME="${PKG_NAME}_${VERSION}_${ARCH}.deb"

echo "[deb] Building DEB package from ${BINARY}"

# Create directory structure
DEB_DIR="${PKG_NAME}-deb"
rm -rf "${DEB_DIR}"
mkdir -p "${DEB_DIR}/usr/bin"
mkdir -p "${DEB_DIR}/usr/share/doc/${PKG_NAME}"
mkdir -p "${DEB_DIR}/DEBIAN"

# Copy binary
cp "${BINARY}" "${DEB_DIR}/usr/bin/${PKG_NAME}"
chmod +x "${DEB_DIR}/usr/bin/${PKG_NAME}"

# Create control file
cat > "${DEB_DIR}/DEBIAN/control" <<CTRL
Package: ${PKG_NAME}
Version: ${VERSION}
Section: utils
Priority: optional
Architecture: ${ARCH}
Maintainer: Koosh <kooshapari@gmail.com>
Homepage: https://github.com/phenotype-org/sharecli
Description: ShareCLI — lightweight agent runtime and process orchestrator
 ShareCLI manages agent processes, pools, health checks, and provides
 a dashboard, tray client, and CLI for agent lifecycle management.
CTRL

# Create DEB
dpkg-deb --build "${DEB_DIR}" "${DEB_NAME}"
echo "[deb] Created ${DEB_NAME}"

# Verify
dpkg-deb --info "${DEB_NAME}"
dpkg-deb --contents "${DEB_NAME}"
echo "[deb] Verified ${DEB_NAME}"

# Cleanup
rm -rf "${DEB_DIR}"
echo "[deb] Done"
