# Secret scanning (gitleaks + trufflehog)

Audit-v38 **C04 L31** (secrets in repo), **C01 L18** (secret management), and **L19** cross-cut (supply-chain hygiene — no secrets in lockfiles, fixtures, or release artefacts).

## Current stance

| Control | Status |
|---------|--------|
| `gitleaks.toml` at repo root | Configured (allowlists + custom rules) |
| `.trufflehog.yml` at repo root | Configured (build/lockfile exclusions) |
| CI gate via `security.yml` | **gitleaks** + **trufflehog** on push/PR, daily cron, `workflow_dispatch` |
| Pre-commit hooks (`.pre-commit-config.yaml`) | **gitleaks** on commit; **trufflehog** on pre-push (`--only-verified`) |
| Local parity | `just secret-scan` → `scripts/ci/secret_scan.sh` |
| Secret rotation runbook | Checklist below; runtime store still env-only |

See also: [`secrets.md`](secrets.md) (L18 env discipline), [`crypto-keys.md`](crypto-keys.md) (serve token policy).

## `gitleaks.toml` rules

Config lives at the repo root. CI invokes it explicitly with `--config gitleaks.toml` (the pinned binary step in `security.yml`); locally, `gitleaks detect` also accepts `--config gitleaks.toml`.

### Allowlist

| Kind | Purpose |
|------|---------|
| **Paths** | Docs, markdown, tests, and `.env.example` / template files — placeholders are not production secrets |
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

gitleaks detect --source . --redact --verbose --config gitleaks.toml
```

Match CI flags: `--redact --verbose` (same as `security.yml`).

## `security.yml` wiring

Workflow: [`.github/workflows/security.yml`](../../.github/workflows/security.yml)

| Trigger | When |
|---------|------|
| `push` / `pull_request` | `main`, `master` |
| `schedule` | Daily `0 2 * * *` UTC |
| `workflow_dispatch` | Manual re-scan |

**Job `secrets` (Secret Detection — gitleaks)**

1. `actions/checkout` with `fetch-depth: 0` (full history — catches secrets in older commits on PRs).
2. Download the pinned gitleaks binary (version + SHA256-verified) and run `gitleaks detect --redact --verbose --exit-code=2 --config gitleaks.toml` over the full history.

> Why not `gitleaks/gitleaks-action`? The action ignores its `args` input
> (verified against the pinned action source), so `--config gitleaks.toml`
> never applied and the lane scanned with the built-in rule set. The direct
> invocation also pins the gitleaks version (the action floats unless
> `GITLEAKS_VERSION` is set) and verifies the download checksum.

**Job `trufflehog` (TruffleHog Scan)**

1. `actions/checkout` with `fetch-depth: 0`.
2. `trufflesecurity/trufflehog@9b6b5326bfe25dbd856eccc8a8275eb5dea7bd52` with `extra_args: --only-verified`.
3. Respects `.trufflehog.yml` path exclusions (lockfiles, `target/`, golden PNGs).

The same workflow also runs SAST (clippy security lints, `cargo audit`), dependency audit, and optional Trivy container scan — gitleaks + trufflehog are the **secret** lanes.

## Pre-commit hooks

Committed [`.pre-commit-config.yaml`](../../.pre-commit-config.yaml) wires:

| Hook | Stage | Command |
|------|-------|---------|
| gitleaks | pre-commit | `gitleaks detect … --config gitleaks.toml` |
| trufflehog | pre-push | `trufflehog filesystem . --fail --only-verified` |

**Prerequisites:** [pre-commit](https://pre-commit.com/) installed (`pip install pre-commit` or your OS package manager).

```bash
pre-commit install
pre-commit install --hook-type pre-push
pre-commit run gitleaks --all-files
```

CI remains the enforcement gate; hooks catch accidents before push.

## Local verify (dual scanner)

```bash
just secret-scan
# or:
bash scripts/ci/secret_scan.sh
```

Single-scanner:

```bash
gitleaks detect --source . --verbose --redact --config gitleaks.toml
trufflehog filesystem . --fail --only-verified
```

Match CI flags: gitleaks `--redact --verbose --exit-code=2 --config gitleaks.toml`; trufflehog `--only-verified`.

## Secret rotation checklist

Use when a real secret may have touched git, CI logs, or a shared host.

| Step | Action |
|------|--------|
| 1. **Contain** | Revoke the exposed credential at the provider (GitHub, OpenAI, AWS, Slack, etc.). |
| 2. **Rotate** | Issue a new secret; update CI/org secrets and runtime env (`SHARECLI_SERVE_TOKEN`, API keys). |
| 3. **Redeploy** | Restart `sharecli serve` (or dependent services) so the new env value is live. |
| 4. **Scan** | Run `just secret-scan` locally; re-run **Security Scan** workflow on `main`. |
| 5. **History** | If the secret was **committed**, use `git filter-repo` or BFG per org policy; force-push only with maintainer approval. |
| 6. **Disclose** | File an internal incident note; open a GHSA/private advisory if the secret grants org-wide access. |
| 7. **Prevent** | Add allowlist entry only for intentional fixtures; never allowlist production-shaped literals. |

**SLO (soft):** human/API keys ≤ 90 days; serve tokens on compromise or quarterly for shared hosts.

## Audit evidence (C04 / C01)

| Line | Evidence in this repo | Score |
|------|------------------------|-------|
| **L31** Secrets in repo | `gitleaks.toml`, `security.yml` (gitleaks + trufflehog), `.pre-commit-config.yaml`, `.trufflehog.yml`, `scripts/ci/secret_scan.sh`, this runbook | **3** — dual scanner CI + pre-commit + PR bot |
| **L18** Secret management | `gitleaks.toml`, `security.yml`, `.env.example`, [`secrets.md`](secrets.md) | **2** — scanning + docs; OS keyring deferred |
| **L19** Supply-chain security | `Cargo.lock` + `--locked` CI, `deny.toml`, CycloneDX SBOM (`sbom.yml` / release); secret scanners prevent leakage into artefacts | **3** — unchanged |

**Soft follow-up**

| Item | Status |
|------|--------|
| Gitleaks + trufflehog runbook + rotation checklist | Done (this file) |
| Federated pre-commit in-repo | Done (`.pre-commit-config.yaml`) |
| Runtime OS keyring for serve token | Deferred |
