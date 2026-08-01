# ShareCLI Ghostty Control Plane Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add a durable, non-FUSE-dependent ShareCLI terminal-agent control plane with Ghostty discovery/live I/O, verified harness resume, and crash recovery.

**Architecture:** Extend `sharecli-session` from CRUD records into a WAL-backed observation ledger. Add terminal adapters behind narrow traits, use existing local ShareCLI IPC for authenticated local operations, and preserve a managed-PTY adapter for sessions ShareCLI launches. FUSE reports optional KEXT/FSKit capability and never decides recovery availability.

**Tech Stack:** Rust, rusqlite SQLite WAL, Tokio, existing ShareCLI NDJSON IPC, Ghostty native capability probe, zmx adapter, Clap, macOS process APIs.

---

## Locked file structure

- Modify: `crates/sharecli-session/src/lib.rs` — ledger records, migrations, confidence/recovery policy.
- Create: `crates/sharecli-session/src/ledger.rs` — append-only observation and compaction operations.
- Create: `crates/sharecli-session/src/adapter.rs` — `SurfaceAdapter`, `OutputObserver`, `InputDispatcher`, `LayoutRestorer` traits.
- Create: `crates/sharecli-session/src/resolver.rs` — harness/session-ID evidence resolution.
- Create: `crates/sharecli-session/src/recovery.rs` — bounded, shell-free recovery executor.
- Modify: `crates/sharecli-session/src/rpc.rs` — typed local RPC envelopes.
- Modify: `src/session.rs` — Ghostty/zmx implementations of the adapter traits.
- Modify: `src/main.rs` — `session watch`, `session recover`, `session observe`, `session send` CLI verbs.
- Modify: `crates/sharecli-ipc/src/handler.rs` — session RPC dispatch.
- Create: `tests/session_ledger.rs` — persistence/restart/ambiguity tests.
- Create: `tests/session_recovery.rs` — dry-run/execution/concurrency tests.
- Create: `tests/session_ghostty.rs` — Ghostty capability and degraded-path tests.
- Modify: `crates/sharecli-fuse/build.rs`, `crates/sharecli-fuse/src/backend.rs`, `crates/sharecli-fuse/src/lib.rs`, `crates/sharecli-fuse/src/session_registry.rs` — truthful KEXT -> FSKit -> non-FUSE capability state.

## Task 1: Repair FUSE capability truth before integration

**Files:**

- Modify: `crates/sharecli-fuse/build.rs`
- Modify: `crates/sharecli-fuse/src/backend.rs`
- Modify: `crates/sharecli-fuse/src/lib.rs`
- Modify: `crates/sharecli-fuse/src/session_registry.rs`
- Test: `crates/sharecli-fuse/src/backend.rs`

- [ ] **Step 1: Write failing capability tests**

```rust
#[test]
fn unavailable_never_calls_fuser_mount() {
    assert_eq!(select_backend_with(Capabilities::default()), FuseBackend::Unavailable);
}

#[test]
fn approved_fskit_is_distinct_from_kernel() {
    let cfg = smoke_fuser_config_for_backend(Some(FuseBackend::Fskit));
    assert!(cfg.mount_options.iter().any(|x| matches!(x, MountOption::CUSTOM(v) if v == "backend=fskit")));
}
```

- [ ] **Step 2: Run the focused test before implementation**

Run: `cargo test -p sharecli-fuse backend::tests --locked`

Expected: the capability tests fail because the current fallback is mislabeled and Unavailable still reaches fuser.

- [ ] **Step 3: Make capability selection explicit**

```rust
pub struct FuseCapabilities {
    pub kernel_loaded: bool,
    pub fskit_approved: bool,
}

pub fn select_backend_with(c: FuseCapabilities) -> FuseBackend {
    if c.kernel_loaded { FuseBackend::Kernel }
    else if c.fskit_approved { FuseBackend::Fskit }
    else { FuseBackend::Unavailable }
}
```

