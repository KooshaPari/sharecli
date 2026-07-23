#!/usr/bin/env bash
# Soft CI assert for unsigned dmg/msi scaffolds (C11 L108) — no codesign secrets.
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
cd "$ROOT"

STUB_DIR="$(mktemp -d)"
trap 'rm -rf "$STUB_DIR"' EXIT

# Stub binaries (layouts only — not runnable product artifacts).
printf '#!/bin/sh\necho sharecli-stub\n' >"$STUB_DIR/sharecli"
chmod +x "$STUB_DIR/sharecli"
printf 'MZ-sharecli-stub' >"$STUB_DIR/sharecli.exe"

export SHARECLI_DMG_BINARY="$STUB_DIR/sharecli"
export SHARECLI_DMG_OUT="$STUB_DIR/dist-dmg"
export SHARECLI_MSI_BINARY="$STUB_DIR/sharecli.exe"
export SHARECLI_MSI_OUT="$STUB_DIR/dist-msi"

bash scripts/packaging/build_dmg_layout.sh
bash scripts/packaging/build_msi_layout.sh

test -x "$SHARECLI_DMG_OUT/sharecli.app/Contents/MacOS/sharecli"
test -f "$SHARECLI_DMG_OUT/sharecli.app/Contents/Info.plist"
grep -q 'phenotype.sharecli' "$SHARECLI_DMG_OUT/sharecli.app/Contents/Info.plist"
test -f "$SHARECLI_DMG_OUT/sharecli.app/UNSIGNED_SOFT.txt"

test -f "$SHARECLI_MSI_OUT/sharecli.wxs"
test -f "$SHARECLI_MSI_OUT/payload/sharecli.exe"
test -f "$SHARECLI_MSI_OUT/UNSIGNED_SOFT.txt"
grep -q 'InstallScope' "$SHARECLI_MSI_OUT/sharecli.wxs"

# cargo-dist targets cover dmg/msi host families (workspace + package metadata).
grep -q 'x86_64-pc-windows-msvc' Cargo.toml
grep -q 'aarch64-apple-darwin' Cargo.toml
grep -q 'x86_64-apple-darwin' Cargo.toml

test -f docs/ops/dmg-msi-packaging.md
grep -q 'build_dmg_layout.sh' docs/ops/dmg-msi-packaging.md
grep -q 'build_msi_layout.sh' docs/ops/dmg-msi-packaging.md

echo "assert_dmg_msi_soft: ok"
