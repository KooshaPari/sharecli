# Gitleaks secret scanning (soft)

Audit-v38 **C01 L18** (secret management) and **L19** cross-cut (supply-chain hygiene — no secrets in lockfiles, fixtures, or release artefacts).

## Current stance

| Control | Status |
|---------|--------|
| `gitleaks.toml` at repo root | Configured (allowlists + custom rules) |
| CI gate via `security.yml` | Enabled on push/PR to `main`, daily cron, `workflow_dispatch` |
| Optional local pre-commit hook | Documented below; not required for CI green |
| Secret rotation runbook | Checklist below; runtime store still env-only |

See also: [`secrets.md`](secrets.md) (L18 env discipline), [`crypto-keys.md`](crypto-keys.md) (serve token policy).

## `gitleaks.toml` rules

Config lives at the repo root. Gitleaks reads it automatically when invoked locally or via `gitleaks-action`.

### Allowlist

| Kind | Purpose |
|------|---------|
| **Files** | Docs, GitHub workflows, markdown, tests, and `.env.example` / template files — placeholders are not production secrets |
| **Regexes** | Known dev literals (`test-secret`, `CHANGE_ME_IN_PROD`, `your-*-here`, etc.) |

Add a new file glob or regex when a fixture legitimately matches a rule (prefer tightening the fixture over disabling rules).

### Custom rules (high-signal)

| Rule ID | Detects |
|---------|---------|
| `openrouter-api-key` | `sk-or-v1-…` |
| `openai-api-key` | `sk-…` (48-char body) |
| `anthropic-api-key` | `sk-ant-…` |
| `github-oauth-token` | `ghp_…` |
| `github-fine-grained-pat` | `github_pat_…` |
| `github-app-token` | `ghs_…` |
| `generic-api-key` | `api_key` / `apikey` assignment patterns |
| `aws-access-key` | `AKIA…` |
| `aws-secret-key` | 40-char base64-like secret material |
| `slack-token` | `xoxb-` / `xoxp-` / etc. |
| `slack-webhook` | `hooks.slack.com/services/…` |
| `private-key` | PEM `BEGIN … PRIVATE KEY` blocks |
| `jwt-token` | `eyJ…` three-part JWT shape |
| `database-url` | `postgres://user:pass@` (and mysql/mongodb) |

Entropy thresholds on API-key rules reduce noise on random-looking literals in tests.

### Local verify

```bash
# Install once (pick one)
brew install gitleaks          # macOS
# or: https://github.com/gitleaks/gitleaks/releases

gitleaks detect --source . --verbose --redact --config gitleaks.toml
```

Match CI flags: `--verbose --redact` (same as `security.yml`).

## `security.yml` wiring

Workflow: [`.github/workflows/security.yml`](../../.github/workflows/security.yml)

| Trigger | When |
|---------|------|
| `push` / `pull_request` | `main`, `master` |
| `schedule` | Daily `0 2 * * *` UTC |
| `workflow_dispatch` | Manual re-scan |

**Job `secrets` (Secret Detection)**

1. `actions/checkout` with `fetch-depth: 0` (full history — catches secrets in older commits on PRs).
2. `gitleaks/gitleaks-action@ff98106e4c7b2bc287b24eaf42907196329070c7` with `args: --verbose --redact`.
3. `GITHUB_TOKEN` supplied for PR annotation (read-only `contents` / `actions` permissions).

The same workflow also runs SAST (clippy security lints, `cargo audit`), dependency audit, and optional Trivy container scan — gitleaks is the **secret** lane only.

## Optional pre-commit hook

CI is the enforcement gate. A local hook catches accidents before push.

**Prerequisites:** [pre-commit](https://pre-commit.com/) installed (`pip install pre-commit` or your OS package manager).

Add to a **local** `.pre-commit-config.yaml` fragment (or merge into your developer overlay):

```yaml
repos:
  - repo: https://github.com/gitleaks/gitleaks
    rev: v8.21.2
    hooks:
      - id: gitleaks
        args: [--verbose, --redact, --config, gitleaks.toml]
```

Enable:

```bash
pre-commit install
pre-commit run gitleaks --all-files   # one-shot sanity check
```

The committed `.pre-commit-config.yaml` may point at shared infra templates; treat gitleaks as **opt-in** until a federated hook lands repo-wide.

## Secret rotation checklist

Use when a real secret may have touched git, CI logs, or a shared host.

| Step | Action |
|------|--------|
| 1. **Contain** | Revoke the exposed credential at the provider (GitHub, OpenAI, AWS, Slack, etc.). |
| 2. **Rotate** | Issue a new secret; update CI/org secrets and runtime env (`SHARECLI_SERVE_TOKEN`, API keys). |
| 3. **Redeploy** | Restart `sharecli serve` (or dependent services) so the new env value is live. |
| 4. **Scan** | Run `gitleaks detect --source . --verbose` locally; re-run **Security Scan** workflow on `main`. |
| 5. **History** | If the secret was **committed**, use `git filter-repo` or BFG per org policy; force-push only with maintainer approval. |
| 6. **Disclose** | File an internal incident note; open a GHSA/private advisory if the secret grants org-wide access. |
| 7. **Prevent** | Add allowlist entry only for intentional fixtures; never allowlist production-shaped literals. |

**SLO (soft):** human/API keys ≤ 90 days; serve tokens on compromise or quarterly for shared hosts.

## Audit evidence (C01)

| Line | Evidence in this repo | Score (soft) |
|------|------------------------|--------------|
| **L18** Secret management | `gitleaks.toml`, `security.yml`, `.env.example`, this runbook, [`secrets.md`](secrets.md) | **2** — scanning + docs; OS keyring deferred |
| **L19** Supply-chain security | `Cargo.lock` + `--locked` CI, `deny.toml`, CycloneDX SBOM (`sbom.yml` / release); gitleaks prevents secret leakage into artefacts | **3** — unchanged; gitleaks polish is L18 documentation |

**Soft follow-up**

| Item | Status |
|------|--------|
| Gitleaks runbook + rotation checklist | Done (this file) |
| Federated pre-commit gitleaks in-repo | Deferred |
| Second scanner (e.g. trufflehog) | Deferred |
| Runtime OS keyring for serve token | Deferred |
