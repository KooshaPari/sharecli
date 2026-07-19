# Native installer packaging (soft) — dmg / msi / deb

Audit-v38 **C11 L108** (native installers). Today `release.yml` ships **unsigned**
`.tar.gz` / `.zip` archives with `.sha256` checksums. This runbook is the soft
plan for classic installers without wiring hard L112 signing secrets.

**Codesign dependency:** [`codesign-notarize.md`](codesign-notarize.md) (L112).
GA installers require Developer ID + notarization (macOS) and Authenticode
(Windows) before flipping deploy matrix rows from partial → proven.

## Current vs planned artifacts

| Host | Today (`release.yml`) | Planned native installer | Release path (planned) |
|------|----------------------|------------------------|------------------------|
| macOS CLI | `sharecli-{target}.tar.gz` | `.dmg` drag-to-`/Applications` or `.pkg` | `dist/sharecli-{version}-aarch64-apple-darwin.dmg` |
| macOS tray/desktop | `sharecli-desktop-macos-*.tar.gz` | `.dmg` bundling `ShareCLITray.app` | `dist/sharecli-desktop-macos-{version}.dmg` |
| Windows CLI | `sharecli-x86_64-pc-windows-msvc.zip` | WiX `.msi` (per-user or machine) | `dist/sharecli-{version}-x86_64-pc-windows-msvc.msi` |
| Windows tray | `sharecli-tray-windows-*.zip` | `.msi` for WinUI tray | `dist/sharecli-tray-windows-{version}.msi` |
| Linux CLI | `sharecli-x86_64-unknown-linux-gnu.tar.gz` | `.deb` (amd64) | `dist/sharecli_{version}_amd64.deb` |
| Linux tray | `sharecli-tray-linux-*.tar.gz` | optional `.deb` unit + binary | `dist/sharecli-tray_{version}_amd64.deb` |

Archives remain the **parity floor** until signed installers attach on tag
(see [`deploy/FINALITY.md`](../deploy/FINALITY.md)).

## Build toolchain (soft)

| OS | Tool | Input | Notes |
|----|------|-------|-------|
| macOS | `cargo dist` + `create-dmg` or `xcodebuild -create-dmg` | `[workspace.metadata.dist]` in `Cargo.toml` | CLI first; Swift tray bundled in a follow-on job |
| Windows | `cargo dist` WiX backend or `wix` crate | `x86_64-pc-windows-msvc` release dir | Tray MSI packages WinUI payload separately |
| Linux | `cargo-deb` or `nfpm` | `target/release/sharecli` | Depends: none for static binary; postinst enables optional systemd sample |

Local smoke (unsigned):

```bash
cargo dist build --artifacts=local --target=aarch64-apple-darwin   # macOS
cargo dist build --artifacts=local --target=x86_64-pc-windows-msvc # Windows
cargo deb --no-build                                               # Linux .deb
```

## CI phases

| Phase | Gate | Status |
|-------|------|--------|
| 1 — archives | `release.yml` matrix → tar.gz/zip + sha256 | **Done** |
| 2 — soft plan | this doc + `deploy.md` link | **Done** (soft) |
| 3 — unsigned installers | `packaging-soft.yml` + `scripts/packaging/build_deb.sh` | **Done** (unsigned `.deb`; dmg/msi follow) |
| 4 — signed installers | `release.yml` + L112 secrets (`codesign-notarize.md`) | **Blocked** on org certs |

Phase 4 intentionally **does not** embed certificate values or secret names beyond
the matrix in [`codesign-notarize.md`](codesign-notarize.md).

## Operator checklist

1. Prefer existing release archives or `cargo install` until `.dmg`/`.msi`/`.deb` attach.
2. Verify `.sha256` before replacing production binaries.
3. After L112 lands: distribute signed/notarized installers only; retire unsigned GA claim.
4. For Linux servers, continue OCI (`Containerfile`, L113) or tarball until `.deb` ships.

## Cross-refs

- Deploy matrix: [`deploy.md`](../deploy.md)
- Auto-update channels (pre-installer): [`auto-update.md`](auto-update.md) (L111)
- Traditional unit samples: [`deploy/systemd/sharecli.service`](../deploy/systemd/sharecli.service) (L115)
