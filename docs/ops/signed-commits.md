# Signed commits (sharecli)

Soft policy for audit-v38 **C04 L34**.

## Current stance

| Control | Status |
|---------|--------|
| DCO `Signed-off-by` documented in `CONTRIBUTING.md` | Soft required (contributor guide) |
| Soft CI warn on missing DCO (`dco-soft.yml`) | Enabled (`continue-on-error`) |
| Soft CI warn on unverified GPG/SSH (`gpg-soft.yml`) | Enabled (`continue-on-error`) |
| Branch ruleset: required signed commits | **Active** — ruleset `main-signed-commits` id **19181236** ([html](https://github.com/KooshaPari/sharecli/rules/19181236)) |
| GPG/SSH signing for maintainers | Encouraged via checklist below |

## Ruleset evidence (2026-07-19)

```text
POST /repos/KooshaPari/sharecli/rulesets → 201
id: 19181236
name: main-signed-commits
target: branch
enforcement: active
conditions.ref_name.include: [refs/heads/main]
rules: [{ type: required_signatures }]
bypass_actors: [{ actor_id: 5, actor_type: RepositoryRole, bypass_mode: always }]  # Admin
```

L34 score stays **2** until a maintainer-signed commit shows a green **Verified** badge on `main` (ruleset alone does not prove signing identity coverage for bots/Dependabot).

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

## Branch protection / ruleset checklist

1. ~~Ruleset or classic branch protection on `main`: **Require signed commits**.~~ **DONE** (ruleset 19181236).
2. Ensure `github-actions[bot]` / dependabot path is excluded or uses a signing identity (Admin bypass covers RepositoryRole id 5 for now).
3. Maintainers publish SSH signing keys (or GPG) and verify a green “Verified” badge on a test PR.
4. After verified commits land on `main`, promote soft CI and consider L34 2→3.

## Soft CI

| Workflow | Checks |
|----------|--------|
| `dco-soft.yml` | Each PR commit has `Signed-off-by:` |
| `gpg-soft.yml` | Each PR commit reports GitHub `verified` (GPG/SSH) via API |

Both stay `continue-on-error` until verified-commit evidence is routine.

## Follow-up

- Confirm Dependabot / Actions commits either sign or use Admin bypass intentionally.
- Land one Verified commit on `main`, then lift L34 to 3.
