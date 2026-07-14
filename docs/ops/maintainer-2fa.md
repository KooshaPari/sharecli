# Maintainer 2FA policy (soft)

Audit-v38 **C04 L36** — in-repo evidence of maintainer multi-factor authentication expectations.

## Policy

| Role | Requirement |
|------|-------------|
| Repo owners / admins (`@KooshaPari` and delegated admins) | GitHub 2FA **required** |
| Write/maintain collaborators | GitHub 2FA **required** before merge rights |
| Preferred second factor | Hardware security key (WebAuthn) or TOTP app |
| SMS OTP | Discouraged |

## Org follow-up

GitHub Organization setting **Require two-factor authentication for everyone in the organization** should be enabled for Phenotype when org policy allows. Until then, this document is the project-level soft contract.

## Verification (manual)

Maintainers: GitHub → Settings → Password and authentication → confirm 2FA is enabled.
