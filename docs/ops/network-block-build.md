# Network-blocked hermetic build (soft)

Audit-v38 **C06 L54** — soft plan for compile/test with **no registry egress**
after a one-time dependency fetch. Complements the fetch-then-offline contract in
[hermetic-builds.md](./hermetic-builds.md); hard SLSA L3 phases live in
[slsa-l3-plan.md](./slsa-l3-plan.md).

## Soft contract (this lane)

| Step | Network | Command / env |
|------|---------|---------------|
| 1 — Warm cache | Allowed once | `cargo fetch --locked` |
| 2 — Offline compile | **Blocked** | `CARGO_NET_OFFLINE=1 cargo check --locked --offline -p sharecli` |
| 3 — Offline build | **Blocked** | `cargo build --locked --offline -p sharecli` (same as `just hermetic` step 2) |
| 4 — CI stand-in (proxy) | Proxy poison | `hermetic-soft.yml` sets `HTTP_PROXY=http://127.0.0.1:9` during offline build |
| 5 — CI netblock probe | Fetch then offline | [`.github/workflows/netblock-soft.yml`](../../.github/workflows/netblock-soft.yml) · [`scripts/ci/netblock_check.sh`](../../scripts/ci/netblock_check.sh) |

`CARGO_NET_OFFLINE=1` tells Cargo to refuse registry access even if a proxy or
stale config would allow it. Pair with `--offline` on build/check commands.

Local probe: `scripts/ci/netblock_check.sh` (fetch if needed, then offline check).

## Vendoring sketch (Phase 1 — not required)

If registry cache or Actions `rust-cache` is unreliable, vendor deps for
air-gapped or fully blocked runners:

```bash
cargo vendor vendor/
# .cargo/config.toml (local or CI-generated — do not commit until policy lands):
# [source.crates-io]
# replace-with = "vendored-sources"
# [source.vendored-sources]
# directory = "vendor"
cargo build --locked --offline -p sharecli
```

**Policy (deferred):** committed `vendor/` vs release-only artifact; digest pin in
provenance predicate; `deny.toml` allow-list before any private mirror (L55).

## Failure modes

| Symptom | Likely cause | Remediation |
|---------|--------------|-------------|
| `failed to fetch` / `network disabled` | No cached crates | `cargo fetch --locked` on a networked host |
| `lock file needs to be updated` | `Cargo.toml` changed without lock refresh | `cargo update -p <crate>` + commit `Cargo.lock` |
| Offline OK locally, CI fails | Cold `rust-cache` on runner | Re-run after fetch step; vendor spike if chronic |

## Hard follow-up (out of scope)

- Required network-block merge gate (`hermetic-hard.yml`)
- Committed `vendor/` tree or org mirror
- SLSA generator L3 containerized builder — see [slsa-l3-plan.md](./slsa-l3-plan.md)

Soft goal: **L54 score 3** — required `netblock` job in `ci.yml` + `ci-success`; soft mirrors
remain advisory (`netblock-soft.yml`, `hermetic-soft.yml`).

**Status:** hard CI gate live · **FR:** FR-003 traceability · **Last sync:** 2026-07-19
