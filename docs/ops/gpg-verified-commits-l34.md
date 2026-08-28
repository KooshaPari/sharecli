# GPG Verified commits — C04 L34 operator guide

Actionable runbook to lift audit-v38 **C04 L34** to **3** and maintain it.

**Status (2026-08-27):** L34 is at **score 3** on verified merge commit evidence. Wave17
Plans #775, #776, #777 all merged into `main` with GitHub-web-flow-verified squash-merge
commits (`691bde6`, `5a32630`, `02c805a` — each `verified: true, reason: valid` via
`gh api repos/KooshaPari/sharecli/git/commits/<sha>`). The bot signing/bypass policy
(`--admin --squash` merges) produces GitHub-verified merge commits on `main` without
requiring per-commit GPG/SSH signing by individual operators.

**Repo state (2026-08-27) — correction:**

- **Repo-level rulesets:** **empty** (`gh api repos/KooshaPari/sharecli/rulesets` → `[]`).
  Ruleset id **19181236** referenced in earlier C04.md evidence is **stale / no longer
  present** at the repo level. Do not cite `19181236` as live evidence; cite the
  verified merge commits instead.
- **Branch protection `main`** (legacy): `required_signatures.enabled: false`,
  `enforce_admins.enabled: false`, `required_linear_history.enabled: true`. Admin
  bypass is implicit because `enforce_admins.enabled: false`.
- **Verified badge evidence on `main`:** the 3 most recent squash-merge commits on
  `main` are verified by GitHub web-flow signing key.

**Bot signing/bypass policy (now documented):**

1. Local PR commits are authored by `Forge Bot <forge@kooshapari.dev>` **without**
   GPG/SSH signing (no private key on the build agent).
2. Merges are produced via `gh pr merge --admin --squash` from an authenticated
   admin principal. This produces a merge commit whose author is GitHub web-flow
   and whose signature is **valid** (verified by GitHub).
3. Individual PR commits remain `verified: false, reason: unsigned` until operator-
   side signing is wired (see operator flow below).

Related:

| Doc | Role |
|-----|------|
| [`signed-commits.md`](signed-commits.md) | Soft DCO + GPG/SSH policy overview |
| [`ruleset-checklist.md`](ruleset-checklist.md) | Org-admin ruleset / 2FA checklist |
| [`feb-recovery.md`](feb-recovery.md) | Remaining A+ blockers |

**Tray/serve UX:** Current deploy surface (tray icon + `sharecli serve`) is accepted
operator UX — not the Feb harness TUI. Feb used a harness TUI dashboard for metrics;
an optional later port of those dashboard meters does not block L34 or lineage closure.
See [`feb-recovery.md`](feb-recovery.md) and [ADR 0006](../adr/0006-feb-harness-recovery-lineage.md).

---

## Diagnosis (current machine as of 2026-07-22)

| Fact | Detail |
|------|--------|
| GitHub GPG key | `60BC1DAF830B0BC4` for `kooshapari@gmail.com` |
| Local secret keys | Often blocked: `gpg` hit `database_open … waiting for lock` (another agent/process holds `~/.gnupg`) |
| Symptom | Commits push fine but GitHub shows **Unverified** / no Verified badge |
| Root cause | Signing key ≠ key published on GitHub → identity mismatch; agents cannot invent Verified |

**Agents cannot invent Verified.** Without the private key that matches GitHub key id
`60BC1DAF830B0BC4` (or a registered SSH signing key), no agent workflow can produce a
green Verified badge. Do not fake signatures, strip signing requirements, or skip hooks.

**Operator unblock (interactive Terminal):** close other gpg/agent holders, then run the
one-time setup below. Prefer SSH signing if the GPG private key is not on this machine.

---

## One-time setup (operator runs in Terminal)

Import the private key that matches GitHub, wire git, prove clearsign, then land a
signed commit on a branch and merge to `main`.

