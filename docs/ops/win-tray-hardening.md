# Windows tray hardening (soft) — sharecli

Audit-v38 **C11 L110** (tray client) and **L118** (release automation). The WinUI
tray (`windows/ShareCLITray`) ships as an unsigned zip on tag (`release.yml`
`tray-windows`, `continue-on-error`). This runbook is the soft hardening
checklist before removing the soft CI gate — **no L112 signing secrets here**;
see [`codesign-notarize.md`](codesign-notarize.md) for Authenticode when org
certs land.

## Current stance

| Area | Status | Notes |
|------|--------|-------|
| Tray binary in-tree | **Done** | `windows/ShareCLITray/` WinUI 3 + `sharecli_ffi` P/Invoke |
| Release attach | **Soft** | `sharecli-tray-windows-x64.zip` + `.sha256` on tag; job `continue-on-error` |
| PR smoke | **Soft** | `desktop-builds.yml` `tray-windows` also `continue-on-error` |
| Single-instance | **Open** | Second launch can spawn duplicate IPC/tray |
| Elevation / UAC | **Open** | No explicit `requestedExecutionLevel` in manifest |
| Application manifest | **Partial** | `app.manifest` present; not yet wired via `ApplicationManifest` in csproj |
| Authenticode signing | **Blocked** | Cross-ref L112 only — [`codesign-notarize.md`](codesign-notarize.md) |

## Hardening checklist

### 1 — Single-instance (mutex)

Tray must behave like a background companion: one process, one notification area
icon, second launch focuses existing instance.

| Step | Action | Evidence target |
|------|--------|-----------------|
| 1 | Create a named mutex at startup (`Global\ShareCLITray.SingleInstance` or per-user `Local\`) | `App.xaml.cs` |
| 2 | On duplicate launch: find existing tray HWND / activate popover; exit duplicate with code 0 | `TrayWindow.xaml.cs` |
| 3 | Ensure `sharecli_ipc_start()` remains idempotent (already documented in `Interop.cs`) | `Interop.cs` |
| 4 | Add smoke: launch twice → one tray icon, one IPC listener | `desktop-builds.yml` or local `just build-tray-windows` |

**Do not** rely on `serve_lock` — that guards `sharecli serve`, not the WinUI tray.

### 2 — Elevation (run as standard user)

Tray must **not** require Administrator. IPC and dashboard open on loopback; no
machine-wide hooks that need elevation.

| Step | Action | Evidence target |
|------|--------|-----------------|
| 1 | Set `requestedExecutionLevel` to `asInvoker` in `app.manifest` | `windows/ShareCLITray/app.manifest` |
| 2 | Ban `requireAdministrator` unless a future feature documents a scoped UAC prompt | manifest review in PR |
| 3 | Install per-user (`%LOCALAPPDATA%\Programs\sharecli-tray`) until `.msi` (L108) | [`dmg-msi-packaging.md`](dmg-msi-packaging.md) |
| 4 | Document that auto-update (L111) stays checksum-based until L112 signed builds | [`auto-update.md`](auto-update.md) |

### 3 — Application manifest

WinUI needs an embedded SxS manifest for DPI and OS compatibility.

| Step | Action | Evidence target |
|------|--------|-----------------|
| 1 | Keep `app.manifest` — Windows 10+ `supportedOS`, `dpiAware` | `windows/ShareCLITray/app.manifest` |
| 2 | Wire manifest in project: `<ApplicationManifest>app.manifest</ApplicationManifest>` | `ShareCLITray.csproj` |
| 3 | Add `PerMonitorV2` DPI awareness when WinUI scaling issues appear | manifest `dpiAwareness` |
| 4 | Optional: `longPathAware` if release paths exceed `MAX_PATH` | manifest |

Today the manifest file exists but csproj only lists it as `<None>` — embedding is
the first hardening PR after this doc lands.

### 4 — Release automation (L118)

| Phase | Gate | Status |
|-------|------|--------|
| 1 — unsigned zip attach | `release.yml` `tray-windows` → `github-release` | **Done** (soft) |
| 2 — harden tray runtime | this doc + mutex + manifest + elevation | **Next** |
| 3 — remove `continue-on-error` | green `desktop-builds.yml` + `release.yml` on `windows-latest` | **Next** |
| 4 — signed tray binary | `signtool` after L112 secrets | **Blocked** — [`codesign-notarize.md`](codesign-notarize.md) |

Phase 4 intentionally **does not** duplicate the L112 secret matrix; follow the
runbook there for `WINDOWS_CERT_*` and `release.yml` wiring.

## Operator checklist

1. Prefer `just build-tray-windows` locally before reporting tray regressions.
2. Verify only one tray icon after login / autostart experiments.
3. Confirm tray runs without UAC prompt on a standard user account.
4. Download `sharecli-tray-windows-*.zip` from GitHub Releases; check `.sha256` until Authenticode ships.
5. After L112: install only signed tray builds; retire unsigned GA claim in [`deploy/FINALITY.md`](../deploy/FINALITY.md).

## Cross-refs

- Deploy matrix (tray row beta): [`deploy.md`](../deploy.md)
- OS parity floor: [`deploy/FINALITY.md`](../deploy/FINALITY.md)
- Native `.msi` path: [`dmg-msi-packaging.md`](dmg-msi-packaging.md) (L108)
- Signing (L112, secrets external): [`codesign-notarize.md`](codesign-notarize.md)
- Auto-update channels: [`auto-update.md`](auto-update.md) (L111)
