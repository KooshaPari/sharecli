# Wave 19 Gap Remediation Spec

**Repo**: `sharecli` | **Date**: 2026-08-28 | **Schema**: audit-v38-ext  
**Predecessor**: Wave 18 (T-900..T-1060) | **Task Range**: T-1100..T-1199  
**Head SHA**: `6df5699` | **Current Score**: 92.0% weighted A

---

## Executive Summary

Wave 19 targets the 5 remaining operational gaps from Wave 18 that are not yet addressed by existing tasks. These focus on making local development infrastructure actually functional: a working eval harness, a hard cosign gate, a local OTel collector, multi-agent scale testing, and dev mode verification. Together these will lift C08 from 83.3% toward 90%+ and close 2 soft gates (C11/C05).

| Gap ID | Cluster | Pillar | Current | Target | Task IDs |
|--------|---------|--------|---------|--------|----------|
| C08 Eval Harness | C08 | L76 | 2 (stub) | 3 (functional) | T-1100..T-1120 (T-1110/T-1120 DONE) |
| C11 Container Cosign | C11 | L54 | 2 (soft) | 3 (hard gate) | T-1130..T-1140 |
| C05 OTel Collector | C05 | L57 | 2 (partial) | 3 (local operational) | T-1150..T-1160 |
| C03 Multi-Agent Scale | C03 | L30.9 | 3 (protocol) | 3 (scale-tested) | T-1170..T-1180 |
| C07 Dev Mode | C07 | L70 | 2 (partial) | 3 (verified) | T-1190 |

**Estimated Score Impact**: +4 points (C08 +3, C11 +1, C05 +1) → 93.5%+ weighted  
**Total Tasks**: 5 gaps, 10 tasks (T-1100..T-1199) — 2 already DONE (T-1110, T-1120)

---

## Gap 1: C08 Eval Harness (T-1100..T-1120) — Partially DONE

### Problem Statement

C08 L76 (Harbor Soft Gate) is currently a stub-only test (`tests/c08_harbor_soft_stub.rs`) that verifies documentation exists but does not actually run any benchmarks. The eval corpus (`docs/eval/corpus/scenarios/`) exists but is not wired into a runnable harness. There is no `sharecli eval` subcommand, no CI workflow that runs benchmarks as a gate, and no eval config file. This means Harbor soak evidence cannot be generated locally and L76 remains at score 2.

### Current State (Post T-1110/T-1120)

- **T-1110 DONE**: `.github/workflows/eval.yml` CI workflow exists and runs benchmarks with artifact upload
- **T-1120 DONE**: `tests/c08_eval_harness_gate.rs` has 12 assertion groups that pass
- **T-1100 STILL NEEDED**: The `sharecli eval` subcommand and `eval.yaml` config file are not yet implemented

### Solution Approach

1. ~~Implement a `sharecli eval` subcommand that runs criterion benchmarks against the eval corpus~~ (T-1100)
2. ~~Create an `eval.yaml` configuration file defining benchmark scenarios, thresholds, and comparison baselines~~ (T-1100)
3. ~~Add a CI workflow (`eval.yml`) that runs the eval harness, stores benchmark results as artifacts, and compares against baselines~~ (T-1110 DONE)
4. ~~Write integration tests verifying the eval harness is operational~~ (T-1120 DONE)

### Files to Create

| File | Purpose |
|------|---------|
| `src/commands/eval.rs` | Eval subcommand implementation — loads `eval.yaml`, runs criterion benchmarks, emits JSON report |
| `eval.yaml` | Eval configuration — benchmark definitions (healthz latency, config load throughput, process list throughput), thresholds, baselines |
| `.github/workflows/eval.yml` | CI workflow — runs `cargo bench` + `sharecli eval`, uploads benchmark artifacts, compares to baseline |
| `tests/c08_eval_harness_gate.rs` | Integration test — verifies eval subcommand exists, runs, and produces valid JSON output |

### Files to Modify

| File | Change |
|------|--------|
| `src/main.rs` | Wire `Commands::Eval` variant into clap |
| `src/lib.rs` | Export `eval` module |
| `tests/c08_harbor_soft_stub.rs` | Update to reference functional harness instead of stub |
| `COMPREHENSIVE_AUDIT_SCORECARD.md` | Update L76 evidence |

### Test Criteria