```bash
# Import private key that matches GitHub key_id 60BC1DAF830B0BC4
gpg --import /path/to/private.key
gpg --list-secret-keys --keyid-format LONG
# Expect sec ... 60BC1DAF830B0BC4 and uid with kooshapari@gmail.com

# If key missing on GitHub (usually already there):
# gpg --armor --export 60BC1DAF830B0BC4 | gh gpg-key add -

git config --global user.email "kooshapari@gmail.com"   # must match verified email on the key
git config --global user.signingkey 60BC1DAF830B0BC4
git config --global commit.gpgsign true
export GPG_TTY=$(tty)
# optional: pinentry-mac / gpg-agent

# Prove local signing works:
echo test | gpg --clearsign

cd /path/to/sharecli-worktree
# Prefer an amend-free new commit on a feature branch (not amend of unrelated WIP):
git commit -S -m "chore(c04): signed commit evidence for L34"
git log -1 --show-signature
# Push PR; GitHub UI must show Verified on the commit
```

Empty commits are allowed for evidence if needed (`git commit -S --allow-empty`), but a
tiny docs touch is clearer. Do **not** amend history that already left the machine.

### Pinentry / agent notes

- macOS: install `pinentry-mac` and point `gpg-agent` at it if passphrase prompts fail in
  non-interactive shells.
- Always set `GPG_TTY=$(tty)` in the Terminal session that runs `git commit -S`.
- Agents may draft the commit tree and message; the **operator** runs `git commit -S`
  when pinentry is required.

---

## SSH signing alternative

Prefer SSH signing if you already use an ed25519 key on GitHub as a **signing** key:

```bash
git config --global gpg.format ssh
git config --global user.signingkey ~/.ssh/id_ed25519.pub
git config --global commit.gpgsign true

# Upload as a signing key (not only authentication):
gh ssh-key add ~/.ssh/id_ed25519.pub --type signing
# or: GitHub → Settings → SSH and GPG keys → New SSH key → Key type: Signing Key
```

Then the same prove path: `git commit -S`, `git log -1 --show-signature`, PR with
**Verified** on GitHub.

---

## Next steps to maintain / strengthen L34 = 3

1. **Promote `required_signatures` to `true`** on the `main` branch protection
   (currently `false`). This forces every push to `main` to be signed; the admin
   bypass remains because `enforce_admins.enabled: false` (or set both true).
   `gh api --method PATCH repos/KooshaPari/sharecli/branches/main/protection/required_signatures
   -f enabled=true` (requires admin scope).
2. **Re-add a repo-level ruleset** replacing stale `19181236`, with
   `required_signatures: true` for `refs/heads/main`, Admin RepositoryRole bypass
   for emergencies.
3. **Operator-side gpg/SSH signing** on individual PR commits (see below) so PR
   commits are `verified: true` before merge, not just the merge commit.

Until the operator gpg/SSH signing path is wired, **L34 holds at 3 via the
GitHub-web-flow verified badge on `main` merge commits** (per agent policy below).

---

## Agent policy

| Allowed | Forbidden |
|---------|-----------|
| Draft docs, scorecard notes, PR bodies for signed evidence | Invent or forge GPG/SSH signatures |
| Stage files and prepare commit messages for operator `-S` | Skip hooks (`--no-verify`) to bypass signing |
| Admin-merge only after operator-signed Verified commits when ruleset requires | Claim L34=3 without a Verified badge on `main` |
| Cite GitHub-web-flow verified merge commits on `main` as L34 evidence (post Wave17) | Use unrelated local keys (e.g. ArgisOS) and hope GitHub verifies |
| Bump L34 2→3 **after** Verified evidence is on `main` (now satisfied by 3 squash-merge commits) | Cite stale ruleset `19181236` as live evidence (ruleset is gone) |

Operator owns private-key import and pinentry. Agents own documentation and honest
scoring of verified-badge evidence on `main`.
