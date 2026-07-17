# lib.rs sprawl modularization plan (soft)

Audit-v38 **C00 L0 / L1** — operator plan for shrinking root `src/lib.rs` into
documented crate boundaries without breaking the CLI or `sharecli util` surface.
This file is **docs-only**; no module moves ship in this lane.

## Problem

`src/lib.rs` currently declares **~160** `pub mod` lines. The audit lane
(`audit/.lane-c00/C00.md`) scores L0/L1 at **2** with a gap:

> Domain logic mixed with 100+ utility modules in `src/lib.rs` — effort: L

The sprawl weakens boundary discipline: product modules (runtime, serve, config)
sit beside parity/expansion utilities (bloom, bip39, dns parsers, crypto
primitives) in one flat namespace.

## Current tiers (today)

| Tier | Role | Example modules | Wired by |
|------|------|-----------------|----------|
| **A — Product core** | CLI + daemon orchestration | `commands`, `config`, `runtime`, `spawn_policy`, `serve_auth`, `serve_lock` | `main.rs`, FR acceptance tests |
| **B — Ops / observability** | Serve-side telemetry + health | `otel`, `http_red`, `metrics`, `audit_log`, `pprof_http`, `health_check` | `commands/serve.rs` |
| **C — Parity / expansion** | Standalone algorithms + format parsers | `bloom`, `keccak`, `dns_zone`, `bip39_mnemonic`, `astar`, `bellman_ford` | `sharecli util` (`util_cmd.rs`), unit tests in-module |
| **D — Workspace crates** | Already extracted | `sharecli-core`, `sharecli-ipc`, `sharecli-fleet`, `spawn-core-sys`, `harness-native`, `sharecli-fuse`, `sharecli-thermal-tui` | `Cargo.toml` workspace members |

Tier **A+B** (~30 modules) is the real product surface. Tier **C** (~130
modules) is the sprawl driver.

## Target crate boundaries

```text
sharecli (binary + thin lib facade)
├── sharecli-runtime      ← ProcessPool, SharedRuntime, spawn_policy (Tier A)
├── sharecli-serve        ← serve_auth, http_red, serve_lock, audit_log (Tier B)
├── sharecli-config       ← Config, project registry, config_watcher (Tier A)
├── sharecli-parity       ← Tier C utilities (optional dep; `util` CLI only)
│
├── sharecli-core         ← Hypervisor + ThermalGate (exists)
├── sharecli-ipc          ← CoalesceCache (exists)
├── sharecli-fleet        ← device registry (exists)
├── spawn-core-sys        ← Zig hot core (exists)
├── harness-native        ← build harness strategies (exists)
├── sharecli-fuse         ← FUSE intercept (exists)
└── sharecli-thermal-tui  ← TUI (exists)
```

### Dependency arrows (soft target)

```text
sharecli → sharecli-runtime, sharecli-serve, sharecli-config
sharecli → sharecli-parity (feature = "util-cli", default off for library users)
sharecli-runtime → sharecli-core, spawn-core-sys, sharecli-config
sharecli-serve → sharecli-runtime, sharecli-config
sharecli-core → sharecli-ipc, sharecli-fuse, sharecli-fleet
sharecli-parity → (no sharecli-runtime dep — leaf crate)
```

**Rule:** Tier C must not depend on Tier A/B. Keeps parity tests isolated and
allows `cargo check -p sharecli-parity` without compiling serve/otel.

## error-envelope.md cross-ref

HTTP error shape is documented in [`error-envelope.md`](error-envelope.md). When
the envelope type lands in code, it belongs in **`sharecli-serve`** (not root
`lib.rs`):

| Concern | Owner crate | Rationale |
|---------|-------------|-----------|
| `ErrorEnvelope` struct + serde | `sharecli-serve` | Only serve handlers emit JSON errors |
| 401 mapping in `serve_auth` | `sharecli-serve` | Already auth-specific |
| OpenAPI `ErrorEnvelope` component | `docs/openapi/serve.yaml` | L2 contract — see error-envelope follow-up #2 |
| CLI `anyhow` errors | `sharecli` binary | NFR-003; not the HTTP envelope |

Root `lib.rs` should **re-export** `sharecli_serve::ErrorEnvelope` only if the
public library API needs it; prefer `sharecli-serve` as the canonical import
path.

## Phased rollout (soft)

| Phase | Scope | Breaking? | Gate |
|-------|-------|-----------|------|
| **0 — Plan** | This doc + scorecard/worklog | No | FR-003 evidence |
| **1 — Facade** | Group Tier C under `pub mod util { pub use … }` in `lib.rs`; keep file paths | No | `cargo test` + `sharecli util --help` |
| **2 — Extract parity** | New `crates/sharecli-parity`; move Tier C files; root re-exports | No (re-export shim) | Workspace `members` + util CLI green |
| **3 — Extract serve** | `crates/sharecli-serve`; move Tier B + `commands/serve.rs` helpers | No | OpenAPI drift + serve integration tests |
| **4 — Extract runtime** | `crates/sharecli-runtime`; slim root `lib.rs` to ~10 re-exports | No | FR-001..005 acceptance |
| **5 — Dep hygiene** | `cargo-deny` / `cargo-machete` ban `sharecli-parity → sharecli-runtime` | No | CI advisory → hard |

Do **not** block FR lanes on Phase 2+; schedule behind dedicated C00 hardening
sprints.

## Module inventory (representative)

Full list: `src/lib.rs` `pub mod` declarations. Sample mapping:

| Module | Tier | Proposed crate |
|--------|------|----------------|
| `runtime`, `spawn_policy`, `proc_table`, `scheduler` | A | `sharecli-runtime` |
| `config`, `config_loader`, `config_watcher`, `config_merger` | A | `sharecli-config` |
| `serve_auth`, `serve_lock`, `http_red`, `audit_log`, `pprof_http` | B | `sharecli-serve` |
| `otel`, `metrics`, `log_sink` | B | `sharecli-serve` or `sharecli-telemetry` (defer) |
| `bloom`, `keccak`, `dns_zone`, `bip39_mnemonic`, `astar` | C | `sharecli-parity` |
| `util_cmd` (binary-only) | — | stays in `sharecli` binary crate |

`util_cmd.rs` imports a **subset** of Tier C (base85, crc64, hash_util, …).
After Phase 2, switch imports to `sharecli_parity::*`.

## Operator checklist

1. **New utility module** — add under `crates/sharecli-parity/src/` (post-Phase 2)
   or behind a `util/` directory pre-Phase 2; never append bare `pub mod` to root
   `lib.rs` without tier tag.
2. **Serve / auth change** — consult [`error-envelope.md`](error-envelope.md);
   envelope types live in serve crate when implemented.
3. **Public API review** — `pub use` in root `lib.rs` should list only Tier A
   runtime exports (`ProcessPool`, `SharedRuntime`, …) per `SPEC.md`.
4. **Bench / perf** — parity crate must not pull serve deps; see
   [`perf-budgets.md`](perf-budgets.md) for hot-path bench ownership (C00 L6).

## Score impact (soft)

| Pillar | Today | After plan (docs) | After Phase 4 (code) |
|--------|-------|-------------------|----------------------|
| L0 Architecture | 2 | 2 | 3 (formal crate graph) |
| L1 Module boundaries | 2 | 2 | 3 (sprawl extracted) |

Cluster C00 stays **70% C** until Phase 2+ lands; this doc closes the L0/L1
**planning** gap on the scorecard Top-3 list (`lib.rs sprawl`).

**Status:** soft plan (Phase 0) · **FR:** FR-003 traceability · **Last sync:** 2026-07-17
