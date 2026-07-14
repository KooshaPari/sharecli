# Code signing & notarization (soft)

Audit-v38 **C11 L112**. Release archives today ship **unsigned** with `.sha256`
checksums (`release.yml` `github-release`). This runbook is the soft contract
until Apple/Windows signing secrets land.

## Current stance

| Platform | Status | Notes |
|----------|--------|-------|
| macOS Developer ID + notarize + staple | Blocked | Needs org Apple Developer cert + `APPLE_*` secrets |
| Windows Authenticode (`signtool`) | Blocked | Needs Authenticode cert + CI secret |
| Linux | N/A for Gatekeeper | Cosign/SLSA cover supply-chain (C06); not OS code-sign |
| Ad-hoc `codesign --sign -` | Dev only | Never for release assets |

## Required secrets (when unblocked)

| Secret | Use |
|--------|-----|
| `APPLE_CERTIFICATE_BASE64` | Developer ID Application .p12 |
| `APPLE_CERTIFICATE_PASSWORD` | Unlock p12 |
| `APPLE_API_KEY` / `APPLE_API_ISSUER` / `APPLE_API_KEY_ID` | `notarytool` |
| `WINDOWS_CERT_PFX_BASE64` / `WINDOWS_CERT_PASSWORD` | Authenticode |

## Soft CI

`.github/workflows/codesign-soft.yml` asserts this doc exists and reports when
signing secrets are absent (`continue-on-error`). It does **not** claim
notarized artifacts.

## Follow-up (hard gate)

1. Add `codesign` / `notarytool` / `signtool` steps to `release.yml`.
2. Staple macOS archives; attach signed Windows binaries.
3. Flip `docs/deploy.md` Releases row from UNSIGNED → signed+notarized.
4. Unblock W4.3 in `docs/ops/governance/WBS-PHASED.md`.