1. `sharecli eval --list` lists all benchmark scenarios from `eval.yaml`
2. `sharecli eval --benchmark healthz` runs the healthz latency benchmark and outputs JSON with p50/p95/p99 latencies
3. `sharecli eval --threshold` exits non-zero if any benchmark exceeds defined threshold
4. `tests/c08_eval_harness_gate.rs` passes with 3 assertion groups: subcommand wiring, benchmark execution, threshold enforcement
5. CI workflow runs on PR and stores `bench-results.json` as artifact

### Effort Estimate

**M** (Medium) — ~3-4 hours  
- Eval subcommand: 1.5h (clap wiring + YAML parse + criterion runner)
- eval.yaml: 0.5h (3-4 benchmark definitions)
- CI workflow: 1h (workflow + artifact upload)
- Tests: 1h (gate test + verification)

### Dependencies

- `criterion` crate already in `[dev-dependencies]` (used by `benches/`)
- `serde_yaml` or `serde_json` for config parsing (add `serde_yaml` dep)
- No blocking dependencies on other Wave 19 tasks

---

## Gap 2: C11 Container Cosign Hard Gate (T-1130..T-1140)

### Problem Statement

C11 L54 (Container Cosign) currently has a hard gate via `workflows/container-cosign.yml` (scored at 3 in the scorecard), but the soft gate path (`ci/c06-cosign-soft` branch) still exists and is used as a fallback. The audit identified that the cosign workflow needs to be promoted from a soft path to a mandatory hard gate — meaning PRs that fail cosign verification must not be mergeable. Currently the soft path allows bypass.

### Solution Approach

1. Promote `cosign-soft.yml` to `cosign.yml` as a hard gate in the CI pipeline
2. Remove the soft gate fallback branch entirely
3. Add a test that verifies cosign verification is mandatory (cannot be bypassed)
4. Update the CI matrix to require cosign check on all container builds

### Files to Create

| File | Purpose |
|------|---------|
| `tests/c11_cosign_gate.rs` | Integration test — verifies cosign verification is enforced and cannot be bypassed via env vars or feature flags |
| `.github/workflows/cosign-hard-gate.yml` | New hard gate workflow — runs on PRs that touch `Containerfile` or `*.yml` container paths; fails if cosign check is missing |

### Files to Modify

| File | Change |
|------|--------|
| `.github/workflows/container-cosign.yml` | Remove soft gate fallback path; add `if: always()` to ensure execution; add `concurrency` to prevent parallel cosign runs |
| `.github/workflows/cosign-soft.yml` | Delete or redirect to `cosign.yml` |
| `CODEOWNERS` | Ensure container path changes require review from cosign owners |
| `COMPREHENSIVE_AUDIT_SCORECARD.md` | Update L54 evidence |

### Test Criteria

1. `tests/c11_cosign_gate.rs` verifies:
   - `COSIGN_BYPASS` env var is not honored (hard gate cannot be skipped)
   - `container-cosign.yml` exists and contains `if: always()` or equivalent
   - No soft gate workflow is referenced from CI matrix
2. CI workflow fails on PR if cosign check is absent
3. No path exists to merge a container PR without cosign verification

### Effort Estimate

**S** (Small) — ~1-2 hours  
- Hard gate workflow: 0.5h
- Remove soft gate: 0.5h
- Test: 0.5h

### Dependencies

- Requires `cosign` binary available in CI (already present via `sigstore/cosign-installer`)
- No blocking dependencies on other Wave 19 tasks

---

## Gap 3: C05 OTel Local Collector (T-1150..T-1160)

### Problem Statement

C05 L57 (OTel Multi-hop) is scored at 2 (partial) because trace context propagation is incomplete and there is no local OTel collector running. Developers cannot test OTLP trace export locally because there is no collector to receive spans. The `src/otel.rs` module exists but has no local development path — it only works against a remote collector. This blocks local verification of distributed tracing flows.

### Solution Approach

1. Create a `docker-compose.yml` that runs a local OpenTelemetry Collector (OTLP receiver + Prometheus exporter + debug exporter)
2. Create a `scripts/otel-collector.sh` helper script to start/stop/status the collector
3. Write a test that verifies the collector is reachable and spans can be exported

### Files to Create