`mount_with_session` must return a typed unavailable error for `Unavailable`; it must not call `fuser::mount`. Restore `backend=fskit` only for an actual FSKit fuser request. Preserve the Windows `winfsp::build::winfsp_link_delayload()` block and add MFMount linking in a separate macOS block.

- [ ] **Step 4: Run focused validation**

Run: `cargo test -p sharecli-fuse backend::tests --locked && cargo check -p sharecli-fuse --locked --no-default-features`

Expected: pass; Windows build-script behavior remains present in source.

## Task 2: Add durable observation ledger

**Files:**

- Create: `crates/sharecli-session/src/ledger.rs`
- Modify: `crates/sharecli-session/src/lib.rs`
- Test: `tests/session_ledger.rs`

- [ ] **Step 1: Write failing restart and ambiguity tests**

```rust
#[test]
fn observation_survives_store_reopen() { /* open file db, append, reopen, assert */ }

#[test]
fn heuristic_session_is_not_auto_resumable() { /* assert recovery policy */ }
```

- [ ] **Step 2: Add immutable observation types**

```rust
pub struct SessionObservation {
    pub observed_at: DateTime<Utc>,
    pub surface: SurfaceRecord,
    pub session: Option<AgentSession>,
    pub capability: SurfaceCapabilities,
}
```

Use a `session_observations` table with an autoincrement sequence, `surface_id`, JSON evidence, and timestamp. Keep `sessions` as the materialized latest-known record. Use `BEGIN IMMEDIATE` for compaction and never delete an observation referenced by the latest materialized session.

- [ ] **Step 3: Run tests**

Run: `cargo test -p sharecli-session --locked && cargo test --test session_ledger --locked`

Expected: persistence works across restart; uncertain records remain operator-visible but non-executable.

## Task 3: Define terminal adapter boundary and capability probes

**Files:**

- Create: `crates/sharecli-session/src/adapter.rs`
- Modify: `src/session.rs`
- Test: `tests/session_ghostty.rs`

- [ ] **Step 1: Write contract tests**

```rust
#[tokio::test]
async fn unsupported_ghostty_reports_typed_degraded_capability() { /* ... */ }

#[tokio::test]
async fn stable_surface_id_is_required_for_send() { /* ... */ }
```

- [ ] **Step 2: Define narrow traits**

```rust
#[async_trait]
pub trait SurfaceAdapter {
    async fn capabilities(&self) -> Result<SurfaceCapabilities>;
    async fn discover(&self) -> Result<Vec<SurfaceRecord>>;
}
pub trait InputDispatcher { async fn send(&self, id: &str, bytes: &[u8]) -> Result<()>; }
pub trait OutputObserver { async fn subscribe(&self, id: &str) -> Result<OutputStream>; }
```

`GhosttyAdapter` must return typed unsupported state until an actual native control surface is proven. Clipboard paste remains an explicitly degraded legacy caster, never an `InputDispatcher` implementation.

- [ ] **Step 3: Run tests**

Run: `cargo test --test session_ghostty --locked`

Expected: no fake native capability; stable surface identity is mandatory for input.

## Task 4: Implement harness evidence resolver

**Files:**

- Create: `crates/sharecli-session/src/resolver.rs`
- Modify: `crates/sharecli-session/src/lib.rs`
- Test: `tests/session_recovery.rs`

- [ ] **Step 1: Write resolver matrix tests**

```rust
#[test]
fn codex_recipe_requires_exact_session_id() { /* argv + id -> Exact */ }
#[test]
fn ambiguous_process_never_yields_recipe() { /* -> Unavailable */ }
```

- [ ] **Step 2: Implement evidence ordering**

Use this order: explicit adapter state -> harness state file -> verified argv -> documented CLI inspection -> unavailable. Emit `Exact`, `Corroborated`, `Heuristic`, or `Unavailable`; only the first two may produce a recovery recipe.

