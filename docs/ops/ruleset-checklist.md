# GitHub ruleset + org 2FA checklist (soft)

Audit-v38 **C04 L34** (signed commits) and **C04 L36** (maintainer 2FA) — org-admin runbook for promoting soft in-repo policy to GitHub enforcement.

## Related policy docs

| Pillar | Document | Current score |
|--------|----------|---------------|
| L34 Signed commits | [`signed-commits.md`](signed-commits.md), [`gpg-verified-commits-l34.md`](gpg-verified-commits-l34.md) | 2 (ruleset **19181236** active; Verified badge on `main` still pending) |
| L36 Maintainer 2FA | [`maintainer-2fa.md`](maintainer-2fa.md) | 1 (project policy; org enforce pending) |

## Preconditions (do not skip)

- [ ] `dco-soft.yml` and `gpg-soft.yml` green for one week on recent PRs (soft `continue-on-error`).
- [ ] Every maintainer with merge rights has SSH or GPG signing configured per [`signed-commits.md`](signed-commits.md).
- [ ] Every maintainer confirms personal GitHub 2FA per [`maintainer-2fa.md`](maintainer-2fa.md).
- [ ] Bot paths documented: `github-actions[bot]`, Dependabot, release automation — excluded or signing.

## L34 — Require signed commits (ruleset)

GitHub → **Settings → Rules → Rulesets** (or classic branch protection on `main`).

| Step | Action | Owner |
|------|--------|-------|
| 1 | Create ruleset targeting `main` (`~DEFAULT_BRANCH`) | Org admin |
| 2 | Enable **Require signed commits** | Org admin |
| 3 | Keep existing PR rules (≥1 approval, code owners, required checks) | Org admin |
| 4 | Exclude or sign for `github-actions[bot]` / Dependabot merge commits | Org admin |
| 5 | Maintainer test PR: every commit shows **Verified** badge | Maintainers |
| 6 | After ruleset live: flip `gpg-soft.yml` from `continue-on-error` to required (follow-up PR) | Maintainers |

**Do not enable** until steps 1–5 in Preconditions are checked. Unsigned bot merges will block release and dependency PRs.

## L36 — Org 2FA enforce

GitHub Organization (**Phenotype**) → **Settings → Authentication security**.

| Step | Action | Owner |
|------|--------|-------|
| 1 | Audit collaborators: Settings → People → filter write/admin | Org admin |
| 2 | Notify collaborators without 2FA; deadline before enforce | Org admin |
| 3 | Enable **Require two-factor authentication for everyone in the organization** | Org admin |
| 4 | Prefer hardware security key (WebAuthn) over SMS per [`maintainer-2fa.md`](maintainer-2fa.md) | Maintainers |
| 5 | Re-audit quarterly; revoke merge rights for non-compliant accounts | Org admin |

Until org enforce is on, [`maintainer-2fa.md`](maintainer-2fa.md) remains the project-level soft contract (L36 stays **1**).

## Evidence map (audit)

| Control | In-repo evidence | Enforced on GitHub |
|---------|------------------|-------------------|
| L34 DCO + soft verified CI | `CONTRIBUTING.md`, `dco-soft.yml`, `gpg-soft.yml`, [`signed-commits.md`](signed-commits.md) | Ruleset **19181236** (`main-signed-commits`) — **active** |
| L34 operator Verified lift | [`gpg-verified-commits-l34.md`](gpg-verified-commits-l34.md) | Green **Verified** badge on `main` — pending |
| L34 ruleset runbook | This file | N/A (checklist only) |
| L36 maintainer 2FA policy | [`maintainer-2fa.md`](maintainer-2fa.md), `SECURITY.md` | Org **Require 2FA** — pending |

## Score impact

- **L34** remains **2** until a maintainer commit on `main` shows **Verified** (ruleset already applied; see [`gpg-verified-commits-l34.md`](gpg-verified-commits-l34.md)).
- **L36** rises to **2** when org-wide 2FA enforce is active (or all owners use hardware keys with org policy documented).