| File | Purpose |
|------|---------|
| `docker-compose.yml` | Local OTel collector stack — OTLP gRPC/HTTP receiver on 4317/4318, Prometheus exporter on 8889, debug exporter to stdout, config via `otel-collector-config.yml` |
| `otel-collector-config.yml` | OTel Collector configuration — receivers (otlp), processors (batch, memory_limiter), exporters (prometheus, logging, debug) |
| `scripts/otel-collector.sh` | Helper script — `start`, `stop`, `status`, `logs` subcommands; checks Docker availability; health-checks collector readiness |
| `tests/c05_otel_collector_gate.rs` | Integration test — verifies collector config is valid YAML, docker-compose.yml is parseable, script exists and is executable, OTLP port 4317 is configurable |

### Files to Modify

| File | Change |
|------|--------|
| `justfile` | Add `otel-start`, `otel-stop`, `otel-status` recipes |
| `docs/ops/LOCAL_LOOP_BUDGETS.md` | Add OTel collector startup time budget |
| `AGENTS.md` | Add OTel local dev instructions to key files section |
| `COMPREHENSIVE_AUDIT_SCORECARD.md` | Update L57 evidence |

### Test Criteria

1. `scripts/otel-collector.sh start` starts the collector and health check passes within 30s
2. `scripts/otel-collector.sh status` reports running when collector is up
3. `scripts/otel-collector.sh stop` cleanly shuts down the collector
4. `tests/c05_otel_collector_gate.rs` verifies:
   - `otel-collector-config.yml` parses as valid YAML with required keys (receivers, exporters, service)
   - `docker-compose.yml` defines an `otel-collector` service
   - `scripts/otel-collector.sh` is executable and accepts `start`/`stop`/`status` subcommands
5. `just otel-start` + `just otel-status` completes successfully

### Effort Estimate

**M** (Medium) — ~3-4 hours  
- docker-compose.yml + config: 1h
- Shell script: 1h
- Tests: 1h
- justfile + docs: 0.5h

### Dependencies

- Docker/Podman must be available locally (documented prerequisite)
- No blocking dependencies on other Wave 19 tasks

---

## Gap 4: C03 Multi-Agent Scale Test (T-1170..T-1180)

### Problem Statement

C03 L30.9 (Claim-Lock Protocol) is scored at 3 with a documented protocol in `AGENTS.md`, but the protocol has never been tested at scale. The claim-lock mechanism assumes file-level ownership but there is no test that spawns multiple concurrent agents and verifies that (a) two agents cannot claim the same file simultaneously, (b) a released lock can be re-claimed, and (c) stale locks are detected and cleaned up. This is a theoretical correctness concern that could cause silent data corruption in multi-agent workflows.

### Solution Approach

1. Create a multi-agent scale test that spawns concurrent claim attempts
2. Verify mutual exclusion, lock release + re-claim, and stale lock detection
3. Use thread spawning with `std::thread` or `tokio::spawn` to simulate concurrent agents

### Files to Create

| File | Purpose |
|------|--------|
| `tests/c03_multi_agent_scale_gate.rs` | Integration test — spawns N concurrent agents attempting to claim files, verifies mutual exclusion, lock lifecycle, and stale detection |

### Files to Modify

| File | Change |
|------|--------|
| `COMPREHENSIVE_AUDIT_SCORECARD.md` | Update L30.9 evidence to include scale test |

### Test Criteria

1. **Mutual Exclusion**: Spawning 10 concurrent claim attempts on the same file results in exactly 1 successful claim; 9 receive "already claimed" errors
2. **Lock Lifecycle**: Agent A claims file X → Agent A releases → Agent B claims file X succeeds
3. **Stale Lock Detection**: Agent A claims file X → Agent A "crashes" (no release) → Stale timeout elapses → Agent B claims file X succeeds with stale-detected flag
4. **Scale Threshold**: Test completes within 10 seconds with 50 concurrent agents across 10 files
5. All assertions use `traceability!()` macro for FR-003 mapping

### Effort Estimate

**S** (Small) — ~2 hours  
- Test implementation: 1.5h
- Verification: 0.5h

### Dependencies

- Existing `tests/coordination.rs` provides coordination primitives to build upon
- No blocking dependencies on other Wave 19 tasks

---

## Gap 5: C07 Dev Mode Verification (T-1190)

### Problem Statement

C07 L70 (Cross-Platform Build) is scored at 2 because `sharecli dev` local development mode is unverified. While the justfile has dev recipes and the devcontainer is configured, there is no automated test that verifies the full dev mode workflow: build → start → serve → health check → hot reload → stop. Without this test, developers cannot be confident that the local dev experience works end-to-end.

### Solution Approach

