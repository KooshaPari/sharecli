# Homebrew bottle (soft) — sharecli

Audit-v38 **C03** agent install path + **C11 L109** (Homebrew formula). Today
`Formula/sharecli.rb` ships in-tree with a filled darwin bottle digest for
`v0.3.0`. A separate **tap** and automated bump on tag remain soft — this
runbook unblocks agents and maintainers without wiring tap CI yet.

**Release source of truth:** [`.github/workflows/release.yml`](../../.github/workflows/release.yml)
`build` (macOS) → `github-release` (attach on `v*` tag).

## Soft contract (this lane)

| Step | Actor | Action |
|------|-------|--------|
| 1 — Tag release | Maintainer | Push `v*` tag or dispatch `release.yml` with `dry_run=false` + `attach_github_release=true` |
| 2 — Darwin asset | CI | `aarch64-apple-darwin` job emits `sharecli-aarch64-apple-darwin.tar.gz` + `.sha256` |
| 3 — Attach | `github-release` job | Uploads unsigned archives to GitHub Release (parity floor: [`deploy/FINALITY.md`](../deploy/FINALITY.md)) |
| 4 — Formula bump | Agent / maintainer | Update `version`, `url`, `sha256` in `Formula/sharecli.rb` from attached asset |
| 5 — Tap publish | Deferred | Copy formula to `homebrew-sharecli` tap (sketch below) |

Until step 5 lands, agents install via:

```bash
brew install --HEAD Formula/sharecli.rb   # builds from git — no bottle
# or, after formula bump in-repo:
brew install Formula/sharecli.rb          # uses bottle when sha256 matches release asset
```

## SHA placeholder policy

| State | `Formula/sharecli.rb` | Allowed install |
|-------|----------------------|-----------------|
| **Pre-release** | `sha256 "PLACEHOLDER"` + `url` pointing at future tag | `--HEAD` only |
| **Post-attach** | Real `sha256` from release tarball | `brew install` from formula path or tap |
| **Drift** | `version` / `url` / `sha256` mismatch | **Block** — do not merge guessed digests |

Rules:

1. **Never** invent or round-trip a digest — compute from the downloaded release file only.
2. **Always** keep `version`, `url`, and `sha256` aligned with `Cargo.toml` `package.version` and the `v{version}` tag.
3. **Darwin first** — bottle URL targets `sharecli-aarch64-apple-darwin.tar.gz` (matches `release.yml` macOS matrix row). Intel macOS bottle is optional follow-on.
4. **Recompute on every** darwin tarball change (re-tag, clobber upload, rebuild) even if version string is unchanged.
5. Document the bump in PR body with `FR-003` traceability when the change is agent-driven.

Operator recompute (after `github-release` attaches):

```bash
VERSION=0.3.0   # match Cargo.toml / tag
gh release download "v${VERSION}" -p 'sharecli-aarch64-apple-darwin.tar.gz'
shasum -a 256 sharecli-aarch64-apple-darwin.tar.gz
# paste digest into Formula/sharecli.rb; set url to:
#   https://github.com/KooshaPari/sharecli/releases/download/v${VERSION}/sharecli-aarch64-apple-darwin.tar.gz
```

**Evidence:** `v0.3.0` digest filled 2026-07-13 (W4.2). Future tags repeat steps 1–4.

## `release.yml` cross-reference

| Job | Relevant output | Formula field |
|-----|-----------------|---------------|
| `build` (`macos-latest`, `aarch64-apple-darwin`) | `sharecli-aarch64-apple-darwin.tar.gz` | `url` basename |
| `build` (package step) | `sharecli-aarch64-apple-darwin.tar.gz.sha256` | verification only (Homebrew uses inline `sha256`) |
| `github-release` | Attaches `*.tar.gz` + `*.sha256` on `refs/tags/v*` | enables step 4 |
| `publish` (crates.io) | Independent of bottle | `cargo install` parallel channel |

Trigger matrix (bottle-relevant):

- **Tag push** `v*` → build + attach (unsigned).
- **workflow_dispatch** with `attach_github_release=true` and `dry_run=false` → attach without tag push (uses `v{Cargo.toml version}`).
- **Default dispatch** `dry_run=true` → no attach; formula stays on last known good digest.

Failure modes:

| Symptom | Likely cause | Remediation |
|---------|--------------|-------------|
| `brew install` checksum mismatch | Formula sha stale vs re-uploaded asset | Re-download tarball; recompute sha256 |
| No darwin asset on release | `upload-artifact` pin / macOS job failure | Fix CI; re-run `release.yml` (see SCORECARD 2026-07-13 release pin) |
| `PLACEHOLDER` still in formula | Attach never ran for current version | Run attach path or use `--HEAD` until bump lands |

## Homebrew tap sketch (Phase 1 — not wired)

Separate public tap so users avoid `--HEAD` / local formula paths:

```
github.com/KooshaPari/homebrew-sharecli/
├── README.md                 # brew tap KooshaPari/sharecli && brew install sharecli
└── Formula/
    └── sharecli.rb           # copied from repo root Formula/sharecli.rb each release
```

Planned flow:

1. On `v*` attach success, open PR to `homebrew-sharecli` updating `sharecli.rb` (version + url + sha256).
2. README documents `brew tap KooshaPari/sharecli` and `brew upgrade sharecli` ([`auto-update.md`](auto-update.md) L111).
3. Optional later: `brew test-bot` on tap repo; Homebrew/core upstream PR only after sustained release cadence.

**Out of scope (hard):** `brew bottle --json` CI job, multi-arch bottles (linux bottle), codesigned/notarized macOS payloads (L112).

## CI phases

| Phase | Gate | Status |
|-------|------|--------|
| 0 — in-repo formula | `Formula/sharecli.rb` + `head do` | **Done** |
| 1 — soft plan | this doc + deploy matrix link | **Done** (soft) |
| 2 — digest on attach | W4.2 `v0.3.0` darwin sha filled | **Done** |
| 3 — tap repo | `homebrew-sharecli` + README one-liner | **Next** |
| 4 — auto-bump bot | PR on tag to tap + in-repo formula | **Deferred** |

## Operator checklist

1. Prefer `cargo install` / `cargo binstall` / GitHub Release + `.sha256` until tap is live.
2. After each `v*` release, verify darwin tarball exists before bumping formula.
3. Run `brew test ./Formula/sharecli.rb` locally when changing digest.
4. Cross-check [`deploy.md`](../deploy.md) matrix row Homebrew **partial → proven** after tap + digest sync.

## Cross-refs

- Deploy matrix: [`deploy.md`](../deploy.md) (Homebrew row)
- Auto-update channels: [`auto-update.md`](auto-update.md) (L111)
- Native installers (future): [`dmg-msi-packaging.md`](dmg-msi-packaging.md) (L108)
- Agent entrypoint install blurb: [`README.md`](../../README.md)

Soft goal: **C03** agents have a documented, checksum-backed `brew install` path;
**C11 L109** stays **2→3** when tap publish proves one-liner install without `--HEAD`.

**Status:** soft plan (Phase 1) · **FR:** FR-003 traceability · **Last sync:** 2026-07-17
