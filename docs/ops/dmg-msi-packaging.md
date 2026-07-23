# Native installer packaging (soft) — dmg / msi / deb

Audit-v38 **C11 L108** (native installers). Today `release.yml` ships **unsigned**
`.tar.gz` / `.zip` archives with `.sha256` checksums. This runbook covers soft
classic-installer scaffolds **without** wiring hard L112 signing secrets.

**Codesign dependency:** [`codesign-notarize.md`](codesign-notarize.md) (L112).
GA installers require Developer ID + notarization (macOS) and Authenticode
(Windows) before flipping deploy matrix rows from partial → proven.

## Current vs planned artifacts

| Host | Today (`release.yml`) | Soft scaffold / CI | Release path (planned) |
|------|----------------------|--------------------|------------------------|
| macOS CLI | `sharecli-{target}.tar.gz` | `build_dmg_layout.sh` → `dist/sharecli.app` | `dist/sharecli-{version}-aarch64-apple-darwin.dmg` |
| macOS tray/desktop | `sharecli-desktop-macos-*.tar.gz` | `.app` layout pattern (tray follow-on) | `dist/sharecli-desktop-macos-{version}.dmg` |
| Windows CLI | `sharecli-x86_64-pc-windows-msvc.zip` | `build_msi_layout.sh` + `wix/sharecli.wxs` | `dist/sharecli-{version}-x86_64-pc-windows-msvc.msi` |
| Windows tray | `sharecli-tray-windows-*.zip` | WiX pattern (tray payload follow-on) | `dist/sharecli-tray-windows-{version}.msi` |
| Linux CLI | `sharecli-x86_64-unknown-linux-gnu.tar.gz` | `build_deb.sh` → unsigned `.deb` | `dist/sharecli_{version}_amd64.deb` |
| Linux tray | `sharecli-tray-linux-*.tar.gz` | optional `.deb` unit + binary | `dist/sharecli-tray_{version}_amd64.deb` |

Archives remain the **parity floor** until signed installers attach on tag
(see [`deploy/FINALITY.md`](../deploy/FINALITY.md)).

## Soft layout scripts (unsigned)

| Script | Output | Notes |
|--------|--------|-------|
| [`scripts/packaging/build_deb.sh`](../../scripts/packaging/build_deb.sh) | `dist/sharecli_*_amd64.deb` | Real `dpkg-deb` artifact in CI |
| [`scripts/packaging/build_dmg_layout.sh`](../../scripts/packaging/build_dmg_layout.sh) | `dist/sharecli.app` | Stages `Contents/MacOS` + `Info.plist`; **no** `codesign` / `notarytool` |
| [`scripts/packaging/build_msi_layout.sh`](../../scripts/packaging/build_msi_layout.sh) | `dist/msi-layout/` | Copies [`wix/sharecli.wxs`](../../scripts/packaging/wix/sharecli.wxs) + `payload/sharecli.exe`; **no** `signtool` |
| [`scripts/packaging/assert_dmg_msi_soft.sh`](../../scripts/packaging/assert_dmg_msi_soft.sh) | exit 0 | Stub-binary soft CI gate (`packaging-soft.yml` job `dmg-msi-soft`) |

Local soft smoke (no secrets):

```bash
# Linux .deb (needs release binary)
cargo build --release -p sharecli && bash scripts/packaging/build_deb.sh

# macOS .app layout (binary or stub)
SHARECLI_DMG_BINARY=target/release/sharecli bash scripts/packaging/build_dmg_layout.sh

# Windows MSI layout (exe or stub)
SHARECLI_MSI_BINARY=target/release/sharecli.exe bash scripts/packaging/build_msi_layout.sh

# Soft CI assert (stubs)
bash scripts/packaging/assert_dmg_msi_soft.sh
```

After layouts land on a host with toolchains: `create-dmg` (macOS) or `wix build`
(Windows) may produce unsigned `.dmg` / `.msi`. Those still **must not** be
distributed as GA until L112 signing.

## Build toolchain (soft)

| OS | Tool | Input | Notes |
|----|------|-------|-------|
| macOS | layout script → `create-dmg` | `build_dmg_layout.sh` + release binary | Soft CI asserts layout only (Linux runner + stub) |
| Windows | layout script → WiX | `build_msi_layout.sh` + `wix/sharecli.wxs` | Soft CI asserts WiX + payload staging |
| Linux | `dpkg-deb` | `build_deb.sh` | Soft CI builds real unsigned `.deb` |

## CI phases

| Phase | Gate | Status |
|-------|------|--------|
| 1 — archives | `release.yml` matrix → tar.gz/zip + sha256 | **Done** |
| 2 — soft plan | this doc + `deploy.md` link | **Done** (soft) |
| 3 — unsigned `.deb` | `packaging-soft.yml` + `build_deb.sh` | **Done** |
| 3.5 — unsigned dmg/msi layouts | `assert_dmg_msi_soft.sh` + WiX/`.app` scaffolds | **Done** (soft; no L112) |
| 4 — signed installers | `release.yml` + L112 secrets (`codesign-notarize.md`) | **Blocked** on org certs |

Phase 4 intentionally **does not** embed certificate values or secret names beyond
the matrix in [`codesign-notarize.md`](codesign-notarize.md).

## Operator checklist

1. Prefer existing release archives or `cargo install` until signed `.dmg`/`.msi`/`.deb` attach.
2. Verify `.sha256` before replacing production binaries.
3. Soft layouts prove packaging shape only — do not ship `UNSIGNED_SOFT` markers as GA.
4. After L112 lands: distribute signed/notarized installers only; retire unsigned GA claim.
5. For Linux servers, continue OCI (`Containerfile`, L113) or tarball; unsigned `.deb` is soft CI evidence.

## Cross-refs

- Deploy matrix: [`deploy.md`](../deploy.md)
- Auto-update channels (pre-installer): [`auto-update.md`](auto-update.md) (L111)
- Traditional unit samples: [`deploy/systemd/sharecli.service`](../deploy/systemd/sharecli.service) (L115)
- FR-003 gate: `tests/c11_l108_dmg_msi_packaging.rs`
