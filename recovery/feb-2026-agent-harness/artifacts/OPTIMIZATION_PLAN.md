# Agent-Harness Optimization Plan
## Deep Research & Implementation Roadmap

**Document Version:** 1.0  
**Date:** 2026-02-07  
**Status:** Planning Phase

---

## Executive Summary

This document presents a comprehensive optimization strategy for agent-harness, derived from research into:
- Build system caching (Bazel, Buck, sccache, ccache)
- Subprocess memoization utilities (bkt, bash-cache)
- Incremental computation frameworks (Turbopack, Pants, Adapton)
- Process pool management patterns
- Cache coherence protocols (MESI, MOESI)
- Content-addressable storage systems

The key insight: **multi-agent shell execution** generalizes to **concurrent command execution with shared state** — a well-studied domain in build systems and distributed computing.

---

## Part 1: Input/Output Caching Enhancements

### 1.1 Content-Addressable Storage (CAS)

**Current State:** Cache keys are SHA256 hashes of (command + args + CWD [+ git state])

**Enhancement:** Full content-addressable storage system

```
┌─────────────────────────────────────────────────────────────────┐
│                    Content-Addressable Cache                     │
├─────────────────────────────────────────────────────────────────┤
│  Action Cache (AC)                                               │
│  ┌──────────────────────────────────────────────────────────┐   │
│  │ Key: SHA256(command + args + input_digests + env_vars)   │   │
│  │ Value: ActionResult { output_digests[], exit_code,       │   │
│  │                       stdout_digest, stderr_digest }     │   │
│  └──────────────────────────────────────────────────────────┘   │
│                                                                  │
│  Content Store (CAS)                                             │
│  ┌──────────────────────────────────────────────────────────┐   │
│  │ Key: SHA256(content)                                      │   │
│  │ Value: Raw bytes (compressed with zstd)                   │   │
│  └──────────────────────────────────────────────────────────┘   │
└─────────────────────────────────────────────────────────────────┘
```

**Implementation:**

```rust
// Proposed CAS structure (Rust addition to FUSE server or standalone daemon)
struct ContentStore {
    path: PathBuf,
    // Content storage: var/cas/objects/{aa}/{bb}/{hash}
}

struct ActionCache {
    path: PathBuf,
    // Action cache: var/cache/actions/{hash}.json
}

#[derive(Serialize, Deserialize)]
struct ActionKey {
    command: String,
    args: Vec<String>,
    input_digests: BTreeMap<String, String>,  // path → SHA256
    env_filter: BTreeMap<String, String>,     // relevant env vars
    cwd_relative: bool,
}

#[derive(Serialize, Deserialize)]
struct ActionResult {
    output_digests: BTreeMap<String, String>,  // path → SHA256
    stdout_digest: String,
    stderr_digest: String,
    exit_code: i32,
    execution_time_ms: u64,
    cache_time: DateTime<Utc>,
}
```

**Benefits:**
- Identical outputs deduplicated across different commands
- Partial cache hits (reuse individual output files)
- Cross-machine cache sharing capability
- Audit trail of all cached artifacts

### 1.2 Input Dependency Tracking

**Current State:** Commands are cached based on arguments only; file changes require TTL expiry

**Enhancement:** Automatic input dependency inference

```
┌─────────────────────────────────────────────────────────────────┐
│                   Input Dependency Tracker                       │
├─────────────────────────────────────────────────────────────────┤
│  Mode 1: Manifest-based (pre-computed)                          │
│    - Record file accesses via strace/dtrace on first run        │
│    - Store manifest: command → [input files]                    │
│    - On subsequent runs, hash only manifest files               │
│                                                                  │
│  Mode 2: Filesystem-level (runtime)                             │
│    - FUSE layer records all read() operations per command       │
│    - Build dependency graph automatically                        │
│    - Invalidate cache when any dependency changes               │
│                                                                  │
│  Mode 3: Language-aware (static analysis)                       │
│    - Parse import/require statements                            │
│    - Track only semantically-relevant dependencies              │
│    - Ignore timestamp-only changes (mtime normalization)        │
└─────────────────────────────────────────────────────────────────┘
```

