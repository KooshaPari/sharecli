# OSV / GHSA hard-fail promotion path (soft)

Audit-v38 **C04 L38** — cross-ecosystem CVE feed via OSV-Scanner on `Cargo.lock`
(OSV.dev + GitHub Security Advisories). Documents the path from **soft detection
(today)** to a **hard merge gate** without flipping CI yet.

Related: [`advisory-hard-fail.md`](advisory-hard-fail.md) (C01 L19 RustSec /
`cargo-deny` lane — shared backlog and phase-4 coupling) · [`container-hardening.md`](container-hardening.md)
(L40 runtime boundary) · `SECURITY.md` (dependency scanning overview).

## Current stance (soft)

| Control | Workflow / config | Gate strength | Notes |
|---------|-------------------|---------------|-------|
| OSV-Scanner | `osv.yml` | **Soft** | `continue-on-error: true` on scan step |
| SARIF upload | `osv.yml` → Code Scanning | Advisory | `continue-on-error: true`; findings visible in Security tab |
| Soft pass shim | `osv.yml` `Soft gate (always pass)` | Soft | `exit 0` regardless of scan outcome |
| Triggers | PR + push `main` + weekly cron | Broad | Runs on every PR (no path filter) |
| RustSec overlap | `audit.yml` / `deny.yml` | Partial hard | Path-filtered; see advisory-hard-fail doc |
| Local DX | — | Missing | No `mise` / `just` OSV task yet |

**Net:** OSV/GHSA findings are **subscribed and surfaced** (score **2**), but the
workflow never fails CI. Branch protection does not require `OSV / GHSA lockfile scan`.

## Target stance (hard)

| Control | Target |
|---------|--------|
| OSV-Scanner | Fail job on **HIGH** / **CRITICAL** (or any unignored vuln once baseline zero) |
| Soft shim | Remove `continue-on-error` + `Soft gate (always pass)` step |
| SARIF | Keep upload on failure (`if: always()`) for Security tab triage |
| Severity policy | Align with `deny.toml` / RustSec ignores — one advisory ID → one ignore location |
| `ci-success` | Add `osv` to aggregator `needs:` (or unified `supply-chain.yml`) |
| Branch protection | `OSV / GHSA lockfile scan` in required checks alongside `ci-success` |
| Score | L38 **2 → 3** when hard gate is live and green for one week |

L38 stays **2** until hard gate is live; this doc closes the **plan** gap in the
C04 scorecard.

## Scanner roles (RustSec vs OSV)

| Tool | Feed / scope | Source of truth | Gate today |
|------|--------------|-----------------|------------|
| **cargo-audit** | RustSec advisory DB | `audit.toml`, `Cargo.lock` | Partial hard (path-filtered) |
| **cargo-deny** `[advisories]` | RustSec (shared DB) | `deny.toml` | Partial hard (path-filtered) |
| **osv-scanner** | OSV.dev + GHSA cross-ecosystem | `osv.yml`, `Cargo.lock` | **Soft** |

OSV catches GHSA IDs and non-RustSec ecosystem advisories that RustSec-only scans
may miss. Promotion rule: **do not duplicate ignores** — resolve or document in
`deny.toml` first; OSV hard-fail inherits the same zero-unignored baseline as
[`advisory-hard-fail.md`](advisory-hard-fail.md) phase 1.

## Soft phases (no hard gate yet)

| Phase | Deliverable | Hard gate? |
|-------|-------------|------------|
| **0 — today** | `osv.yml` PR + weekly + SARIF; soft pass shim | No |
| **1 — backlog burn** | Clear RustSec ignores per advisory-hard-fail checklist (`quick-xml`, `async-nats`/`gix`) | No |
| **2 — local DX** | `mise run osv` or `just osv-scan` mirroring CI args | No |
| **3 — dry-run hard** | Remove soft shim on a feature branch; fix findings; revert to soft on `main` | No — experiment only |
| **4 — hard gate** | Remove soft shim on `main`; add to `ci-success` + branch protection | **Yes — deferred** |

Phase 0–3 are **documentation + soft CI** only. Phase 4 needs maintainer sign-off
after [`advisory-hard-fail.md`](advisory-hard-fail.md) phase 1 backlog is cleared
and `main` stays green for one week with OSV failing truthfully on a dry-run branch.

## Coupling with C01 L19 (advisory hard-fail)

| Milestone | C01 L19 | C04 L38 |
|-----------|---------|---------|
| RustSec backlog cleared | `cargo audit --locked` clean | Prerequisite |
| Unified supply-chain workflow | `supply-chain.yml` (audit + deny + OSV) | OSV job merged in |
| Hard gate flip | Required `cargo deny` + `cargo audit` | Required `osv` job |
| Ignore hygiene | `deny.toml` dated `reason` only | OSV inherits same baseline |

Do **not** hard-fail OSV independently while RustSec backlog and `deny.toml` ignores
remain — duplicate noise and conflicting triage. Follow the joint phase-4 sequence in
[`advisory-hard-fail.md`](advisory-hard-fail.md#hard-gate-wiring-phase-4--planned).

## Hard-gate wiring (phase 4 — planned)

1. **`osv.yml`** — remove soft posture:
   ```yaml
   # DELETE: continue-on-error: true on scan step
   # DELETE: "Soft gate (always pass)" step
   - name: Scan Cargo.lock with OSV-Scanner
     uses: google/osv-scanner-action/osv-scanner-action@...
     with:
       scan-args: |-
         --lockfile=Cargo.lock
         --format=sarif
         --output=osv-results.sarif
         --severity=HIGH,CRITICAL
   ```
2. **`ci-success`** (or unified `supply-chain.yml`) — add `osv` to `needs:`.
3. **Branch protection** — require `OSV / GHSA lockfile scan`.
4. **SARIF** — keep upload step with `if: always()` for Security tab history.
5. **Scorecard** — re-score L38 to **3** after one green week on `main`.

Optional follow-up: NVD webhook / GitHub Advisory watch for out-of-band notification
(effort M; not required for score 3).

## Commands (local dry-run)

```bash
# Install (maintainer one-time)
go install github.com/google/osv-scanner/cmd/osv-scanner@latest

# CI parity (soft today — fails locally, does not block merge)
osv-scanner --lockfile=Cargo.lock

# SARIF output (matches osv.yml)
osv-scanner --lockfile=Cargo.lock --format=sarif --output=osv-results.sarif

# Severity-filtered (phase-4 target)
osv-scanner --lockfile=Cargo.lock --severity=HIGH,CRITICAL
```

Expect failure until phase-1 backlog items in [`advisory-hard-fail.md`](advisory-hard-fail.md)
are resolved; do not add `|| true` or `exit 0` shims in CI.

## Audit evidence (C04 L38)

| Line | Evidence | Score |
|------|----------|-------|
| **L38** CVE feed subscribed | `osv.yml`, SARIF upload, `SECURITY.md`, this promotion plan | **2** — unchanged; hard gate deferred |

**Soft follow-up**

| Item | Status |
|------|--------|
| OSV hard-fail promotion plan | Done (this file) |
| RustSec backlog burn (shared with C01 L19) | Open — see advisory-hard-fail.md |
| Local `mise` / `just` OSV task | Open |
| Remove soft shim + required check | Deferred |
| L38 score lift (2 → 3) | Deferred |
