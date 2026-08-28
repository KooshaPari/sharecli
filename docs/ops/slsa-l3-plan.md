# SLSA L3 / network-block roadmap (soft)

Audit-v38 **C06 L53–L54**. Documents the path from SLSA Build **L2 (today)** to **L3**
without enabling a hard CI gate yet.

Related: [hermetic-builds.md](./hermetic-builds.md) (L54 soft offline gate) ·
[`docs/slsa.md`](../slsa.md) (L2 attestation + L56 container cosign).

## Current posture (L2)

| Pillar | Score | Evidence |
|--------|-------|----------|
| L53 Build provenance | 2 | `release-attestation.yml` + `docs/slsa.md` (SLSA Build L2) |
| L54 Hermetic builds | 2 | `hermetic-soft.yml` + `just hermetic` + [hermetic-builds.md](./hermetic-builds.md) |
| L55 Dependency confusion | 3 | `deny.toml` `unknown-registry=deny` + `deny.yml` |
| L56 Container provenance | 3 | `container-cosign.yml` hard GHCR sign+attest+verify (T-660); soft sign-blob retained |

L55/L56 are **prerequisites** for trustworthy L3 provenance: dependency sources are
scoped to crates.io (L55), and container digests can be signed before GHCR publish
defaults (L56).

## Soft phases (no hard gate)

| Phase | Deliverable | Blocks network? | Hard gate? |
|-------|-------------|-----------------|------------|
| **0 — today** | L2 attestation + hermetic-soft poisoned-proxy | Partial (offline phase only) | No (`continue-on-error`) |
| **1 — vendor spike** | `cargo vendor` + documented digest pin in release job | Optional local mirror | No |
| **2 — network-block job** | Dedicated workflow step with egress denied post-`cargo fetch` | Yes (Actions `block` or self-hosted airgap) | No — report-only artifact |
| **3 — SLSA generator L3** | `generator_containerized_slsa3.yml` on release only | Yes (containerized builder) | Soft — landed Wave17 Plan 777 (PR #TBD). Attestation produced, not merge-blocking. |
| **4 — hard gate** | Required check on `main` + release | Yes | **Deferred** |

Phase 0–3 are **documentation + soft CI** only. Phase 4 is explicitly out of scope
for this lane until org signs off on flake budget and vendor policy.

## Network-block design

1. **Fetch boundary** — `cargo fetch --locked` runs once with network (or from warmed
   registry cache). All compile/test steps use `--offline`.
2. **Poisoned-proxy stand-in** — today’s `hermetic-soft.yml` sets
   `HTTP_PROXY=http://127.0.0.1:9` during the offline build (see
   [hermetic-builds.md](./hermetic-builds.md)). Proves lockfile completeness without
   a dedicated air-gapped runner.
3. **Hard network-block (planned)** — GitHub Actions egress restriction or
   `slsa-framework/slsa-github-generator` containerized builder with no outbound
   except pinned registries. Artifact: `hermetic-network-block-report.json` (pass/fail
   log, not a merge blocker).
4. **Vendor fallback** — if registry outage blocks offline builds, commit or cache
   `vendor/` as a release-only input; document digest in provenance predicate.

## L53 upgrade checklist (SLSA Build L3)

- [x] **Switch `release-attestation.yml` to `generator_containerized_slsa3.yml`** (Wave17 Plan 777, PR TBD)
- [ ] Pin build image by digest in generator workflow
- [ ] Ephemeral / single-tenant builder (no shared mutable state)
- [x] Non-forgeable provenance (sigstore re-sign, ephemeral Fulcio key, OIDC issuer `https://token.actions.githubusercontent.com`)
- [x] Transparency log publication (ephemeral Rekor, TUF-pinned)
- [ ] Hermetic inputs: lockfile + vendored deps + `SOURCE_DATE_EPOCH` (L52)
- [x] Cross-link L56 cosign verify on release container (`container-cosign-verify.sh` + `docs/slsa.md`)

## L55 / L56 coupling

- **L55** — `deny.toml` source policy must stay `unknown-registry=deny` before any
  private mirror is added for vendor/airgap; new registries require deny allow-list +
  ADR.
- **L56** — hard path: `cosign sign` + `cosign attest` on GHCR push
  (`container-cosign.yml`); soft `sign-blob` retained; deploy verify in
  `scripts/container-cosign-verify.sh`, `docs/slsa.md`, [ghcr-publish.md](./ghcr-publish.md).

## Commands (local dry-run)

```bash
# Phase 0 — same as hermetic soft
just hermetic

# Phase 1 spike (not CI-required)
cargo vendor
cargo build --locked --offline -p sharecli

# Phase 3 consumer verify (after generator L3 lands)
gh attestation verify <artifact> --owner KooshaPari
```

## Out of scope (this PR)

- Required `hermetic-hard.yml` or network-block merge gate
- Committed `vendor/` tree
- ~~Switching `release-attestation.yml` to `generator_containerized_slsa3.yml`~~ (DONE — Wave17 Plan 777)
- Re-pinning `slsa-framework/slsa-github-generator/.github/workflows/generator_containerized_slsa3.yml@v2` from `@v2` to `@<commit-sha>` (separate hardening PR)

## Wave17 Plan 777 — C06 L53 2 → 3

**Status:** ✅ Shipped. **Workflow:** `.github/workflows/release-attestation.yml`.

### Diff summary

- Replaced `slsa-framework/slsa-github-generator/attest-build-provenance@v1` (L2 action) with
  `slsa-framework/slsa-github-generator/.github/workflows/generator_containerized_slsa3.yml@v2`
  (L3 reusable workflow).
- Removed host-runner `actions/checkout` / `rust-toolchain` / `rust-cache` / `upload-artifact`
  steps (those run inside the L3 generator's ephemeral container).
- Updated `BUILD_MANIFEST.txt` field `runner: ephemeral-container` (L3 invariant).
- Added `slsa_level: 3` and `generator:` line to the manifest.

### C06 scorecard bump

- L53 2 → 3
- L54 stays 2 (hermetic flags wired in release.yml but not yet enforced as hard gate)
- L55/L56 unchanged (already 3)

### Verification on next tagged release

```bash
gh release download v<tag> --pattern '*.intoto.jsonl'
slsa-verifier verify-artifact \
  --provenance-path ./release-artifacts.intoto.jsonl \
  --source-uri github.com/KooshaPari/sharecli \
  --source-tag <tag>
# Expect: PASSED, buildLevel: 3, builder.id contains generator_containerized_slsa3.yml
```

### Soft goal

L53 = 3 with a single, reproducible source (the generator workflow).
The hermetic / netblock pieces (L54 hard, L52) remain deferred — see phase plan above.