**Implementation Strategy:**

```bash
# Phase 1: Extend FUSE to track reads per process tree
# In fs.rs, add:
struct ReadTracker {
    agent: String,
    files_read: DashSet<PathBuf>,
    files_written: DashSet<PathBuf>,
}

# Phase 2: Generate input digests from tracked reads
fn compute_action_key(tracker: &ReadTracker, cmd: &[String]) -> ActionKey {
    let input_digests = tracker.files_read.iter()
        .filter(|p| !is_generated_file(p))  // Skip build outputs
        .map(|p| (p.to_string(), hash_file(p)))
        .collect();
    // ...
}
```

### 1.3 Stale-While-Revalidate Pattern

**Inspiration:** bkt's `--stale` flag, HTTP cache-control headers

**Current State:** Cache is either fresh or expired; expired = synchronous re-execution

**Enhancement:** Background refresh while serving stale data

```
Timeline without stale-while-revalidate:
  T=0      T=30s (TTL)    T=31s (request)
  [EXEC]───────────────────[WAIT...EXEC]────→
   ↑ fast                   ↑ slow (blocking)

Timeline with stale-while-revalidate (stale=10s, ttl=30s):
  T=0      T=20s         T=25s (request)    T=30s
  [EXEC]───────[stale]───[RETURN cached]─────────→
                          └──[BG REFRESH]───[done]

  T=30s (next request)
  [RETURN fresh from BG]
```

**Implementation:**

```bash
# In harness::strategy::coalesce()
harness::strategy::coalesce_swr() {
    local stale_threshold="$6"  # New parameter
    
    # ... acquire lock, check cache ...
    
    if [[ "$age" -lt "$ttl" ]]; then
        # Fresh: return immediately
        if [[ "$age" -gt "$stale_threshold" && "$stale_threshold" -gt 0 ]]; then
            # Fresh but stale: trigger background refresh
            harness::_background_refresh "$cache_key" "$real_cmd" "$@" &
        fi
        cat "$out"; cat "$err" >&2
        return "$(cat "$rc")"
    fi
    
    # ... execute synchronously ...
}

harness::_background_refresh() {
    local cache_key="$1"; shift
    local real_cmd="$1"; shift
    
    # Non-blocking lock attempt
    local lock="${HARNESS_LOCKS}/${cache_key}.refresh"
    exec 201>"$lock"
    flock -n 201 || return 0  # Another refresh in progress
    
    # Execute and update cache
    "$real_cmd" "$@" > "${out}.new" 2> "${err}.new"
    echo $? > "${rc}.new"
    
    # Atomic swap
    mv "${out}.new" "$out"
    mv "${err}.new" "$err"
    mv "${rc}.new" "$rc"
    
    flock -u 201
}
```

### 1.4 Hierarchical Cache Layers

**Inspiration:** Bazel's multi-layer caching, CPU cache hierarchy

```
┌─────────────────────────────────────────────────────────────────┐
│                    Cache Hierarchy                               │
├─────────────────────────────────────────────────────────────────┤
│                                                                  │
│  L1: In-Memory (per-process)                                    │
│  ├── Latency: ~1μs                                              │
│  ├── Size: 100MB                                                │
│  ├── Scope: Current harness process                             │
│  └── Use: Repeated identical commands in tight loops            │
│                                                                  │
│  L2: Shared Memory (/dev/shm)                                   │
│  ├── Latency: ~10μs                                             │
│  ├── Size: 1GB                                                  │
│  ├── Scope: All agents on same machine                          │
│  └── Use: Cross-agent deduplication                             │
│                                                                  │
│  L3: Disk (var/cache)                                           │
│  ├── Latency: ~1ms (SSD)                                        │
│  ├── Size: 10GB                                                 │
│  ├── Scope: Persistent across reboots                           │
│  └── Use: Long-term caching, large outputs                      │
│                                                                  │
│  L4: Remote (optional, future)                                  │
│  ├── Latency: ~50ms                                             │
│  ├── Size: Unlimited                                            │
│  ├── Scope: Team-wide / CI                                      │
│  └── Protocol: HTTP/gRPC (Bazel remote-apis compatible)         │
│                                                                  │
└─────────────────────────────────────────────────────────────────┘
```