- [ ] **Step 3: Run tests**

Run: `cargo test --test session_recovery resolver --locked`

Expected: recipes contain argv vectors and cwd, never shell strings.

## Task 5: Add local IPC and CLI recovery control

**Files:**

- Modify: `crates/sharecli-session/src/rpc.rs`
- Modify: `crates/sharecli-ipc/src/handler.rs`
- Modify: `src/main.rs`
- Test: `tests/session_recovery.rs`

- [ ] **Step 1: Add failing CLI/RPC tests**

```rust
#[test]
fn recover_without_execute_is_dry_run() { /* no process runner calls */ }
#[test]
fn send_rejects_unknown_surface() { /* typed not-found */ }
```

- [ ] **Step 2: Add verbs**

```text
sharecli session watch [--interval-seconds N]
sharecli session observe <surface-id>
sharecli session send <surface-id> [--file PATH]
sharecli session recover [--execute] [--max-parallel N]
```

`recover` defaults to dry run. Execution calls `Command` with a verified argv vector and `current_dir`, bounded by a Tokio semaphore. IPC methods mirror list/inspect/observe/send/recovery.plan/recovery.execute/cancel.

- [ ] **Step 3: Run focused tests**

Run: `cargo test --test session_recovery --locked && cargo test -p sharecli-ipc --locked`

Expected: dry run never launches; execution only launches exact/corroborated records.

## Task 6: Layout restoration and managed PTY integration

**Files:**

- Modify: `src/session.rs`
- Create: `crates/sharecli-session/src/recovery.rs`
- Test: `tests/session_recovery.rs`

- [ ] **Step 1: Write recovery ordering tests**

```rust
#[tokio::test]
async fn executor_limits_parallel_launches_and_records_outcomes() { /* max 2 */ }
#[tokio::test]
async fn unresolved_surface_is_reported_not_guessed() { /* manual outcome */ }
```

- [ ] **Step 2: Implement executor**

The executor restores adapter-supported layout first, then starts recipes using bounded concurrency. It records `Resumed`, `SkippedAmbiguous`, `UnsupportedSurface`, and `LaunchFailed` outcomes in the ledger. zmx sessions use their adapter capabilities; ordinary Ghostty panes use only proven native capabilities.

- [ ] **Step 3: Run integration test**

Run: `cargo test --test session_recovery --locked`

Expected: no session is silently dropped or guessed; results are persistable and renderable.

## Task 7: Operator cockpit and crash dogfood

**Files:**

- Modify: `src/commands/serve.rs`
- Modify: `src/dashboard.html`
- Modify: desktop/tray IPC consumers as required
- Test: `tests/session_recovery.rs`
- Test: `tests/e2e_chaos_recovery.rs`

- [ ] **Step 1: Add dashboard/IPC fixture tests**

Add fixtures with active, resumable, ambiguous, unsupported, and failed sessions. Assert no recipe/session ID is exposed beyond local authenticated IPC.

- [ ] **Step 2: Surface state**

Expose counts and per-session recovery outcomes in existing `monitoring.report`/dashboard IPC shapes. Add a recovery action that requires explicit execute confirmation.

- [ ] **Step 3: Dogfood non-FUSE crash recovery**

Run: `cargo test --test e2e_chaos_recovery --locked`

Expected: after a controlled daemon/terminal simulation restart, the ledger reconstructs a dry-run plan and resumes only verified fixture sessions without any FUSE mount.

## Plan self-review

- Spec coverage: Tasks 1-7 cover optional FUSE truth, ledger, Ghostty capability isolation, live I/O, resolver, executor, layout, and operator visibility.
- Placeholder scan: all implementation tasks name files, APIs, tests, and commands.
- Type consistency: `SurfaceRecord`, `AgentSession`, `ResolutionConfidence`, and argv-based `ResumeRecipe` remain the canonical cross-task types.
