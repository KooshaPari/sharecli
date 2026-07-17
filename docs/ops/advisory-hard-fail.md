# Advisory hard-fail promotion path (soft)

Audit-v38 **C01 L19** — supply-chain CVE posture via `cargo-audit` (RustSec) and
`cargo-deny` (licenses, bans, advisories, sources). Documents the path from
**soft detection (today)** to a **hard merge gate** without flipping CI yet.

Related: [`gitleaks.md`](gitleaks.md) (secret leakage into lockfiles) ·
[`slsa-l3-plan.md`](slsa-l3-plan.md) (L53–L56 provenance) · C04 `osv.yml` (OSV/GHSA
feed, still soft).

## Current stance (soft)

| Control | Workflow / config | Gate strength | Notes |
|---------|-------------------|---------------|-------|
| `cargo audit` (RustSec) | `audit.yml` | **Partial hard** | Fails on unignored vulns, but triggers only on `Cargo.toml` / `Cargo.lock` path filters + weekly cron |
| `cargo audit` (duplicate) | `security.yml` (`sast`, `dependencies`) | Soft overlap | Runs on every push/PR; redundant with `audit.yml` — consolidate in hard phase |
| `cargo deny check` | `deny.yml` | **Partial hard** | Fails on policy breach; same path filters as `audit.yml` |
| Yanked crates | `audit.toml` (`yanked = "warn"`) | Soft | Transitive yanked (`spin` via `pprof`) warn-only; no `--deny warnings` |
| Ignored advisories | `deny.toml` `[advisories].ignore` | Documented exceptions | Three `RUSTSEC-*` entries with `reason` (no safe upgrade yet) |
| OSV / GHSA | `osv.yml` | Soft | `continue-on-error: true`; SARIF upload only |
| Local DX | `mise.toml` `audit` task | Advisory | `cargo audit` for maintainer dry-run |

**Net:** scanners exist and fail on **new** unignored advisories when lockfile
workflows fire, but they are not yet a **required check on every PR** and the
RustSec backlog + OSV lane remain soft.

## Target stance (hard)

| Control | Target |
|---------|--------|
| `cargo audit` | Required check on **all** PRs to `main`; zero unignored `RUSTSEC-*` / GHSA in `Cargo.lock` |
| `cargo deny check` | Required check on **all** PRs; `cargo deny --locked check advisories bans licenses sources` |
| Ignore policy | Only `deny.toml` / `audit.toml` entries with dated `reason`; no silent `|| true` |
| Yanked crates | Either upgrade off yanked versions or explicit `deny.toml` / `audit.toml` exception |
| OSV | Remove soft shim in `osv.yml`; fail on HIGH/CRITICAL (C04 L38 couples here) |
| Branch protection | `audit` + `deny` (or unified `supply-chain.yml`) in required checks alongside `ci-success` |

L19 score stays **3** until hard gate is live; this doc closes the **plan** gap in
the C01 scorecard.

## Scanner roles (do not duplicate blindly)

| Tool | Feed / scope | Source of truth |
|------|--------------|-----------------|
| **cargo-audit** | RustSec advisory DB | `audit.toml`, `Cargo.lock` |
| **cargo-deny** `[advisories]` | RustSec (shared DB) + ignore list | `deny.toml` |
| **cargo-deny** `[licenses]` `[bans]` `[sources]` | License, duplicate crate, registry/git policy | `deny.toml` |
| **osv-scanner** | OSV.dev / GHSA cross-ecosystem | `osv.yml` (soft today) |

Promotion rule: **one advisory ID → one ignore location** (`deny.toml` preferred;
`audit.toml` only for cargo-audit-specific knobs like `yanked`).

## Soft phases (no hard gate yet)

| Phase | Deliverable | Hard gate? |
|-------|-------------|------------|
| **0 — today** | `audit.yml` + `deny.yml` + `security.yml` + `deny.toml` ignores | No (path-filtered + OSV soft) |
| **1 — backlog burn** | Clear `RUSTSEC-2026-0194/0195` (`quick-xml` ≥ 0.41) and trim `deny.toml` ignores as upgrades land | No |
| **2 — unify workflows** | Single `supply-chain.yml`: audit + deny + SARIF; drop duplicate `security.yml` audit steps | No — still report-only on OSV |
| **3 — widen triggers** | Run on every PR/push (remove `paths:` filter) + add to `ci-success` needs | No — `continue-on-error` on OSV only |
| **4 — hard gate** | Branch protection required checks; remove OSV soft shim; `yanked = "deny"` or resolved deps | **Yes — deferred** |

Phase 0–3 are **documentation + soft CI** only. Phase 4 needs maintainer sign-off
after `main` stays green for one week with widened triggers.

## Backlog checklist (phase 1)

Current `deny.toml` ignores (must shrink before hard gate):

| ID | Crate / area | Blocker | Action |
|----|--------------|---------|--------|
| `RUSTSEC-2025-0134` | `rustls-pemfile` via `async-nats` | Major `async-nats` / TLS migration | Track dep upgrade PR |
| `RUSTSEC-2025-0140` | `gix` 0.71 pin | Breaking API in `Cargo.toml` | Bump `gix` + fix callers |
| `RUSTSEC-2026-0049` | `rustls-webpki` via `async-nats` | Transitive rustls stack | Same train as 0134 |

Historical wave-0 failures (`audit/WAVE0_FAILURE_MATRIX.md`):

| ID | Crate | Fix |
|----|-------|-----|
| `RUSTSEC-2026-0194` | `quick-xml` | `>= 0.41.0` |
| `RUSTSEC-2026-0195` | `quick-xml` | `>= 0.41.0` |

## Hard-gate wiring (phase 4 — planned)

1. **Workflow** — merge `audit.yml` + `deny.yml` into `.github/workflows/supply-chain.yml`:
   ```yaml
   - run: cargo audit --locked
   - run: cargo deny --locked check advisories bans licenses sources
   ```
2. **`ci-success`** — add `supply-chain` to `needs:` so aggregator fails truthfully.
3. **Branch protection** — require `cargo audit (RustSec)` + `cargo deny` (or unified job name).
4. **`audit.toml`** — set `yanked = "deny"` once `spin`/`pprof` chain is upgraded.
5. **`osv.yml`** — remove `continue-on-error` and soft pass step; align severity with C04 L38.
6. **VEX / ignore hygiene** — any remaining ignore requires `reason` + issue link in `deny.toml`.

## Commands (local dry-run)

```bash
# RustSec only (matches audit.yml)
cargo audit

# Full deny policy (matches deny.yml)
cargo deny check

# Locked CI parity
cargo audit --locked
cargo deny --locked check advisories bans licenses sources

# mise shortcut
mise run audit
```

Expect failure until phase-1 backlog items are resolved; do not add `|| true` in CI.

## Audit evidence (C01 L19)

| Line | Evidence | Score |
|------|----------|-------|
| **L19** Supply-chain security | `Cargo.lock` + `--locked` CI, `deny.toml`, `audit.yml`, `deny.yml`, CycloneDX SBOM, this promotion plan | **3** — unchanged; hard gate deferred |

**Soft follow-up**

| Item | Status |
|------|--------|
| Advisory hard-fail promotion plan | Done (this file) |
| Backlog burn (`quick-xml`, `async-nats`/`gix` ignores) | Open |
| Unified `supply-chain.yml` + required checks | Deferred |
| OSV hard-fail (C04 L38) | Deferred |