**Implementation Sketch:**

```rust
// Add to FUSE daemon or new cache daemon
trait CacheLayer: Send + Sync {
    fn get(&self, key: &str) -> Option<CachedResult>;
    fn put(&self, key: &str, result: &CachedResult);
    fn delete(&self, key: &str);
}

struct HierarchicalCache {
    layers: Vec<Box<dyn CacheLayer>>,
}

impl HierarchicalCache {
    fn get(&self, key: &str) -> Option<CachedResult> {
        for (i, layer) in self.layers.iter().enumerate() {
            if let Some(result) = layer.get(key) {
                // Promote to faster layers
                for faster in &self.layers[..i] {
                    faster.put(key, &result);
                }
                return Some(result);
            }
        }
        None
    }
}
```

---

## Part 2: Concurrent Execution Optimizations

### 2.1 Speculative Execution

**Inspiration:** CPU branch prediction, Pants build system

**Concept:** Predict likely outcomes and execute speculatively

```
Scenario: Agent A runs "npm install" while Agent B runs "npm test"

Without speculation:
  A: npm install ────────────────────→ done
  B:              [waiting for lock]   npm test ──→ done
  Total time: 30s + 10s = 40s

With speculation:
  A: npm install ────────────────────→ done
  B: [speculative npm test with cached node_modules] ──→ verify ──→ done
     (likely same as A's result, so verification succeeds)
  Total time: max(30s, 10s + verify) ≈ 31s
```

**Implementation Approach:**

```bash
# In rules.conf, add speculative hints:
# npm:test  coalesce ttl=30 speculate_on=npm:install

harness::strategy::speculative() {
    local depends_on="$1"  # Command this speculation depends on
    local real_cmd="$2"; shift 2
    
    # Check if dependency is currently executing
    local dep_lock="${HARNESS_LOCKS}/${depends_on}.lock"
    if flock -n 200 "$dep_lock" 2>/dev/null; then
        flock -u 200
        # Dependency not running, execute normally
        exec "$real_cmd" "$@"
    fi
    
    # Dependency is running — speculate using cached state
    local speculative_out=$(mktemp)
    local speculative_rc
    
    # Execute with CoW snapshot of current state
    (
        export HARNESS_SPECULATIVE=1
        "$real_cmd" "$@"
    ) > "$speculative_out" 2>&1
    speculative_rc=$?
    
    # Wait for dependency to finish
    flock -s 200 "$dep_lock"
    
    # Verify speculation was valid (compare file hashes)
    if harness::_verify_speculation "$depends_on"; then
        cat "$speculative_out"
        return $speculative_rc
    else
        # Speculation invalid, re-execute
        harness::log WARN "Speculation miss for ${real_cmd}, re-executing"
        exec "$real_cmd" "$@"
    fi
}
```

### 2.2 Intelligent Command Coalescing

**Current State:** Commands coalesce purely on argument matching

**Enhancement:** Semantic coalescing based on command behavior

```
┌─────────────────────────────────────────────────────────────────┐
│                 Semantic Coalescing Rules                        │
├─────────────────────────────────────────────────────────────────┤
│                                                                  │
│  Rule 1: Idempotent Commands                                    │
│  ├── git status, git diff, git log                              │
│  ├── ls, find, grep (without -exec)                             │
│  └── All reads → Can be served from single execution            │
│                                                                  │
│  Rule 2: Monotonic Commands                                     │
│  ├── npm install (only adds, never removes)                     │
│  ├── apt update                                                  │
│  └── Later result subsumes earlier → Cache latest only          │
│                                                                  │
│  Rule 3: Commutative Commands                                   │
│  ├── touch file1; touch file2 ≡ touch file2; touch file1       │
│  └── Can be reordered for better batching                       │
│                                                                  │
│  Rule 4: Subsumption                                            │
│  ├── "ruff check ." subsumes "ruff check src/"                  │
│  ├── "git diff" subsumes "git diff -- file.py"                  │
│  └── Broader command result contains narrower                    │
│                                                                  │
└─────────────────────────────────────────────────────────────────┘
```

