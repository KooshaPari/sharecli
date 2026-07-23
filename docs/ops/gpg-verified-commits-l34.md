# GPG Verified commits — C04 L34 operator guide

Actionable runbook to lift audit-v38 **C04 L34** from **2 → 3**.

**Ruleset already active:** `main-signed-commits` id **19181236**
([html](https://github.com/KooshaPari/sharecli/rules/19181236)) — `required_signatures`
on `refs/heads/main`. Ruleset alone does **not** lift L34; GitHub must show a green
**Verified** badge on a commit that lands on `main`.

Related:

| Doc | Role |
|-----|------|
| [`signed-commits.md`](signed-commits.md) | Soft DCO + GPG/SSH policy overview |
| [`ruleset-checklist.md`](ruleset-checklist.md) | Org-admin ruleset / 2FA checklist |
| [`feb-recovery.md`](feb-recovery.md) | Remaining A+ blockers (includes L34) |

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

## After first Verified commit on `main`

1. Confirm the merge commit (or the signed tip) shows **Verified** in the GitHub UI.
2. Update [`audit/SCORECARD-v38.md`](../../audit/SCORECARD-v38.md): L34 **2 → 3** with
   evidence (commit SHA + Verified badge). Agents may do this **after** evidence exists —
   not before.
3. Tick the remaining maintainer steps in [`ruleset-checklist.md`](ruleset-checklist.md)
   (bot signing / Admin bypass review; consider promoting `gpg-soft.yml` off
   `continue-on-error`).
4. Refresh `audit/.lane-c04/C04.md` L34 gaps once the badge is on `main`.

Until that Verified evidence exists, **L34 stays 2**.

---

## Agent policy

| Allowed | Forbidden |
|---------|-----------|
| Draft docs, scorecard notes, PR bodies for signed evidence | Invent or forge GPG/SSH signatures |
| Stage files and prepare commit messages for operator `-S` | Skip hooks (`--no-verify`) to bypass signing |
| Admin-merge only after operator-signed Verified commits when ruleset requires | Claim L34=3 without a Verified badge on `main` |
| Bump L34 2→3 **after** Verified evidence is on `main` | Use unrelated local keys (e.g. ArgisOS) and hope GitHub verifies |

Operator owns private-key import and pinentry. Agents own documentation and honest
scoring until Verified lands.
