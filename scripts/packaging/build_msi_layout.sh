#!/usr/bin/env bash
# Stage an unsigned Windows MSI build layout for sharecli (C11 L108 soft — no Authenticode/L112).
# Copies WiX source + payload under dist/msi-layout/ for candle/light or `wix build`.
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
cd "$ROOT"

VERSION="${SHARECLI_MSI_VERSION:-$(grep '^version' Cargo.toml | head -1 | sed -E 's/.*"([^"]+)".*/\1/')}"
BINARY="${SHARECLI_MSI_BINARY:-target/release/sharecli.exe}"
OUT_DIR="${SHARECLI_MSI_OUT:-dist/msi-layout}"
WXS_SRC="${SHARECLI_MSI_WXS:-scripts/packaging/wix/sharecli.wxs}"

if [[ ! -f "$BINARY" ]]; then
  echo "error: binary not found at $BINARY (pass SHARECLI_MSI_BINARY=… or build release first)" >&2
  exit 1
fi

if [[ ! -f "$WXS_SRC" ]]; then
  echo "error: WiX source missing at $WXS_SRC" >&2
  exit 1
fi

rm -rf "$OUT_DIR"
install -d "$OUT_DIR/payload"
install -m 755 "$BINARY" "$OUT_DIR/payload/sharecli.exe"
install -m 644 "$WXS_SRC" "$OUT_DIR/sharecli.wxs"

cat >"$OUT_DIR/UNSIGNED_SOFT.txt" <<EOF
sharecli unsigned MSI layout (C11 L108 soft)
version=${VERSION}
authenticode=deferred-L112
wix=sharecli.wxs
payload=payload/sharecli.exe
build_hint=wix build -d ShareCLIVersion=${VERSION} -d ShareCLISourceDir=payload sharecli.wxs -o sharecli.msi
EOF

cat >"$OUT_DIR/build-hints.md" <<EOF
# Unsigned MSI soft build hints

\`\`\`bash
# From dist/msi-layout/ with WiX Toolset v4+
wix build -d ShareCLIVersion=${VERSION} -d ShareCLISourceDir=payload sharecli.wxs -o sharecli_${VERSION}_x64.msi
\`\`\`

Do **not** claim GA until Authenticode (L112) lands. Soft CI only stages this layout.
EOF

echo "staged $OUT_DIR"
test -f "$OUT_DIR/sharecli.wxs"
test -f "$OUT_DIR/payload/sharecli.exe"
grep -q 'Product' "$OUT_DIR/sharecli.wxs"