**Implementation:**

```bash
# Extended rules.conf format:
# COMMAND  STRATEGY  semantics=idempotent|monotonic|commutative  subsumes=pattern

# Example:
# git:diff      coalesce ttl=5 semantics=idempotent subsumes="git:diff --*"
# ruff:check    coalesce ttl=30 semantics=idempotent subsumes="ruff:check *"

harness::check_subsumption() {
    local running_cmd="$1"
    local pending_cmd="$2"
    local subsumes_pattern="$3"
    
    # Check if running command's scope includes pending command's scope
    # This requires parsing the actual file arguments
    # ...
}
```

### 2.3 Command Queue Prioritization

**Current State:** FIFO queue when concurrency limits hit

**Enhancement:** Priority queue based on command characteristics

```
Priority Factors:
┌────────────────────────────────────────────────────────────────┐
│  Factor                  │ Weight │ Rationale                  │
├─────────────────────────┼────────┼────────────────────────────┤
│  User-interactive (TTY)  │ +100   │ Human waiting              │
│  Fast commands (<1s avg) │ +50    │ Quick feedback             │
│  Critical path           │ +30    │ Blocks other work          │
│  Background/CI           │ +0     │ Can wait                   │
│  Already cached          │ +20    │ Will return instantly      │
│  Resource-heavy          │ -20    │ May starve others          │
└────────────────────────────────────────────────────────────────┘
```

### 2.4 Work Stealing & Load Balancing

**Inspiration:** Fork-join frameworks, GNU Make jobserver

```
┌─────────────────────────────────────────────────────────────────┐
│                     Jobserver Integration                        │
├─────────────────────────────────────────────────────────────────┤
│                                                                  │
│  GNU Make Jobserver Protocol:                                   │
│  - Token pipe: MAKEFLAGS="--jobserver-auth=<R>,<W>"            │
│  - Read token before starting job                               │
│  - Write token back when job completes                          │
│                                                                  │
│  Integration with harness:                                      │
│  1. Detect MAKEFLAGS in environment                             │
│  2. Respect token count for coalesced command parallelism       │
│  3. Allow nested builds to share token pool                     │
│                                                                  │
│  Custom harness jobserver:                                      │
│  - Unix socket: /tmp/harness-jobs.sock                         │
│  - Semaphore-based token distribution                           │
│  - Supports priority borrowing                                  │
│                                                                  │
└─────────────────────────────────────────────────────────────────┘
```

---

## Part 3: Filesystem & I/O Optimizations

### 3.1 Enhanced FUSE Layer

**Current Features:**
- Copy-on-write isolation per agent
- Write serialization
- IPC control (commit/discard)

**Proposed Enhancements:**

```rust
// Add to FUSE server

/// 3.1.1: Read-ahead prediction
struct ReadAheadPredictor {
    /// Historical access patterns per command
    patterns: HashMap<String, Vec<AccessPattern>>,
}

impl ReadAheadPredictor {
    fn predict(&self, cmd: &str, file: &Path) -> Vec<PathBuf> {
        // If file X is read, files Y and Z are usually read next
        // Pre-fetch them into page cache
    }
}

/// 3.1.2: Write coalescing
struct WriteCoalescer {
    pending: HashMap<PathBuf, Vec<WriteOp>>,
    flush_interval: Duration,
}

impl WriteCoalescer {
    fn write(&mut self, path: PathBuf, offset: u64, data: Vec<u8>) {
        // Buffer small writes, flush periodically or on fsync
        // Reduces syscall overhead for write-heavy workloads
    }
}

/// 3.1.3: Negative lookup cache
struct NegativeCache {
    /// Files confirmed to not exist (avoid repeated stat() failures)
    nonexistent: HashSet<PathBuf>,
    ttl: Duration,
}

/// 3.1.4: Directory entry cache
struct DentryCache {
    /// Cached readdir results
    entries: HashMap<PathBuf, (Vec<DirEntry>, Instant)>,
}
```

