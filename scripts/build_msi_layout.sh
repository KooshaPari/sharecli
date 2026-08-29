#!/usr/bin/env bash
# T-1000: Windows MSI packaging layout builder
# Creates a WiX-based MSI from a release build.
set -euo pipefail

BINARY="${1:-target/release/sharecli.exe}"
PKG_NAME="sharecli"
VERSION="${2:-0.1.0}"
MSI_NAME="${PKG_NAME}-${VERSION}-x64.msi"

echo "[msi] Building MSI layout from ${BINARY}"

if [[ ! -f "${BINARY}" ]]; then
  echo "[msi] ERROR: Binary not found at ${BINARY}" >&2
  exit 1
fi

# Create staging directory
STAGE_DIR="${PKG_NAME}-msi"
rm -rf "${STAGE_DIR}"
mkdir -p "${STAGE_DIR}/bin"
mkdir -p "${STAGE_DIR}/doc"

cp "${BINARY}" "${STAGE_DIR}/bin/${PKG_NAME}.exe"

# Create WiX source if candle is available
if command -v candle &>/dev/null; then
  cat > "${STAGE_DIR}/${PKG_NAME}.wxs" <<WXS
<?xml version="1.0" encoding="UTF-8"?>
<Wix xmlns="http://schemas.microsoft.com/wix/2006/wi">
  <Product Id="*" Name="${PKG_NAME}" Version="${VERSION}" Language="1033" Manufacturer="Phenotype" UpgradeCode="12345678-1234-1234-1234-123456789012">
    <Package InstallerVersion="200" Compressed="yes" InstallScope="perMachine" />
    <MajorUpgrade DowngradeErrorMessage="A newer version is already installed." />
    <MediaTemplate EmbedCab="yes" />
    <Feature Id="Main">
      <ComponentRef Id="MainExe" />
    </Feature>
  </Product>
  <Fragment>
    <Directory Id="TARGETDIR" Name="SourceDir">
      <Directory Id="ProgramFilesFolder">
        <Directory Id="INSTALLFOLDER" Name="${PKG_NAME}" />
      </Directory>
    </Directory>
  </Fragment>
  <Fragment>
    <Component Id="MainExe" Directory="INSTALLFOLDER" Guid="*">
      <File Id="ExeFile" Source="${STAGE_DIR}/bin/${PKG_NAME}.exe" KeyPath="yes" />
    </Component>
  </Fragment>
</Wix>
WXS
  candle "${STAGE_DIR}/${PKG_NAME}.wxs" -o "${STAGE_DIR}/" && light "${STAGE_DIR}/${PKG_NAME}.wixobj" -o "${MSI_NAME}"
  echo "[msi] Created ${MSI_NAME}"
else
  echo "[msi] WiX toolset (candle/light) not available — staging only"
  echo "[msi] Layout prepared at ${STAGE_DIR}/bin/${PKG_NAME}.exe"
fi

# Cleanup staging
rm -rf "${STAGE_DIR}"
echo "[msi] Done"
