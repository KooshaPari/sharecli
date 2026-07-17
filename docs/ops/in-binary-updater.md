# In-binary updater (soft) — sharecli

Audit-v38 **C11 L111** (auto-update). External operator channels are documented in
[`auto-update.md`](auto-update.md) (crates.io / binstall / brew / checksummed
Releases). This runbook is the **soft plan** for in-process self-update and
desktop appcast paths — **no `sharecli self-update` or Sparkle/WinUI updater
ships in this lane**; hard wiring waits on L112 signing secrets and a TUF
metadata pipeline sketch below.

**Prerequisite:** [`codesign-notarize.md`](codesign-notarize.md) (L112) for
signature-verified updates per rubric acceptance.

## Current stance

| Surface | Today | In-binary plan |
|---------|-------|----------------|
| CLI (`sharecli`) | Manual replace via release tarball / `cargo install` | `sharecli self-update` (deferred) |
| macOS tray (`ShareCLITray.app`) | Manual `.tar.gz` + sha256 | Sparkle 2 appcast (deferred) |
| Windows tray (WinUI) | Manual zip + sha256 | App installer / WinGet + optional in-app check (deferred) |
| Linux tray (SNI) | Manual tarball | Package manager or `self-update` only (deferred) |
| Update trust | `.sha256` sidecar on GH Releases | TUF targets + codesign (L112) + SLSA provenance cross-check (C06) |

Until this plan graduates, operators MUST follow [`auto-update.md`](auto-update.md).

## Soft contract (Phase 0 — this doc)

| Step | Action | Status |
|------|--------|--------|
| 1 | Document external channels | **Done** — [`auto-update.md`](auto-update.md) |
| 2 | Sketch in-binary surfaces + TUF roles | **Done** — this doc |
| 3 | Prototype `self-update --check-only` (no install) | **Next** |
| 4 | Signed metadata + replace binary | **Blocked** — L112 |
| 5 | Tray Sparkle / WinUI appcast | **Blocked** — L112 + tray hardening |

## Proposed update surfaces

### CLI — `sharecli self-update`

| Concern | Sketch |
|---------|--------|
| Discovery | `GET` release metadata or TUF `targets.json` for current triple |
| Download | Stream to temp file; verify hash + signature before swap |
| Install | Atomic rename beside current exe; Windows: schedule replace on exit |
| Rollback | Keep previous binary as `sharecli.old` until health check passes |
| Channels | `--channel stable\|beta` maps to TUF delegations (future) |

Local probe (future): `sharecli self-update --check-only` prints latest version
and whether the running binary is current — no write until Phase 4.

### Desktop — Sparkle (macOS) / WinUI (Windows)

| Host | Mechanism | Notes |
|------|-----------|-------|
| macOS tray | Sparkle 2 + EdDSA or Apple signing | Appcast XML hosted beside releases; delta updates after baseline |
| Windows tray | MSIX / WiX minor update or custom appcast | Prefer `.msi` from [`dmg-msi-packaging.md`](dmg-msi-packaging.md) before in-app delta |
| Linux tray | No Sparkle equivalent | Rely on `self-update` or distro package; see [`auto-update.md`](auto-update.md) |

Tray updaters MUST NOT ship before [`win-tray-hardening.md`](win-tray-hardening.md)
Phase 3 (green CI) to avoid updating a known-soft binary.

## TUF metadata sketch (Phase 1 — not required)

Use [The Update Framework (TUF)](https://theupdateframework.io/) so clients
verify metadata before downloading artifacts. Supply-chain overlap with C06
(SLSA/cosign) is intentional: TUF governs **update discovery**; SLSA
provenance governs **build integrity**.

```
sharecli-update/
  metadata/
    root.json          # offline root; pins targets/snapshot keys
    targets.json       # version → per-triple artifact hashes + download URLs
    snapshot.json      # pins targets version
    timestamp.json     # short-lived freshness (online key)
  targets/
    sharecli-x86_64-unknown-linux-gnu
    sharecli-aarch64-apple-darwin
    ...
```

| Role | Holder | Rotation |
|------|--------|----------|
| Root | Maintainer offline / HSM | Rare; documents targets + snapshot keys |
| Targets | Release CI (`release.yml`) | Each tag publish |
| Snapshot | Release CI | Each metadata publish |
| Timestamp | Online service or GH Pages cron | Hours–days TTL |

**Verification flow (client):**

1. Fetch `timestamp.json` → verify → fetch `snapshot.json` → `targets.json`.
2. Resolve current OS/arch target; compare version to running binary.
3. Download artifact; verify TUF-listed hash.
4. Verify OS code signature (L112) or minisign/cosign where applicable.
5. Optional: verify SLSA provenance from [`slsa.md`](../slsa.md) matches commit + digest.

**Publishing (CI sketch — deferred):**

```bash
# Pseudocode — not wired
tuf-cli init --dir sharecli-update
tuf-cli targets add sharecli-x86_64-unknown-linux-gnu@v0.4.0 \
  --sha256 "$(sha256sum target/release/sharecli)" \
  --url "https://github.com/KooshaPari/sharecli/releases/download/v0.4.0/..."
tuf-cli snapshot
tuf-cli timestamp
# Upload metadata/ to gh-pages or release asset bundle
```

**Policy (deferred):** hosted metadata URL; key ceremony; whether root ships
embedded in binary vs first-run bootstrap; integration with Sigstore for
timestamp role.

## CI phases

| Phase | Gate | Status |
|-------|------|--------|
| 1 — external channels | [`auto-update.md`](auto-update.md) + deploy link | **Done** |
| 2 — soft plan | this doc + cross-refs | **Done** (soft) |
| 3 — check-only CLI | `self-update --check-only` + unit tests (no install) | **Next** |
| 4 — TUF publish job | `release.yml` metadata attach (unsigned OK for dogfood) | **Deferred** |
| 5 — signed install path | L112 + atomic replace + tray appcast | **Blocked** |

Phase 5 intentionally **does not** embed certificate values; follow
[`codesign-notarize.md`](codesign-notarize.md).

## Hard follow-up (out of scope)

- Required auto-update on by default (opt-in only until L112 proven)
- Delta/binary patches and staged rollouts (`--channel beta`)
- Rollback automation beyond `sharecli.old` retention
- WinGet / Homebrew cask in-app elevation flows

Soft goal: **L111 stays partial** with agent-readable in-binary + TUF path;
today's operator contract remains [`auto-update.md`](auto-update.md).

## Operator checklist

1. Use external channels in [`auto-update.md`](auto-update.md) for production today.
2. Verify `.sha256` (and later signatures) before replacing any binary.
3. Do not enable experimental `self-update` until Phase 4 documents verify steps.
4. After L112: require signed/notarized artifacts for in-binary install paths.
5. Cross-check release digest against SLSA provenance when consuming GH Releases.

## Cross-refs

- External channels (current): [`auto-update.md`](auto-update.md)
- Signing (L112): [`codesign-notarize.md`](codesign-notarize.md)
- Native installers: [`dmg-msi-packaging.md`](dmg-msi-packaging.md) (L108)
- Win tray hardening: [`win-tray-hardening.md`](win-tray-hardening.md) (L110)
- SLSA / cosign: [`slsa.md`](../slsa.md) (C06)
- Deploy matrix: [`deploy.md`](../deploy.md)

**Status:** soft plan (Phase 0) · **FR:** FR-003 traceability · **Last sync:** 2026-07-17