### 3.2 Optimized Stat Cache Shim

**Current:** LD_PRELOAD with simple hash map

**Enhancement:** More sophisticated caching

```c
// Enhanced statcache_shim.c

// 3.2.1: Batch invalidation
// Instead of individual entry invalidation, use generation numbers
typedef struct {
    uint64_t generation;
    struct stat st;
    char path[PATH_MAX];
} CacheEntry;

static atomic_uint64_t current_generation = 0;

void invalidate_subtree(const char* prefix) {
    // Increment generation — all entries with old generation are stale
    atomic_fetch_add(&current_generation, 1);
}

// 3.2.2: Bloom filter for negative cache
#include "bloom.h"
static struct bloom negative_bloom;

int stat_wrapper(const char* path, struct stat* st) {
    // Fast path: check bloom filter for known non-existent
    if (bloom_check(&negative_bloom, path)) {
        errno = ENOENT;
        return -1;
    }
    // ... rest of caching logic
}

// 3.2.3: Per-directory caching
// Cache entire readdir() results, not individual entries
```

### 3.3 I/O Scheduler Integration

```
┌─────────────────────────────────────────────────────────────────┐
│                   I/O Priority Classes                           │
├─────────────────────────────────────────────────────────────────┤
│                                                                  │
│  Class 1: REALTIME (ionice -c 1)                                │
│  └── User-interactive commands (TTY detected)                   │
│                                                                  │
│  Class 2: BEST-EFFORT (ionice -c 2)                             │
│  ├── Level 0-3: Agent commands with cache miss                  │
│  └── Level 4-7: Background refresh, speculative execution       │
│                                                                  │
│  Class 3: IDLE (ionice -c 3)                                    │
│  └── Cache garbage collection, background sync                  │
│                                                                  │
└─────────────────────────────────────────────────────────────────┘
```

**Implementation:**

```bash
harness::set_io_priority() {
    local class="$1"
    local level="${2:-4}"
    
    if command -v ionice &>/dev/null; then
        ionice -c "$class" -n "$level" -p $$
    fi
}

# In coalesce strategy, before execution:
if [[ -t 1 ]]; then
    harness::set_io_priority 2 0  # Best-effort, high priority
else
    harness::set_io_priority 2 4  # Best-effort, normal priority
fi
```

---

## Part 4: Agent Coordination Enhancements

### 4.1 Intent Broadcasting

**Problem:** Agents don't know what other agents are doing

**Solution:** Lightweight pub/sub for intent signaling

```
┌─────────────────────────────────────────────────────────────────┐
│                    Intent Broadcasting                           │
├─────────────────────────────────────────────────────────────────┤
│                                                                  │
│  Agent A broadcasts: INTENT {                                   │
│    agent: "claude",                                             │
│    action: "write",                                             │
│    files: ["src/main.py", "tests/test_main.py"],               │
│    estimated_duration: 5000ms                                   │
│  }                                                              │
│                                                                  │
│  Harness reactions:                                             │
│  ├── Pause cache writes for affected files                      │
│  ├── Queue conflicting commands from other agents               │
│  ├── Pre-invalidate related caches                              │
│  └── Notify Agent B of potential conflict                       │
│                                                                  │
│  Implementation: Unix socket multicast or shared memory ring    │
│                                                                  │
└─────────────────────────────────────────────────────────────────┘
```

### 4.2 Deadlock Detection & Prevention

