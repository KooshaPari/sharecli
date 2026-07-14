# Signed commits (sharecli)

Soft policy for audit-v38 **C04 L34**.

## Current stance

| Control | Status |
|---------|--------|
| DCO `Signed-off-by` documented in `CONTRIBUTING.md` | Soft required (contributor guide) |
| Soft CI warn on missing DCO (`dco-soft.yml`) | Enabled (`continue-on-error`) |
| Branch protection: required signed commits | Not enabled (org setting follow-up) |
| GPG/SSH signing for maintainers | Encouraged, not enforced |

## Contributor quick path

```bash
git commit -s -m "feat(scope): message"
```

## Follow-up

Enable GitHub branch protection “Require signed commits” once maintainers
publish signing keys and bot commits are accounted for.
