# agent-harness

**OS-level command proxy that deduplicates, caches, and rate-limits tool invocations from concurrent AI coding agents.**

## Problem

You're running 5 instances of Claude Code, 3 of Augment CLI, and a Cursor session — all on the same codebase. Each agent independently triggers `ruff check .`, `eslint`, `tsc`, `npm run lint`, etc. The result:

- **8+ identical lint processes** compete for CPU, disk I/O, and file descriptors
- **Redundant work** — same files, same arguments, same output, wasted 8×
- **Resource contention** — file descriptor limits hit, processes thrash
- **No coordination possible** — each agent runs in its own isolated bash process

## Solution

`agent-harness` is a single bash script that acts as a transparent proxy via the [busybox symlink pattern](https://en.wikipedia.org/wiki/BusyBox). It sits first in `PATH`, detects whether the caller is an AI agent (by walking the process tree), and applies configurable strategies:

```
┌────────────────────────────────────────────────────────────────┐
│                        AI Agents                               │
│   Claude ×5        Augment ×3       Cursor ×2       Aider     │
│   (isolated bash)  (isolated bash)  (isolated bash)  ...      │
│        │                │                │                     │
│        ▼                ▼                ▼                     │
│   ┌────────────────────────────────────────────────────┐      │
│   │              agent-harness (PATH proxy)             │      │
│   │                                                    │      │
│   │  $0 = "ruff" ──→ is caller an agent? ──→ NO ──→ exec real │
│   │                         │ YES                      │      │
│   │                         ▼                          │      │
│   │                  look up rule ──→ dispatch          │      │
│   │                                                    │      │
│   │  ┌───────────┐ ┌──────────┐ ┌───────────────────┐ │      │
│   │  │ coalesce  │ │  queue   │ │    passthrough     │ │      │
│   │  │ (flock +  │ │ (N-slot  │ │  (direct exec,    │ │      │
│   │  │  cache +  │ │  pool)   │ │   zero overhead)   │ │      │
│   │  │  dedup)   │ │          │ │                    │ │      │
│   │  └───────────┘ └──────────┘ └───────────────────┘ │      │
│   └────────────────────────────────────────────────────┘      │
│        │                                                       │
│        ▼                                                       │
│   Real binary (ruff, eslint, tsc, etc.)                       │
└────────────────────────────────────────────────────────────────┘
```

**Human users get zero overhead** — the proxy detects non-agent callers and immediately `exec`s the real binary.

## Quick Start

```bash
# Clone or download
git clone <repo> ~/agent-harness
cd ~/agent-harness

# Install (user-level, modifies ~/.bashrc)
chmod +x install.sh bin/harness
./install.sh --user

# Activate in current shell
export PATH="$HOME/.local/share/agent-harness/proxy:$PATH"

# Verify
harness status
harness test
```

## How It Works

### 1. Transparent Interception (Busybox Pattern)

The installer creates symlinks in a `proxy/` directory for each command in `rules.conf`:

```
proxy/
├── ruff      → ../bin/harness
├── eslint    → ../bin/harness
├── tsc       → ../bin/harness
├── npm       → ../bin/harness
├── .ruff.real       # cached: /usr/local/bin/ruff
├── .eslint.real     # cached: /usr/bin/eslint
└── ...
```

When any program calls `ruff`, PATH resolves to our symlink. The harness reads `basename $0` to know which command was invoked.

### 2. Agent Detection (Process Tree Walk)

The harness walks `/proc/$PPID/comm` up the process tree, matching against patterns in `agents.conf`:

```
ruff (PID 5432)
  └─ bash (PID 5430)
      └─ node (PID 5100)
          └─ claude (PID 5001)  ← MATCH! This is an agent.
```

If no agent ancestor is found → **immediate `exec`** to the real binary (human user).

### 3. Rule Matching

Each command can have strategy rules based on the subcommand:

```conf
ruff:check    coalesce  ttl=15  cache_key=git  nocache_args=--fix
ruff:format   queue     max_concurrent=1
```

Exact matches (`ruff:check`) take priority over wildcards (`ruff:*` or `ruff`).

### 4. Strategy Execution

| Strategy | Behavior | Best For |
|---|---|---|
| **coalesce** | `flock` → check cache → execute if stale → cache result. Concurrent arrivals block and receive the cached output. | Read-only checks: `ruff check`, `eslint`, `tsc`, `mypy` |
| **queue** | N-slot concurrency limiter. Each caller executes independently but parallelism is bounded. | Mutating commands: `ruff format`, `black`, `prettier --write` |
| **debounce** | Sleep before executing (no caching). Absorbs rapid-fire calls. | Rare use cases where you want delay but not dedup. |
| **passthrough** | Direct `exec`, zero overhead. | Tests, interactive commands, anything you don't want proxied. |

## Configuration

### `etc/rules.conf`

```conf
# FORMAT: COMMAND[:SUBCOMMAND]  STRATEGY  [key=value ...]

# Python
ruff:check      coalesce  ttl=15  cache_key=git  debounce_ms=150  nocache_args=--fix
ruff:format     queue     max_concurrent=1
mypy            coalesce  ttl=30  cache_key=git

# JavaScript
eslint          coalesce  ttl=15  cache_key=git  nocache_args=--fix
prettier:--check coalesce ttl=15
tsc             coalesce  ttl=20  cache_key=git

# Don't proxy tests
pytest          passthrough
```

#### Options Reference

| Option | Default | Description |
|---|---|---|
| `ttl=N` | `10` | Cache lifetime in seconds (coalesce only) |
| `error_ttl=N` | `=ttl` | Shorter TTL for failed (non-zero exit) results |
| `debounce_ms=N` | `0` | Milliseconds to wait before executing |
| `max_concurrent=N` | `1` | Max parallel executions (queue only) |
| `cache_key=MODE` | `time` | Cache key strategy: `time`, `git`, or `args` |
| `nocache_args=X,Y` | `""` | Flags that force fallback from coalesce → queue |

#### Cache Key Modes

- **`time`** — `hash(cmd + args + CWD)`. Expires after `ttl` seconds. Simple and fast.
- **`git`** — `hash(cmd + args + CWD + git status)`. Cache stays valid as long as working tree is unchanged. Ideal for lint tools. Falls back to `time` mode outside git repos.
- **`args`** — `hash(cmd + args)`. CWD-independent. Use when the command operates on absolute paths.

### `etc/agents.conf`

```conf
# Substring matched against process names in parent tree
claude
cursor
augment
aider
copilot
windsurf
cody
codex
```

## CLI

```bash
harness status     # Show proxies, rules, cache stats, recent log
harness sync       # Recreate symlinks from rules.conf (run after editing)
harness flush      # Clear all cached results and lock files
harness log        # Tail the harness log (live)
harness test       # Run diagnostics: agent detection + binary resolution
harness readcache  # File read cache subsystem (see below)
harness fuse       # FUSE overlay filesystem (see below)
```

## Advanced Features

### Tier 2: Dual-Git Audit Layer

The audit layer maintains a shadow git repository that records all file operations performed through the harness, providing complete accountability and rollback capability.

```bash
# Initialize audit layer for a directory
bin/audit init /path/to/project

# Show audit status
bin/audit status

# Show detailed change log
bin/audit log

# Rollback all changes made by a specific agent
bin/audit rollback claude

# Commit accumulated changes with attribution
bin/audit commit
```

The audit layer uses git hooks to intercept file writes and maintains attribution metadata for each change.

### Tier 2.5: File Read Cache Subsystem

The readcache system provides a three-layer caching architecture for file reads:

1. **LD_PRELOAD stat cache** — Intercepts `stat()` calls, caches metadata
2. **RAM disk mirroring** — Hot files served from tmpfs
3. **Manifest preload** — Warm caches based on tool access patterns

```bash
# Show cache status
harness readcache status

# Compile the stat cache shim (requires gcc)
harness readcache compile

# Warm caches for a specific tool
harness readcache warm /path/to/project ruff

# Create a RAM disk mirror
harness readcache ramdisk init /path/to/project 512  # 512MB
harness readcache ramdisk watch /path/to/project /mnt/mirror

# Record tool access patterns
harness readcache record /path/to/project ruff check .
```

**Performance**: The stat cache shim achieves 27-47× speedup on metadata-heavy operations.

### Tier 3: FUSE Overlay Filesystem

The FUSE server provides kernel-level write serialization and optional copy-on-write isolation per agent.

```bash
# Mount FUSE overlay
harness fuse mount /home/user/project /mnt/harness/project

# Mount with copy-on-write per agent
harness fuse mount /src /mnt/src --cow --foreground

# Show mount status
harness fuse status

# List CoW overlays
harness fuse list /mnt/src

# Commit an agent's changes to the real filesystem
harness fuse commit /mnt/src claude

# Discard an agent's changes (rollback)
harness fuse discard /mnt/src claude

# Unmount
harness fuse unmount /mnt/harness/project
```

**Features**:
- **Write serialization**: Per-file locks prevent concurrent write conflicts
- **Copy-on-Write**: Each agent gets an isolated view of the filesystem
- **Agent attribution**: All operations logged with agent identification
- **IPC control**: Commit/discard/list via Unix socket

**Building** (requires Rust):
```bash
cd fuse && cargo build --release
```

## Examples

### Before: 5 agents, 5 redundant lint runs

```
[agent-1]  ruff check .   →  2.3s  (CPU spike, 400+ FDs)
[agent-2]  ruff check .   →  2.1s  (same files, same result)
[agent-3]  ruff check .   →  2.5s  (contention, slower)
[agent-4]  ruff check .   →  3.1s  (FD pressure)
[agent-5]  ruff check .   →  2.8s  (total wall time ≈ 3.1s, 5× CPU)
```

### After: 1 execution, 4 cache hits

```
[agent-1]  ruff check .   →  2.3s  (executes, caches result)
[agent-2]  ruff check .   →  0.01s (blocks on lock, gets cached result)
[agent-3]  ruff check .   →  0.01s (cache hit)
[agent-4]  ruff check .   →  0.01s (cache hit)
[agent-5]  ruff check .   →  0.01s (cache hit)
```

**Result: 1× CPU, ~5× faster for agents 2-5, no FD pressure.**

### Safety: `--fix` flag detected

```
[agent-1]  ruff check --fix .   →  nocache_args match!
                                    Falls back to queue (max_concurrent=1)
                                    Executes normally, no caching.
```

## Installation Options

```bash
# User-level (recommended for personal machines)
./install.sh --user
# Installs to: ~/.local/share/agent-harness
# Modifies:    ~/.bashrc

# System-wide (for shared servers / CI)
sudo ./install.sh --system
# Installs to: /opt/agent-harness
# Modifies:    /etc/profile.d/agent-harness.sh

# Custom directory
./install.sh --dir /path/to/wherever
```

### Uninstall

```bash
./uninstall.sh
```

## Technical Details

### Locking

Uses `flock(1)` for POSIX advisory file locks. Locks are automatically released when the process exits (even on SIGKILL), preventing deadlocks.

### Atomic Cache Writes

Results are written to temporary files and atomically renamed (`mv`), ensuring concurrent readers never see partial output.

### File Descriptor Management

The harness uses FDs 200+ for lock files to avoid conflicting with the proxied command's own FD usage.

### Process Tree Walk

Walks `/proc/PID/comm` and `/proc/PID/cmdline` on Linux. Falls back to `ps -o comm=` on macOS. Stops after 32 hops (safety limit).

### Performance

- **Human users**: 0ms overhead (immediate `exec` after ~2ms agent detection)
- **Agent cache hit**: ~10ms (lock acquire + file read + output)
- **Agent cache miss**: `debounce_ms` + real execution time

## Limitations

- **stdin-based input**: Commands reading from stdin (pipes) are cached based on args only, not stdin content. If you pipe different input, you'll get stale results. Add such commands to `passthrough`.
- **macOS**: Requires `flock` (install via `brew install flock` or use the util-linux package). Process detection uses `ps` fallback (slightly slower).
- **Containers**: If agents run in separate containers, mount a shared volume at the harness var directory for cross-container lock/cache sharing.

## Contributing

The entire harness is ~400 lines of bash across two files (`bin/harness` + `lib/core.sh`). It's intentionally simple and hackable.

## License

MIT