```rust
// Add to harness daemon

struct LockGraph {
    /// Maps agent → set of locks held
    holdings: HashMap<String, HashSet<LockId>>,
    /// Maps agent → set of locks waiting for
    waiters: HashMap<String, HashSet<LockId>>,
}

impl LockGraph {
    fn detect_deadlock(&self) -> Option<Vec<String>> {
        // Cycle detection in wait-for graph
        // Returns list of agents in deadlock cycle
    }
    
    fn prevent_deadlock(&self, agent: &str, lock: LockId) -> bool {
        // Before granting lock, check if it would create cycle
        // If so, deny or suggest alternative ordering
    }
}
```

### 4.3 Fair Scheduling

```
┌─────────────────────────────────────────────────────────────────┐
│                 Fair Share Scheduling                            │
├─────────────────────────────────────────────────────────────────┤
│                                                                  │
│  Each agent gets a "share" of execution time:                   │
│                                                                  │
│  Agent    │ Share │ Used (last 60s) │ Priority Boost           │
│  ─────────┼───────┼─────────────────┼─────────────────────────  │
│  claude   │ 33%   │ 45%             │ -12 (over quota)         │
│  cursor   │ 33%   │ 20%             │ +13 (under quota)        │
│  copilot  │ 33%   │ 35%             │ -2  (near quota)         │
│                                                                  │
│  Priority boost affects queue ordering when contention occurs   │
│                                                                  │
└─────────────────────────────────────────────────────────────────┘
```

---

## Part 5: Polishing & Quality of Life

### 5.1 Observability Improvements

```bash
# harness metrics — Machine-readable metrics endpoint

harness::metrics() {
    cat <<METRICS
# HELP harness_cache_hits_total Total cache hits
# TYPE harness_cache_hits_total counter
harness_cache_hits_total{agent="claude"} $(get_hits claude)
harness_cache_hits_total{agent="cursor"} $(get_hits cursor)

# HELP harness_cache_hit_rate Cache hit rate
# TYPE harness_cache_hit_rate gauge
harness_cache_hit_rate $(calculate_hit_rate)

# HELP harness_command_duration_seconds Command execution time
# TYPE harness_command_duration_seconds histogram
harness_command_duration_seconds_bucket{le="0.1"} $(count_under 100)
harness_command_duration_seconds_bucket{le="1"} $(count_under 1000)
harness_command_duration_seconds_bucket{le="10"} $(count_under 10000)
METRICS
}
```

### 5.2 Interactive Dashboard

```
┌──────────────────────────────────────────────────────────────────────┐
│  harness dashboard                                    [q]uit [r]efresh │
├──────────────────────────────────────────────────────────────────────┤
│  AGENTS ACTIVE                                                        │
│  ┌────────┬────────┬───────────┬────────────┬─────────────────────┐  │
│  │ Agent  │ PID    │ Commands  │ Cache Hits │ Current             │  │
│  ├────────┼────────┼───────────┼────────────┼─────────────────────┤  │
│  │ claude │ 12345  │ 47        │ 89%        │ ruff check .        │  │
│  │ cursor │ 12346  │ 23        │ 76%        │ (idle)              │  │
│  └────────┴────────┴───────────┴────────────┴─────────────────────┘  │
│                                                                       │
│  LOCK CONTENTION                                                      │
│  ┌─────────────────────────────────────────────────────────────────┐ │
│  │ git.lock: claude (held 2.3s), cursor (waiting 1.1s)            │ │
│  └─────────────────────────────────────────────────────────────────┘ │
│                                                                       │
│  CACHE ACTIVITY (last 60s)                                           │
│  hits ▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓░░░░░░ 72%                                  │
│  exec ░░░░░░░░░░░░░░░░░░▓▓▓▓▓▓ 28%                                  │
│                                                                       │
│  FUSE MOUNTS                                                         │
│  /home/user/project → /mnt/project [CoW: 3 overlays, 12MB]          │
└──────────────────────────────────────────────────────────────────────┘
```

### 5.3 Self-Tuning