1. Create a test that exercises the `sharecli dev` workflow
2. Verify: build succeeds, server starts, health endpoint responds, config hot-reload triggers, server stops cleanly
3. Use process spawning with health check polling

### Files to Create

| File | Purpose |
|------|---------|
| `tests/c07_dev_mode_gate.rs` | Integration test — spawns `sharecli dev`, polls health endpoint, verifies hot reload with config touch, confirms clean shutdown |

### Files to Modify

| File | Change |
|------|--------|
| `COMPREHENSIVE_AUDIT_SCORECARD.md` | Update L70 evidence to include dev mode verification |
| `docs/ops/LOCAL_LOOP_BUDGETS.md` | Add dev mode startup budget |

### Test Criteria

1. `sharecli dev` starts within 5 seconds (local loop budget compliance)
2. Health endpoint (`/health`) returns HTTP 200 within 2 seconds of startup
3. Config file touch triggers hot-reload (detected via `/health/processes` timestamp change)
4. `SIGTERM` / graceful shutdown completes within 3 seconds
5. Exit code is 0 on clean shutdown
6. Test is marked `#[cfg(not(target_os = "windows"))]` or uses process-group termination on Windows

### Effort Estimate

**S** (Small) — ~2 hours  
- Test implementation: 1.5h
- Verification: 0.5h

### Dependencies

- Requires `sharecli` binary to be buildable (`cargo build`)
- Requires port availability (use random port via `--port 0` or test-specific port)
- No blocking dependencies on other Wave 19 tasks

---

## Cross-Cutting Concerns

### Score Impact Projection

| Cluster | Pillar | Current | After Wave 19 | Points Gained |
|---------|--------|---------|---------------|---------------|
| C08 | L76 (Harbor Soft Gate) | 2 | 3 | +1 |
| C11 | L54 (Container Cosign) | 3 | 3 | 0 (already hard; soft path removal is hygiene) |
| C05 | L57 (OTel Multi-hop) | 2 | 3 | +1 |
| C03 | L30.9 (Claim-Lock) | 3 | 3 | 0 (already at 3; scale test adds confidence) |
| C07 | L70 (Cross-Platform) | 2 | 3 | +1 |

**Net score gain**: +3 points across 3 clusters  
**Projected weighted**: 92.0% → ~93.5% A

### Effort Summary

| Gap | Task IDs | Effort | Parallelizable |
|-----|----------|--------|----------------|
| C08 Eval Harness | T-1100..T-1120 | M | Yes |
| C11 Cosign Hard Gate | T-1130..T-1140 | S | Yes |
| C05 OTel Collector | T-1150..T-1160 | M | Yes |
| C03 Multi-Agent Scale | T-1170..T-1180 | S | Yes |
| C07 Dev Mode | T-1190 | S | Yes |
| **Total** | | **~10-12h** | **All parallel** |

### Recommended Execution Order

All 5 gaps are independent and can be worked in parallel. If single-agent execution is required:

1. **T-1130** (C11 Cosign) — smallest, unblocks CI hygiene
2. **T-1170** (C03 Multi-Agent) — no file creation, test-only
3. **T-1190** (C07 Dev Mode) — test-only, no new deps
4. **T-1100** (C08 Eval) — requires new subcommand + workflow
5. **T-1150** (C05 OTel) — requires Docker validation

### Verification

After all tasks complete:

1. `just test` — full suite passes
2. `just test-nextest` — parallel suite passes  
3. `cargo llvm-cov --lib` — coverage remains >= current pin
4. All 5 new gate tests pass individually
5. Scorecard updated: C08 ≥ 87.5%, C05 ≥ 100%, C07 ≥ 100%

---

## Risk Register

| Risk | Likelihood | Impact | Mitigation |
|------|-----------|--------|------------|
| Docker unavailable for OTel collector | Medium | Low | Gate test validates config/YAML only; Docker required only for `start` subcommand |
| Port conflicts in dev mode test | Low | Low | Use random port allocation |
| Criterion benchmarks non-deterministic | Medium | Medium | Use `--baseline` flag; allow ±10% variance in CI |
| Cosign hard gate blocks legitimate soft-path users | Low | High | Document migration path; keep soft gate as commented-out template |
| Multi-agent lock contention under heavy load | Low | Medium | Use `tokio::sync::Semaphore` with bounded concurrency |

---

**Spec Version**: 1.0  
**Created**: 2026-08-28  
**Status**: READY for claiming  
**Next**: Update `WORK_DAG.md` with T-1100..T-1199 task rows
