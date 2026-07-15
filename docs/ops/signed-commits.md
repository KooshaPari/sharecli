# Signed commits (sharecli)

Soft policy for audit-v38 **C04 L34**.

## Current stance

| Control | Status |
|---------|--------|
| DCO `Signed-off-by` documented in `CONTRIBUTING.md` | Soft required (contributor guide) |
| Soft CI warn on missing DCO (`dco-soft.yml`) | Enabled (`continue-on-error`) |
| Soft CI warn on unverified GPG/SSH (`gpg-soft.yml`) | Enabled (`continue-on-error`) |
| Branch protection: required signed commits | Not enabled (org / ruleset follow-up) |
| GPG/SSH signing for maintainers | Encouraged via checklist below |

## Contributor quick path (DCO)

```bash
git commit -s -m "feat(scope): message"
```

## Maintainer quick path (GPG or SSH signing)

```bash
# SSH signing (GitHub-recommended)
git config --global gpg.format ssh
git config --global user.signingkey ~/.ssh/id_ed25519.pub
git config --global commit.gpgsign true
# upload the same public key under GitHub → Settings → SSH and GPG keys → Signing keys

# or classic GPG
git config --global user.signingkey <KEYID>
git config --global commit.gpgsign true
```

## Branch protection / ruleset soft checklist (org admin)

Do **not** flip these on until bots and release automation sign commits.

1. Ruleset or classic branch protection on `main`: **Require signed commits**.
2. Ensure `github-actions[bot]` / dependabot path is excluded or uses a signing identity.
3. Maintainers publish SSH signing keys (or GPG) and verify a green “Verified” badge on a test PR.
4. After one week soft-green on `gpg-soft.yml`, promote to required.

## Soft CI

| Workflow | Checks |
|----------|--------|
| `dco-soft.yml` | Each PR commit has `Signed-off-by:` |
| `gpg-soft.yml` | Each PR commit reports GitHub `verified` (GPG/SSH) via API |

Both stay `continue-on-error` until branch protection is enabled.

## Follow-up

Enable GitHub “Require signed commits” once maintainers and bots are accounted for.