```bash
# Auto-adjust parameters based on observed behavior

harness::autotune() {
    # Analyze recent history
    local avg_hit_rate=$(harness::get_avg_hit_rate)
    local avg_exec_time=$(harness::get_avg_exec_time)
    local contention_rate=$(harness::get_contention_rate)
    
    # Adjust TTLs
    if (( $(echo "$avg_hit_rate < 0.5" | bc -l) )); then
        # Low hit rate → increase TTL
        harness::suggest "Consider increasing TTL: current hit rate is ${avg_hit_rate}"
    fi
    
    # Adjust concurrency
    if (( $(echo "$contention_rate > 0.3" | bc -l) )); then
        # High contention → recommend queue strategy
        harness::suggest "High lock contention detected. Consider 'queue' strategy for hot commands"
    fi
    
    # Adjust cache size
    local cache_pressure=$(harness::get_cache_pressure)
    if (( $(echo "$cache_pressure > 0.9" | bc -l) )); then
        harness::suggest "Cache near capacity. Consider increasing HARNESS_CACHE_SIZE"
    fi
}
```

### 5.4 Graceful Degradation

```bash
# When system is under stress, degrade gracefully

harness::check_system_pressure() {
    local load=$(cat /proc/loadavg | awk '{print $1}')
    local cpus=$(nproc)
    local pressure=$(echo "$load / $cpus" | bc -l)
    
    if (( $(echo "$pressure > 2.0" | bc -l) )); then
        # System overloaded
        export HARNESS_MODE="minimal"
        harness::log WARN "System under pressure, switching to minimal mode"
        # Disable: speculative execution, background refresh, read-ahead
    elif (( $(echo "$pressure > 1.5" | bc -l) )); then
        # System loaded
        export HARNESS_MODE="reduced"
        # Disable: speculative execution
    else
        export HARNESS_MODE="full"
    fi
}
```

---

## Implementation Roadmap

### Phase 1: Foundation (2-3 days) — ✅ COMPLETE
- [x] Lock timeout to prevent infinite waits
- [x] Stale-while-revalidate pattern
- [x] Enhanced metrics/logging (Prometheus-compatible)
- [x] Cache compression for large outputs
- [ ] Content-addressable storage implementation (advanced, deferred)

### Phase 2: Intelligence (3-4 days) — ✅ COMPLETE
- [x] Priority queue for command scheduling
  - 5 priority levels: critical, high, normal, low, background
  - Aging prevents starvation (waiting boosts priority)
  - Fair scheduling penalizes agents using excessive resources
- [x] Semantic coalescing rules
  - Path normalization for linters ("." → project root)
  - Cache sharing for semantically equivalent commands
- [ ] Input dependency tracking (FUSE-based) — deferred to Phase 3

### Phase 3: Performance (2-3 days) — ✅ COMPLETE
- [x] Hierarchical cache (L1 memory + L2 disk)
  - L1: /dev/shm (tmpfs), 100MB max, 60s TTL, ultra-fast
  - L2: var/cache (disk), unlimited, longer TTL, compressed
  - Automatic promotion from L2 to L1 on access
  - LRU eviction when L1 is full
- [x] I/O scheduler integration
  - ionice-based disk I/O priority
  - Priority mapping: critical→realtime, high→best-effort:0, normal→best-effort:4, low→best-effort:7, background→idle
  - Automatic I/O priority based on command type
- [x] Negative stat cache
  - Tracks files that don't exist to avoid repeated lookups
  - Short TTL (5s) for freshness
- [ ] Enhanced stat cache shim — existing implementation sufficient
- [ ] FUSE-based input dependency tracking — deferred

### Phase 4: Coordination (2-3 days) — ✅ COMPLETE
- [x] Intent broadcasting
  - Agents signal planned file operations before execution
  - Conflict detection for write-write and read-write conflicts
  - Automatic intent expiration and cleanup
- [x] Deadlock detection
  - Wait-for graph construction from lock records
  - Cycle detection using DFS
  - Automatic resolution by aborting youngest waiter
- [x] Fair share scheduling
  - Configurable share allocation per agent (default: 25% each)
  - Usage tracking with 50% decay for smoothing
  - Priority adjustment based on quota vs actual usage
  - Integration with priority queue effective priority calculation

