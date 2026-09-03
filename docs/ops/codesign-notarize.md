# Code signing & notarization (hard gate — Infisical-backed)

Audit-v38 **C11 L112**. macOS release binaries are signed with a Developer ID
Application cert, notarized via `notarytool`, and stapled. Apple signing secrets
are fetched from **Infisical** at CI runtime (never committed, never stored as
raw GitHub secrets). Windows Authenticode remains soft until Azure Key Vault is
provisioned.

## Current stance

| Platform | Status | Notes |
|----------|--------|-------|
| macOS Developer ID + notarize + staple | **Hard gate** | Infisical-backed Apple secrets (`codesign.yml` `macos-sign`) |
| Windows Authenticode | **Soft** (`continue-on-error`) | Azure Key Vault not configured |
| Linux | N/A for Gatekeeper | Cosign/SLSA cover supply-chain (C06); not OS code-sign |
| Ad-hoc `codesign --sign -` | Dev only | Never for release assets |

## Secret flow

`.github/workflows/codesign.yml` (`macos-sign` job):

1. Install Infisical CLI (`brew install infisical/get-cli/infisical`).
2. `infisical export --projectId … --env … --token "$INFISICAL_TOKEN"` → dotenv.
3. Load only the 5 Apple keys into `GITHUB_ENV` (GitHub masks them automatically).

| Infisical key | Use |
|---------------|-----|
| `APPLE_CERTIFICATE` | Developer ID Application `.p12` (base64) |
| `APPLE_CERTIFICATE_PWD` | `.p12` unlock password |
| `APPLE_ID` | notarytool Apple ID |
| `APPLE_TEAM_ID` | Apple Developer team ID |
| `APPLE_APP_PASSWORD` | app-specific password for notarytool |

## Required repo config (one-time)

- **Secret** `INFISICAL_TOKEN` — a Machine Identity token scoped to the project.
- **Var** `INFISICAL_PROJECT_ID` (defaults to `8efe392e-56a6-4c3c-89f9-8141183dd7e8`).
- **Var** `INFISICAL_ENV` (defaults to `prod`).

## Follow-up (Windows)

1. Provision Azure Key Vault + Authenticode cert.
2. Set `AZURE_KEY_VAULT_URL` / `_CLIENT_ID` / `_CLIENT_SECRET` / `_TENANT_ID` / `_CERTIFICATE`.
3. Flip `windows-sign` job `continue-on-error: false`.