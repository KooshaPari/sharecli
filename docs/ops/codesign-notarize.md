# Code signing & notarization (soft)

Audit-v38 **C11 L112**. Release archives today ship **unsigned** with `.sha256`
checksums (`release.yml` `github-release`). This runbook is the soft contract
until Apple/Windows signing secrets land.

## Current stance

| Platform | Status | Notes |
|----------|--------|-------|
| macOS Developer ID + notarize + staple | **Blocked** | Needs org Apple Developer cert + `APPLE_*` secrets |
| Windows Authenticode (`signtool`) | **Blocked** | Needs Authenticode cert + CI secret |
| Linux | N/A for Gatekeeper | Cosign/SLSA cover supply-chain (C06); not OS code-sign |
| Ad-hoc `codesign --sign -` | Dev only | Never for release assets |

**Secret inventory (2026-07-22):** `gh secret list -R KooshaPari/sharecli` still
returns **zero** Actions secrets. No `APPLE_*` / `WINDOWS_CERT_*` (or any other)
repo secrets are configured. Status remains **Blocked** — do not invent secrets.
Agent follow-up cannot flip L112 without the operator adding secrets in GitHub
(Settings → Secrets and variables → Actions).

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

1. Add the secrets above to the repo (or org) Actions secret store — never commit them.
2. Add `codesign` / `notarytool` / `signtool` steps to `release.yml`.
3. Staple macOS archives; attach signed Windows binaries.
4. Flip `docs/deploy.md` Releases row from UNSIGNED → signed+notarized.
5. Unblock W4.3 in `docs/ops/governance/WBS-PHASED.md`.