### Phase 5: Polish (1-2 days) — ✅ COMPLETE
- [x] Interactive dashboard
  - Real-time terminal UI with cache stats, fair share, intents, queue status
  - Auto-refresh every 2 seconds (configurable via HARNESS_DASHBOARD_REFRESH)
  - Keyboard controls: q=quit, r=refresh
- [x] Self-tuning system
  - Analyzes metrics to detect issues (low hit rate, L1 underused, contention, timeouts)
  - Generates actionable recommendations with color-coded severity
  - Auto-apply safe fixes (L1 size, lock timeout)
  - Benchmark command for measuring L1 vs L2 performance
  - Rule generation suggestions based on observed patterns
- [x] Documentation (inline help, CLI commands)

---

## Appendix A: Research Sources

1. **Build Systems**
   - Bazel Remote Caching: https://bazel.build/remote/caching
   - Buck2 Architecture: https://buck2.build
   - sccache: https://github.com/mozilla/sccache
   - ccache direct mode: https://ccache.dev/manual/latest.html

2. **Memoization Tools**
   - bkt: https://github.com/dimo414/bkt
   - bash-cache: https://github.com/dimo414/bash-cache

3. **Incremental Computation**
   - Turbopack: https://nextjs.org/blog/turbopack-incremental-computation
   - Pants speculation: https://www.pantsbuild.org/blog/2021/02/01/fast-incremental-builds-speculation-cancellation
   - Adapton: https://arxiv.org/abs/1503.07792

4. **Cache Coherence**
   - MESI Protocol: https://en.wikipedia.org/wiki/MESI_protocol
   - Directory-based coherence: Patterson & Hennessy, Computer Architecture

5. **Process Scheduling**
   - GNU Make jobserver: https://www.gnu.org/software/make/manual/html_node/Parallel.html
   - Linux ionice: https://man7.org/linux/man-pages/man1/ionice.1.html

---

## Appendix B: Quick Wins (Can implement immediately)

✅ **COMPLETED** — All quick wins have been implemented as of 2026-02-07

1. ✅ **Lock timeout** (~20 lines) — DONE
   - Added `flock --timeout=30` via `HARNESS_LOCK_TIMEOUT` env var
   - Logs warning and falls back to uncached execution on timeout
   - Tracked via `lock_timeouts` metric

2. ✅ **Background refresh flag (stale-while-revalidate)** (~80 lines) — DONE
   - Added `stale=<duration>` to rules.conf options
   - Spawns background process for refresh while serving stale cache
   - Non-blocking lock prevents refresh pile-up
   - Tracked via `cache_stale_hits` and `refresh_triggered` metrics

3. ✅ **Prometheus metrics endpoint** (~100 lines) — DONE
   - `harness metrics` command (Prometheus format default)
   - `harness metrics json` for JSON output
   - Tracks: cache_hits, cache_misses, cache_stale_hits, lock_timeouts,
     lock_waits, commands_executed, refresh_triggered, cache_hit_ratio

4. ✅ **Cache compression** (~30 lines) — DONE
   - Automatic zstd compression for outputs > 10KB
   - Configurable via `HARNESS_COMPRESS_THRESHOLD` env var
   - Transparent decompression on cache read
   - Significant space savings for verbose command outputs

5. ✅ **Enhanced cache stats** — DONE
   - Status now shows: entries, fresh, compressed, hits, misses, hit_rate%
   - Supports both compressed and uncompressed cache entries

**Configuration Reference:**

Environment variables:
- `HARNESS_LOCK_TIMEOUT=30`        — Max seconds to wait for lock
- `HARNESS_COMPRESS_THRESHOLD=10240` — Compress outputs larger than N bytes
- `HARNESS_STALE_THRESHOLD=0`      — Global default stale-while-revalidate (0=disabled)

Rule options:
- `stale=<seconds>`  — Per-rule stale-while-revalidate threshold

Example rule with all optimizations:
```
ruff:check  coalesce  ttl=15  stale=5  cache_key=git  debounce_ms=150
```
This means: cache for 15s, but after 5s serve stale + refresh in background.
