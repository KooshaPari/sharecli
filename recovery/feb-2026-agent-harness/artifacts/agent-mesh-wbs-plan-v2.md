# Agent Mesh Coordination Layer — Comprehensive Phased WBS Plan

**Version:** 2.0  
**Date:** February 2026  
**Prerequisite:** agent-harness Phases 1-10 (core.sh, bin/harness)  
**Sponsor Model:** Human as executive sponsor (absolute last resort for decisions)

---

## Executive Summary

This plan implements a heterogeneous coding agent mesh synthesizing research from April 2025–February 2026 including:
- Byzantine consensus protocols (CP-WBFT, DecentLLMs, Aegean, Six Sigma Agent)
- Production tool internals (Claude Code Agent Teams, Cursor 2.0, Devin 2.0, Augment Intent)
- Protocol standards (MCP, A2A, AGENTS.md)
- Collaboration patterns (tournament, pair programming, saga, code review)
- Shell→agent injection via tmux with race condition mitigation
- Blackboard architecture with autonomous volunteering
- Failure taxonomy and recovery patterns

**Total Phases:** 16 (Phases 11-26, continuing from agent-harness)  
**Estimated Implementation:** 14-18 weeks

---

# PART I: INFRASTRUCTURE FOUNDATION

---

## Phase 11: IPC Mechanisms & Filesystem Primitives

**Objective:** Establish low-level coordination primitives on tmpfs with proven reliability patterns.

### 11.1 tmpfs Setup & Management

| Task | Description | Deliverable |
|------|-------------|-------------|
| 11.1.1 | Mount tmpfs at `/tmp/agent-mesh` or `/dev/shm/agent-mesh` | Mount script |
| 11.1.2 | Size configuration: 256MB default, configurable | Config system |
| 11.1.3 | Mode 1777 (sticky bit) for multi-user safety | Permission model |
| 11.1.4 | Disk usage monitoring with alerts at 80%/90% thresholds | Monitor daemon |
| 11.1.5 | Automatic cleanup: completed tasks >60min, logs >30min | Cleanup policy |

### 11.2 Atomic Operations Library

| Task | Description | Deliverable |
|------|-------------|-------------|
| 11.2.1 | `atomic_write()`: write to temp file + `rename()` (POSIX atomic guarantee) | `lib/atomic.sh` |
| 11.2.2 | `atomic_mkdir()`: directory creation as lock (EEXIST = already held) | Lock primitive |
| 11.2.3 | `atomic_claim()`: mkdir + owner file + lease timestamp | Claim primitive |
| 11.2.4 | `flock()` wrapper for critical sections with timeout | `lib/flock.sh` |
| 11.2.5 | Lamport timestamps for causal ordering across agents | HLC integration |

### 11.3 Maildir Message Queue

```
$MESH_DIR/maildir/agent-{uuid}/
├── tmp/          # Writes in progress
├── new/          # Delivered, unread
├── cur/          # Being processed
└── processed/    # Archived (optional)
```

| Task | Description | Deliverable |
|------|-------------|-------------|
| 11.3.1 | Maildir write: create in tmp/, rename to new/ | Write protocol |
| 11.3.2 | Maildir read: move from new/ to cur/, process, delete or archive | Read protocol |
| 11.3.3 | Message envelope schema: id, sender, recipient, timestamp, type, ttl_seconds, payload | Envelope schema |
| 11.3.4 | Unique message IDs for idempotent processing | ID generator |
| 11.3.5 | TTL enforcement: auto-expire unprocessed messages | Expiry daemon |

### 11.4 Event Notification System

| Task | Description | Deliverable |
|------|-------------|-------------|
| 11.4.1 | Primary: `inotifywait -m -r -e create,modify,moved_to,delete` on coordination dirs | inotify wrapper |
| 11.4.2 | Latency target: 1-10ms event detection | Benchmark suite |
| 11.4.3 | Fallback (NFS, macOS): polling with `find -newer` at 1s intervals | Polling fallback |
| 11.4.4 | SIGUSR1 for urgent preemption: sender writes flag file, sends signal | Signal protocol |
| 11.4.5 | inotify watch limits: check `max_user_watches`, warn if <10000 | Limit checker |

### 11.5 Write-Ahead Logging (WAL)

| Task | Description | Deliverable |
|------|-------------|-------------|
| 11.5.1 | Append operation intent to `$MESH_DIR/wal/{seq}.log` before executing | WAL writer |
| 11.5.2 | Mark operation complete after success | Completion marker |
| 11.5.3 | On restart: replay incomplete operations or rollback | Recovery daemon |
| 11.5.4 | Idempotent operation design: content-addressed outputs, check-before-write | Design patterns |
| 11.5.5 | WAL compaction: merge completed entries periodically | Compaction job |

**Exit Criteria:** Atomic primitives pass stress tests (1000 concurrent operations); inotify latency <10ms; WAL recovery works after simulated crash.

---

## Phase 12: Agent Process Discovery & Registry

**Objective:** Detect, identify, and register heterogeneous coding agents on the local machine.

### 12.1 Process Scanner

| Task | Description | Deliverable |
|------|-------------|-------------|
| 12.1.1 | `/proc/[pid]/cmdline` scanner with agent-specific patterns | `lib/discovery.sh` |
| 12.1.2 | Claude Code: Node.js with `claude` in cmdline OR native binary at `~/.local/bin/claude` | Pattern |
| 12.1.3 | Aider: Python process with `aider` in arguments | Pattern |
| 12.1.4 | Cursor/Windsurf: Electron apps (detect via process tree, GPU/renderer children) | Pattern |
| 12.1.5 | Cline/Continue.dev: VS Code extensions at `~/.vscode/extensions/` | Extension detector |
| 12.1.6 | Extract working directory from `/proc/[pid]/cwd` | Project association |
| 12.1.7 | Read environment from `/proc/[pid]/environ` (API keys, config) | Env extraction |

### 12.2 Real-Time Process Monitoring

| Task | Description | Deliverable |
|------|-------------|-------------|
| 12.2.1 | Primary: `forkstat` (netlink proc connector) for execve() monitoring | forkstat integration |
| 12.2.2 | Alternative: eBPF `execsnoop` for lower overhead | eBPF option |
| 12.2.3 | Fallback: `/proc/*/cmdline` polling every 5 seconds | Polling scanner |
| 12.2.4 | New agent detection → trigger registration flow | Registration hook |
| 12.2.5 | Agent exit detection → trigger cleanup flow | Cleanup hook |

### 12.3 Agent Manifest System

```yaml
# $MESH_DIR/agents/agent-{uuid}.yaml
id: agent-{uuid}
type: claude-code | aider | cursor | cline | devin | custom
pid: 12345
started_at: "2026-02-10T12:00:00Z"
working_directory: /home/user/project
worktree: .mesh/worktrees/agent-{uuid}
branch: mesh/{uuid}/main

capabilities:
  languages: [typescript, python, go]
  frameworks: [react, fastapi, gin]
  specializations: [security, performance, testing]
  
operational_design_domain:  # ODD boundaries
  allowed_paths: [src/, tests/, docs/]
  forbidden_paths: [.env, secrets/, node_modules/]
  allowed_commands: [npm test, pytest, go test]
  forbidden_commands: [rm -rf, DROP TABLE]
  max_file_size_kb: 1000
  
priority: 100  # For leader election (higher = more capable)
status: idle | working | blocked | error
current_task: task-{id} | null
confidence_history: []  # Rolling accuracy for reputation
```

### 12.4 Capability Index

```
$MESH_DIR/capabilities/
├── by-language/
│   ├── typescript.list    # agent UUIDs
│   ├── python.list
│   └── go.list
├── by-framework/
│   ├── react.list
│   └── fastapi.list
├── by-specialization/
│   ├── security.list
│   ├── performance.list
│   └── testing.list
└── matrix.json            # Full capability matrix
```

### 12.5 Heartbeat & Liveness

| Task | Description | Deliverable |
|------|-------------|-------------|
| 12.5.1 | Touch-file heartbeat at `$MESH_DIR/heartbeats/agent-{uuid}` | Heartbeat protocol |
| 12.5.2 | Heartbeat interval: 5 seconds | Timing constant |
| 12.5.3 | Failure threshold: 15 seconds (3 missed heartbeats) | Detection threshold |
| 12.5.4 | Grace period: 30 seconds before full removal | Grace period |
| 12.5.5 | Stale agent cleanup: reclaim tasks, notify dependents, archive manifest | Cleanup daemon |
| 12.5.6 | Lease renewal for claimed tasks (separate from heartbeat) | Lease protocol |

**Exit Criteria:** Scanner correctly identifies all supported agent types; registration completes within 2 seconds; stale agents detected within 15 seconds.

---

## Phase 13: Shell→Agent Injection Layer

**Objective:** Reliably send commands to running agents via tmux with race condition mitigation.

### 13.1 tmux Session Management

| Task | Description | Deliverable |
|------|-------------|-------------|
| 13.1.1 | Session naming convention: `mesh-{agent-uuid}` | Naming standard |
| 13.1.2 | Session discovery: `tmux list-sessions -F '#{session_name}'` | Discovery |
| 13.1.3 | Pane targeting: `session:window.pane` format | Targeting |
| 13.1.4 | Session creation for new agents: `tmux new-session -d -s mesh-{uuid}` | Creation |
| 13.1.5 | Integration with Claude Squad's existing session patterns | Compatibility |

### 13.2 Command Injection with Race Condition Mitigation

**Critical:** Naive `send-keys "command" Enter` has 15% command loss.

| Task | Description | Deliverable |
|------|-------------|-------------|
| 13.2.1 | Reliable pattern: `send-keys -l "text"`, `sleep 1.5`, `send-keys Enter` | `lib/inject.sh` |
| 13.2.2 | Configurable delay (1.0-2.0s based on agent responsiveness) | Delay tuning |
| 13.2.3 | Multi-line command handling: escape newlines properly | Multiline support |
| 13.2.4 | Special character escaping: quotes, backticks, dollar signs | Escape utilities |
| 13.2.5 | Command batching: queue commands, inject with proper pacing | Batch injector |

```bash
# WRONG - 15% command loss
tmux send-keys -t mesh-abc:0.0 "Fix the auth bug" Enter

# CORRECT - reliable injection
inject_command() {
    local target="$1" command="$2"
    tmux send-keys -t "$target" -l "$command"
    sleep 1.5
    tmux send-keys -t "$target" Enter
}
```

### 13.3 Agent Readiness Detection

| Task | Description | Deliverable |
|------|-------------|-------------|
| 13.3.1 | Capture pane content: `tmux capture-pane -p -t target -S -10` | Capture utility |
| 13.3.2 | Readiness patterns by agent type: | Pattern registry |
| | - Aider: `aider>` or `───` separator | |
| | - Claude Code: `claude>` or `>` prompt | |
| | - Shell: `$` or `%` or `#` | |
| 13.3.3 | Busy detection patterns: spinner chars `⠋⠙⠹⠸⠼⠴⠦⠧⠇⠏`, "Thinking...", progress bars | Busy patterns |
| 13.3.4 | Wait-for-ready with timeout: | Ready waiter |

```bash
wait_for_ready() {
    local target="$1" pattern="$2" timeout="${3:-120}"
    local elapsed=0
    while [ "$elapsed" -lt "$timeout" ]; do
        if tmux capture-pane -p -t "$target" -S -5 | grep -qE "$pattern"; then
            return 0
        fi
        sleep 1
        elapsed=$((elapsed + 1))
    done
    return 1  # Timeout
}
```

### 13.4 Agent State Machine

| State | Description | Indicators |
|-------|-------------|------------|
| **WAITING** | Ready for input | Prompt visible, no spinner |
| **BUSY** | Processing | Spinner, "Thinking...", progress |
| **IDLE** | Between tasks | Prompt visible, no recent output |
| **ERROR** | Failed state | Error messages, stack traces |
| **BLOCKED** | Waiting on external | "Waiting for...", paused |

| Task | Description | Deliverable |
|------|-------------|-------------|
| 13.4.1 | State detection from pane content | State detector |
| 13.4.2 | State change events: emit to event log | State events |
| 13.4.3 | State persistence in agent manifest | Manifest updates |

### 13.5 Output Streaming & Capture

| Task | Description | Deliverable |
|------|-------------|-------------|
| 13.5.1 | Stream output: `tmux pipe-pane -t target "cat >> /tmp/agent-{uuid}.log"` | Output streaming |
| 13.5.2 | Output parsing: extract structured results from agent responses | Output parser |
| 13.5.3 | Completion detection: agent signals task done | Completion detector |
| 13.5.4 | Error extraction: parse error messages and stack traces | Error parser |

**Exit Criteria:** Command injection reliability >99%; readiness detection works for all agent types; state machine correctly tracks agent states.

---

## Phase 14: Context Injection & Configuration Coordination

**Objective:** Inject mesh coordination instructions into agents via their native configuration mechanisms.

### 14.1 Configuration File Injection (Tier 1 - Zero Modification)

| Task | Description | Deliverable |
|------|-------------|-------------|
| 14.1.1 | Create `AGENT.md` in project root as single source of truth | AGENT.md template |
| 14.1.2 | Symlink to tool-specific files: `.cursorrules`, `.clinerules`, `.windsurfrules` | Symlink manager |
| 14.1.3 | Claude Code: `CLAUDE.md` in project root | Claude adapter |
| 14.1.4 | Aider: `--read` flag for additional context files | Aider adapter |
| 14.1.5 | Dynamic content injection: current mesh state, active tasks, coordination rules | Content generator |

### 14.2 AGENT.md Schema

```markdown
# Project: {project_name}

## Mesh Coordination

You are part of an agent mesh. Follow these coordination rules:

### Your Identity
- Agent ID: {uuid}
- Capabilities: {capabilities}
- Current Task: {task_id}
- Worktree: {worktree_path}

### Coordination Rules
1. Only modify files in your assigned directories: {allowed_paths}
2. Before modifying shared files, check $MESH_DIR/locks/
3. Write completion status to $MESH_DIR/tasks/done/
4. If blocked, write to $MESH_DIR/escalation/

### Communication
- Your inbox: $MESH_DIR/inbox/{uuid}/
- Broadcast: $MESH_DIR/broadcast/
- Check inbox between major operations

### Current Mesh State
- Active agents: {agent_count}
- Your dependencies: {dependencies}
- Agents depending on you: {dependents}
```

### 14.3 Aider Watch-Files Integration

| Task | Description | Deliverable |
|------|-------------|-------------|
| 14.3.1 | Create `.aider.watch` file with mesh coordination markers | Watch file |
| 14.3.2 | `# AI:` comment markers as inline instructions | Marker format |
| 14.3.3 | Dynamic updates to watch file based on mesh state | Dynamic updater |

### 14.4 VS Code Extension Communication

| Task | Description | Deliverable |
|------|-------------|-------------|
| 14.4.1 | Detect VS Code IPC socket: `VSCODE_IPC_HOOK_CLI` env var | Socket detector |
| 14.4.2 | Unix domain socket at `/run/user/<UID>/vscode-ipc-<UUID>.sock` | Socket locator |
| 14.4.3 | Cline standalone mode: `cline-core.ts` with gRPC interface | Cline adapter |
| 14.4.4 | Continue.dev headless CLI mode (`cn` command) | Continue adapter |
| 14.4.5 | Cursor remote debugging: `--remote-debugging-port=9222` for CDP | Cursor CDP adapter |

**Exit Criteria:** AGENT.md correctly injected for all agent types; configuration changes reflected within 5 seconds; no conflicts between symlinked files.

---

## Phase 15: Git Worktree Isolation & Branch Coordination

**Objective:** Provide each agent with isolated filesystem via git worktrees.

### 15.1 Worktree Lifecycle

| Task | Description | Deliverable |
|------|-------------|-------------|
| 15.1.1 | Create on registration: `git worktree add .mesh/worktrees/agent-{uuid} -b mesh/{uuid}/main` | `lib/worktree.sh` |
| 15.1.2 | Branch naming: `mesh/{agent-uuid}/{task-id}` | Branch schema |
| 15.1.3 | Worktree per task (optional): isolate task branches | Task worktrees |
| 15.1.4 | Cleanup on agent departure (after 30s grace) | Cleanup hooks |
| 15.1.5 | Orphan worktree detection and cleanup | Orphan detector |
| 15.1.6 | Worktree health: detect corruption, repair or recreate | Health monitor |

### 15.2 Branch Coordination

| Task | Description | Deliverable |
|------|-------------|-------------|
| 15.2.1 | Branch registry in `$MESH_DIR/branches/` | Branch registry |
| 15.2.2 | Atomic branch creation with collision retry | Collision avoidance |
| 15.2.3 | Branch status tracking: active, ready-to-merge, merged, abandoned | Status tracking |
| 15.2.4 | Pre-merge conflict detection: `git merge-tree` analysis | Conflict detector |

### 15.3 Merge Queue

```
$MESH_DIR/merge-queue/
├── pending/
│   └── merge-{id}.yaml       # Branch, priority, dependencies
├── in_progress/
│   └── merge-{id}.yaml       # Currently merging
├── conflicts/
│   └── merge-{id}.yaml       # Conflict details
└── completed/
    └── merge-{id}.yaml       # Merge result
```

| Task | Description | Deliverable |
|------|-------------|-------------|
| 15.3.1 | Submit branch to merge queue | Queue submission |
| 15.3.2 | Priority ordering: P1-P4, then FIFO | Priority queue |
| 15.3.3 | Conflict resolution: auto-merge if trivial, escalate if semantic | Auto-merge |
| 15.3.4 | Post-merge test gate: run tests before finalizing | Test gate |
| 15.3.5 | Rollback on test failure | Rollback mechanism |

### 15.4 Tool-Specific Integration

| Task | Description | Deliverable |
|------|-------------|-------------|
| 15.4.1 | Claude Squad compatibility: respect `cs-*` branch patterns | Claude Squad adapter |
| 15.4.2 | Cursor Background Agents: detect cloud VM worktrees | Cursor adapter |
| 15.4.3 | Aider: single-worktree mode (Aider manages own commits) | Aider adapter |
| 15.4.4 | Devin: detect Devin's internal branch management | Devin adapter |

**Exit Criteria:** Agents operate in isolated worktrees; merge queue handles 10+ concurrent branches; conflict detection catches semantic conflicts.

---

## Phase 16: Sandboxing & Boundary-Based Autonomy

**Objective:** Replace per-action approval with boundary-based autonomy using OS-level sandboxing.

### 16.1 Sandbox Runtime Integration

| Task | Description | Deliverable |
|------|-------------|-------------|
| 16.1.1 | Linux: bubblewrap (`bwrap`) integration | `lib/sandbox-linux.sh` |
| 16.1.2 | macOS: seatbelt (`sandbox-exec`) integration | `lib/sandbox-macos.sh` |
| 16.1.3 | Filesystem policy: agent reads/writes only within worktree + approved paths | FS policy |
| 16.1.4 | Network policy: allowlist of approved domains | Network policy |
| 16.1.5 | Process policy: prevent unrestricted subprocess spawning | Process policy |
| 16.1.6 | Target: 84% reduction in permission prompts (Anthropic benchmark) | Metrics |

### 16.2 Autonomy Tiers

| Tier | Scope | Approval | Examples |
|------|-------|----------|----------|
| **1** | Read-only | Auto | Code analysis, search, read docs |
| **2** | Worktree writes | Auto | Edit files in own worktree |
| **3** | Git commits | Auto | Commit to own branch |
| **4** | Shared resources | Consensus/Test | Merge, external API (safe) |
| **5** | Production/Irreversible | Human | Deploy, credentials, delete data |

### 16.3 Operational Design Domain (ODD)

| Task | Description | Deliverable |
|------|-------------|-------------|
| 16.3.1 | Per-agent ODD definition in manifest | ODD schema |
| 16.3.2 | Path allowlist/denylist | Path rules |
| 16.3.3 | Command allowlist/denylist | Command rules |
| 16.3.4 | File type restrictions | Type rules |
| 16.3.5 | Size limits (max file size, total changes) | Size limits |
| 16.3.6 | ODD violation detection and blocking | Violation detector |

### 16.4 Test-Gated Autonomy

| Task | Description | Deliverable |
|------|-------------|-------------|
| 16.4.1 | Detect project test suite (pytest, jest, go test, cargo test) | Suite detector |
| 16.4.2 | Run tests after agent modifications | Test runner |
| 16.4.3 | Auto-approve Tier 4 actions if tests pass | Test gate logic |
| 16.4.4 | Escalate if tests fail after N retries | Failure escalation |
| 16.4.5 | Coverage delta tracking: flag if coverage drops >5% | Coverage monitor |

### 16.5 Hooks System

| Task | Description | Deliverable |
|------|-------------|-------------|
| 16.5.1 | PreToolUse hooks: check boundary before action | Pre-hooks |
| 16.5.2 | PostToolUse hooks: validate results, update state | Post-hooks |
| 16.5.3 | Stop hooks: emergency halt conditions (circuit breaker) | Stop hooks |
| 16.5.4 | Hook configuration: YAML-based policy definitions | Policy YAML |
| 16.5.5 | Dynamic policy reload without restart | Hot reload |

**Exit Criteria:** Permission prompts reduced by ≥80%; sandbox prevents all ODD violations; test gate correctly blocks failing changes.

---

# PART II: RESOURCE MANAGEMENT & CONSTRAINTS

---

## Phase 17: Resource Management & Constraints

**Objective:** Manage computational resources across multiple concurrent agents.

### 17.1 Memory Management

| Task | Description | Deliverable |
|------|-------------|-------------|
| 17.1.1 | Per-agent RAM tracking via `/proc/[pid]/status` | Memory monitor |
| 17.1.2 | Typical footprints: Claude Code 2-4GB, Cursor 4-8GB (Electron), Aider 1-2GB | Baseline metrics |
| 17.1.3 | Total mesh limit: configurable (default 80% of system RAM) | Limit config |
| 17.1.4 | Memory pressure alerts: warn at 70%, critical at 85% | Alerting |
| 17.1.5 | Graceful degradation: pause lowest-priority agent on pressure | Degradation logic |
| 17.1.6 | OOM prevention: preemptive task migration | OOM prevention |

### 17.2 Process Limits

| Task | Description | Deliverable |
|------|-------------|-------------|
| 17.2.1 | Practical agent limit: 5-10 on typical dev machine (32GB RAM, 8 cores) | Sizing guide |
| 17.2.2 | File descriptor limits: monitor per-agent FD usage | FD monitor |
| 17.2.3 | Process count limits: detect runaway subprocess spawning | Process monitor |
| 17.2.4 | CPU quota (optional): cgroups-based CPU limiting | CPU cgroups |

### 17.3 inotify Watch Management

| Task | Description | Deliverable |
|------|-------------|-------------|
| 17.3.1 | Check `max_user_watches`: default 8192, often 524288 on modern systems | Watch checker |
| 17.3.2 | Watch budget allocation: ~1KB kernel memory per watch | Budget tracking |
| 17.3.3 | Watch consolidation: single recursive watch vs many file watches | Consolidation |
| 17.3.4 | Fallback to polling if watches exhausted | Fallback trigger |

### 17.4 tmpfs Budget

| Task | Description | Deliverable |
|------|-------------|-------------|
| 17.4.1 | Mesh tmpfs size: 256MB default, configurable | Size config |
| 17.4.2 | Usage monitoring: `df` on mesh directory | Usage monitor |
| 17.4.3 | Cleanup triggers: completed tasks >60min, logs >30min | Cleanup policy |
| 17.4.4 | Emergency cleanup: aggressive purge at 90% capacity | Emergency cleanup |

**Exit Criteria:** Mesh operates within resource budgets; no OOM kills; graceful degradation under pressure.

---

# PART III: COORDINATION & COLLABORATION

---

## Phase 18: Leader Election & Federation

**Objective:** Implement Bully Algorithm for coordinator selection without central server.

### 18.1 Bully Algorithm Implementation

| Task | Description | Deliverable |
|------|-------------|-------------|
| 18.1.1 | Priority assignment: capability score, historical reliability, uptime | Priority calculator |
| 18.1.2 | Election trigger: coordinator heartbeat stale (>15s) | Trigger detector |
| 18.1.3 | Election directory: `$MESH_DIR/election/` | Directory structure |
| 18.1.4 | Candidate announcement: write to `election/candidates/` | Announcement |
| 18.1.5 | Victory declaration: highest-priority responsive candidate wins | Victory logic |
| 18.1.6 | Coordinator file: `election/coordinator.json` with UUID, priority, elected_at | Coordinator record |

```bash
check_coordinator() {
    local coord_file="$MESH_DIR/election/coordinator.json"
    local coord_mtime=$(stat -c %Y "$coord_file" 2>/dev/null || echo 0)
    local now=$(date +%s)
    
    if [ $((now - coord_mtime)) -gt 15 ]; then
        initiate_election
    fi
}

initiate_election() {
    # Announce candidacy
    echo "{\"uuid\": \"$AGENT_UUID\", \"priority\": $PRIORITY}" > \
        "$MESH_DIR/election/candidates/$AGENT_UUID.json"
    
    # Wait for other candidates
    sleep 3
    
    # Check if we have highest priority
    local highest=$(find "$MESH_DIR/election/candidates/" -name "*.json" \
        -exec jq -r '.priority' {} \; | sort -rn | head -1)
    
    if [ "$PRIORITY" -eq "$highest" ]; then
        declare_victory
    fi
}
```

### 18.2 Coordinator Responsibilities

| Task | Description | Deliverable |
|------|-------------|-------------|
| 18.2.1 | Task assignment: match tasks to capable agents | Assignment logic |
| 18.2.2 | Dependency tracking: maintain DAG, unblock ready tasks | DAG manager |
| 18.2.3 | Conflict arbitration: decide file ownership disputes | Arbitration |
| 18.2.4 | Health monitoring: track agent status, detect failures | Health monitor |
| 18.2.5 | Consensus initiation: start votes when needed | Consensus trigger |
| 18.2.6 | Heartbeat maintenance: update coordinator heartbeat every 5s | Heartbeat |

### 18.3 Coordinator Failover

| Task | Description | Deliverable |
|------|-------------|-------------|
| 18.3.1 | Failover detection: coordinator heartbeat stale | Detection |
| 18.3.2 | State recovery: read from persistent mesh state | Recovery |
| 18.3.3 | In-flight task handling: reassign or wait for lease expiry | Task recovery |
| 18.3.4 | Split-brain prevention: coordinator epoch numbers | Epoch tracking |

### 18.4 Leaderless Operations

| Task | Description | Deliverable |
|------|-------------|-------------|
| 18.4.1 | DecentLLMs pattern: parallel execution, geometric median aggregation | Leaderless mode |
| 18.4.2 | Mode selection: leaderless for simple decisions, leader for complex | Mode selector |
| 18.4.3 | Hybrid approach: coordinator for orchestration, leaderless for consensus | Hybrid protocol |

**Exit Criteria:** Leader election completes within 5 seconds; failover within 20 seconds; no split-brain scenarios.

---

## Phase 19: Task Management & Kanban Filesystem

**Objective:** Implement filesystem-based task coordination with dependency tracking.

### 19.1 Task Schema

```yaml
# $MESH_DIR/tasks/{status}/task-{id}.yaml
id: task-001
title: "Implement user authentication"
description: "Add OAuth2 login flow with Google provider"
priority: P2                    # P1-P4 (Linear-style)
status: pending                 # pending, claimed, in_progress, blocked, review, done
assignee: null                  # agent-uuid when claimed
created_by: coordinator | human | agent-{uuid}

dependencies:
  hard: [task-000]              # Block start until complete
  soft: [task-002]              # Can start with partial info

created_at: "2026-02-10T12:00:00Z"
claimed_at: null
started_at: null
completed_at: null

artifacts:
  - path: "src/auth/oauth.ts"
    hash: "sha256:abc123..."
    
timeout_seconds: 3600           # 1 hour default

escalation_policy:
  after_600s: notify_lead       # 10 min: ping lead agent
  after_1800s: reassign         # 30 min: try different agent
  after_3600s: escalate_human   # 60 min: human review

acceptance_criteria:
  - "OAuth flow works with Google"
  - "Tests pass"
  - "No security vulnerabilities"
```

### 19.2 Kanban Directory Structure

```
$MESH_DIR/tasks/
├── pending/                    # Ready to claim
├── claimed/                    # Claimed but not started
├── in_progress/                # Active work
├── blocked/                    # Waiting on dependencies
├── review/                     # Awaiting review/approval
├── done/                       # Completed
├── failed/                     # Failed tasks
└── index.yaml                  # Generated read-only overview
```

### 19.3 Atomic Task Operations

| Task | Description | Deliverable |
|------|-------------|-------------|
| 19.3.1 | Claim: `mkdir $MESH_DIR/claims/task-{id}/` (EEXIST = already claimed) | Claim protocol |
| 19.3.2 | Lease: `lease_expires` timestamp, 30-second renewal | Lease protocol |
| 19.3.3 | State transition: atomic `rename()` between directories | State machine |
| 19.3.4 | Dependency resolution: auto-move from blocked to pending when deps complete | DAG walker |
| 19.3.5 | Orphan reclamation: expired leases trigger reassignment | Orphan reclaimer |

### 19.4 Task Allocation Strategies

| Strategy | Use Case | Implementation |
|----------|----------|----------------|
| **Atomic claim** | Interchangeable tasks | First `mkdir` wins |
| **Contract Net** | Heterogeneous capabilities | CFP → Bids → Award |
| **Capability match** | Specialized work | Query capability index |
| **Load balance** | Even distribution | Least-loaded agent |

### 19.5 Contract Net Protocol

```
$MESH_DIR/contracts/
├── cfp/                        # Calls for proposals
│   └── cfp-{id}.yaml          # Task requirements, deadline
├── bids/
│   └── cfp-{id}/
│       └── agent-{uuid}.yaml  # Capability score, time estimate, confidence
└── awards/
    └── cfp-{id}.yaml          # Winner, rationale
```

| Task | Description | Deliverable |
|------|-------------|-------------|
| 19.5.1 | CFP broadcast: coordinator publishes task requirements | CFP publisher |
| 19.5.2 | Bid submission: agents assess fit, submit bids | Bid submitter |
| 19.5.3 | Bid evaluation: score by capability match + confidence + history | Bid evaluator |
| 19.5.4 | Award notification: winner assigned, losers notified | Award notifier |

### 19.6 Living Spec Pattern (Augment Intent)

| Task | Description | Deliverable |
|------|-------------|-------------|
| 19.6.1 | `$MESH_DIR/spec/LIVING_SPEC.md`: evolving requirements document | Living spec |
| 19.6.2 | All agents read and update spec as they progress | Spec protocol |
| 19.6.3 | Spec versioning: track changes, attribute to agents | Version tracking |
| 19.6.4 | Spec conflicts: merge or escalate to coordinator | Conflict handling |

**Exit Criteria:** Tasks flow correctly through states; dependency DAG works; Contract Net improves task-capability matching by >20%.

---

## Phase 20: Blackboard System & Shared Knowledge

**Objective:** Implement shared knowledge base with autonomous volunteering pattern.

### 20.1 Blackboard Architecture

```
$MESH_DIR/blackboard/
├── codebase_map.md             # Project structure, entry points
├── facts.json                  # Discovered facts about codebase
├── decisions.jsonl             # Append-only decision log
├── errors.jsonl                # Error patterns and solutions
├── api_contracts.yaml          # Shared API definitions
├── embeddings.db               # SQLite + sqlite-vec for semantic search
└── requests/
    └── request-{id}.yaml       # Knowledge requests from agents
```

### 20.2 Autonomous Volunteering (Google Research LbMAS Pattern)

| Task | Description | Deliverable |
|------|-------------|-------------|
| 20.2.1 | Knowledge requests: agents post questions to `requests/` | Request protocol |
| 20.2.2 | Capability self-assessment: agents evaluate if they can answer | Assessment logic |
| 20.2.3 | Voluntary response: capable agents write to blackboard | Response protocol |
| 20.2.4 | No master-slave: agents self-select based on expertise | Volunteering |
| 20.2.5 | Research shows: 13-57% improvement over master-slave | Benchmark |

### 20.3 Conflict Resolution Agents

| Task | Description | Deliverable |
|------|-------------|-------------|
| 20.3.1 | Conflict resolver: dedicated agent filters contradictory entries | Resolver agent |
| 20.3.2 | Cleaner agent: removes redundant or stale information | Cleaner agent |
| 20.3.3 | Fact verification: cross-check facts against codebase | Verifier |

### 20.4 Semantic Search

| Task | Description | Deliverable |
|------|-------------|-------------|
| 20.4.1 | Embed blackboard content using local embeddings | Embedding pipeline |
| 20.4.2 | sqlite-vec for vector similarity search | Vector DB |
| 20.4.3 | Query interface: agents search for relevant context | Query API |
| 20.4.4 | Incremental updates: re-embed on changes | Incremental embed |

### 20.5 Stigmergy (Indirect Coordination)

| Task | Description | Deliverable |
|------|-------------|-------------|
| 20.5.1 | Environment as communication: agents observe shared filesystem | Stigmergy pattern |
| 20.5.2 | No direct messaging required for simple coordination | Message-free coord |
| 20.5.3 | Pheromone trails: task completion patterns guide future work | Pattern detection |

**Exit Criteria:** Blackboard queries return relevant results; autonomous volunteering activates for 80%+ of requests; no stale/contradictory facts persist.

---

## Phase 21: Consensus & Voting Mechanisms

**Objective:** Implement confidence-weighted Byzantine consensus for agent decisions.

### 21.1 Voting Protocol Suite

| Protocol | Threshold | Use Case |
|----------|-----------|----------|
| **Simple majority** | >50% | Routine implementation choices |
| **Supermajority** | ≥66% | Architectural changes, API modifications |
| **Unanimity** | 100% | Security-critical, production deployments |
| **Weighted** | ≥66% weighted | Default; incorporates confidence + specialization |

### 21.2 Confidence-Weighted BFT (CP-WBFT Pattern)

| Task | Description | Deliverable |
|------|-------------|-------------|
| 21.2.1 | Confidence probe: structured self-reflection prompt | Probe template |
| 21.2.2 | Two-level probing: prompt-level + behavioral indicators | Probe levels |
| 21.2.3 | Weight formula: `vote_weight = base × confidence × specialization_boost` | Calculator |
| 21.2.4 | Transmission weighting: high-confidence agents influence more | Flow weighting |
| 21.2.5 | Target: survive 85.7% fault rate (CP-WBFT benchmark) | Fault tolerance |

### 21.3 Consensus Directory Structure

```
$MESH_DIR/consensus/
├── active/
│   └── decision-{id}.yaml      # Open decision
├── proposals/
│   └── decision-{id}/
│       └── agent-{uuid}.yaml   # Independent proposals
├── votes/
│   └── decision-{id}/
│       └── agent-{uuid}.yaml   # Weighted votes
└── results/
    └── decision-{id}.yaml      # Final outcome
```

### 21.4 Consensus Protocol Flow

```
1. PROPOSE: Initiator creates decision-{id} in active/
2. DRAFT: All agents independently write to proposals/{id}/
   - All-Agents Drafting: prevents groupthink
   - Wait: all agents OR timeout (60s) OR quorum (66%)
3. SHARE: Proposals revealed simultaneously
   - Fast path: if all proposals hash-identical → accept
4. VOTE: Agents review proposals, write weighted votes
5. TALLY: Aggregate votes per threshold
6. DECIDE: Winner → results/{id}.yaml
```

### 21.5 Aegean Early Termination

| Task | Description | Deliverable |
|------|-------------|-------------|
| 21.5.1 | Incremental quorum detection: stop when outcome is certain | Early termination |
| 21.5.2 | Don't wait for stragglers if quorum reached | Straggler skip |
| 21.5.3 | Target: 1.2-20× latency reduction (Aegean benchmark) | Latency improvement |

### 21.6 Six Sigma Reliability

| Task | Description | Deliverable |
|------|-------------|-------------|
| 21.6.1 | Formula: `system_error = O(p^{⌈n/2⌉})` | Calculator |
| 21.6.2 | 5 agents with 5% per-action error → 0.11% system error | Sizing guide |
| 21.6.3 | 13 agents → 3.4 DPMO (Six Sigma) | Enterprise guide |
| 21.6.4 | Cost-reliability tradeoff visualization | Dashboard |

### 21.7 Heterogeneous Team Requirement

| Task | Description | Deliverable |
|------|-------------|-------------|
| 21.7.1 | Enforce: minimum 2 different model families in any consensus | Diversity enforcer |
| 21.7.2 | Rationale: homogeneous teams have correlated failures | Documentation |
| 21.7.3 | Model diversity > agent count for reliability | Design principle |

**Exit Criteria:** Consensus completes within 2 minutes; heterogeneous teams outperform homogeneous; early termination reduces latency by >50%.

---

## Phase 22: Gated Multi-Agent Debate

**Objective:** Implement debate protocol that triggers only when beneficial, with adaptive stopping.

### 22.1 Debate Gating (iMAD Pattern)

| Task | Description | Deliverable |
|------|-------------|-------------|
| 22.1.1 | Self-critique feature extraction: 41 linguistic/semantic features | Extractor |
| 22.1.2 | Gate classifier: predict if debate will help | Classifier |
| 22.1.3 | Target: ≥70% precision (don't trigger useless debates) | Precision target |
| 22.1.4 | Auto-trigger: high stakes, low confidence, conflicting proposals | Trigger rules |
| 22.1.5 | Auto-skip: unanimous agreement, simple facts, reversible decisions | Skip rules |

### 22.2 Debate Protocol Structure

```
$MESH_DIR/debate/
├── sessions/
│   └── debate-{id}/
│       ├── config.yaml         # Participants, topic, max rounds
│       ├── rounds/
│       │   ├── round-1/
│       │   │   └── agent-{uuid}.yaml  # Position + evidence
│       │   └── round-2/
│       │       └── agent-{uuid}.yaml
│       ├── stability.json      # Belief distribution tracking
│       └── result.yaml
└── active/
    └── debate-{id}
```

### 22.3 Adaptive Stability Detection (NeurIPS 2025)

| Task | Description | Deliverable |
|------|-------------|-------------|
| 22.3.1 | Track belief distributions across rounds | Tracker |
| 22.3.2 | Beta-Binomial mixture model for stability | Statistical model |
| 22.3.3 | KS-test: stop when distribution stable | Stability test |
| 22.3.4 | Maximum 3 rounds (research shows degradation beyond) | Round cap |
| 22.3.5 | Early termination: 2/3 convergence with high confidence | Convergence check |

### 22.4 Tool-MAD Pattern

| Task | Description | Deliverable |
|------|-------------|-------------|
| 22.4.1 | Assign different evidence sources per agent | Tool assignment |
| 22.4.2 | Agent A: searches codebase, Agent B: checks tests, Agent C: reviews docs | Source diversity |
| 22.4.3 | Target: up to 35% improvement (Tool-MAD benchmark) | Improvement target |

### 22.5 Anti-Conformity Measures

| Task | Description | Deliverable |
|------|-------------|-------------|
| 22.5.1 | Hide confidence scores during debate (prevent anchoring) | Confidence masking |
| 22.5.2 | Anonymous initial positions | Position anonymization |
| 22.5.3 | Instruct: "only change beliefs with clear evidence" (FREE-MAD) | Instruction |
| 22.5.4 | Majority pressure mitigation: require justification for position change | Change justification |

**Exit Criteria:** Debate triggers only when predicted to help (≥70% precision); converges within 2 rounds on 80% of cases; heterogeneous teams outperform by ≥5%.

---

## Phase 23: Collaboration Patterns

**Objective:** Implement structured collaboration patterns for quality and error recovery.

### 23.1 Tournament Pattern

| Task | Description | Deliverable |
|------|-------------|-------------|
| 23.1.1 | Fork task to N agents working in parallel | Fork logic |
| 23.1.2 | Each agent produces solution in own branch | Branch per agent |
| 23.1.3 | Run test suite on each solution | Test runner |
| 23.1.4 | Score solutions: test pass rate, coverage, code quality | Scorer |
| 23.1.5 | Merge winner to main | Winner merge |
| 23.1.6 | Evidence: OpenAI 12/12 ICPC 2025, Sakana ALE-Agent | Research backing |

```
[Task] → Fork
  ├→ Agent A → branch solution-a → tests → Score A
  ├→ Agent B → branch solution-b → tests → Score B
  └→ Agent C → branch solution-c → tests → Score C
  → Compare → Merge winner
```

### 23.2 Pair Programming (Atlassian Rovo Dev Pattern)

| Task | Description | Deliverable |
|------|-------------|-------------|
| 23.2.1 | Two agents: driver (implements) + navigator (reviews) | Role assignment |
| 23.2.2 | Turn-taking via `turn_signal.json` lockfile | Turn protocol |
| 23.2.3 | Driver: write code, release turn, signal navigator | Driver flow |
| 23.2.4 | Navigator: review, write feedback, release turn | Navigator flow |
| 23.2.5 | Iterate until convergence (no remaining comments) | Convergence |

### 23.3 Code Review Protocol

```
$MESH_DIR/reviews/
└── pr-{id}/
    ├── metadata.yaml           # Author, reviewer, files
    ├── diff.patch              # Changes
    ├── review.yaml             # Line-level comments + severity
    └── verdict.yaml            # APPROVED | CHANGES_REQUESTED | REJECTED
```

| Task | Description | Deliverable |
|------|-------------|-------------|
| 23.3.1 | State machine: DRAFT → OPEN → APPROVED/CHANGES_REQUESTED/REJECTED | State machine |
| 23.3.2 | Reviewer assignment: different model family than author | Reviewer picker |
| 23.3.3 | Comment severity: nitpick, suggestion, required, blocking | Severity levels |
| 23.3.4 | Re-review after changes | Review cycles |

### 23.4 Saga Pattern (Error Recovery)

| Task | Description | Deliverable |
|------|-------------|-------------|
| 23.4.1 | Each step has compensating transaction | Compensation registry |
| 23.4.2 | Git makes this natural: every change → `git revert` | Git compensation |
| 23.4.3 | Saga log: track steps and their compensations | Saga log |
| 23.4.4 | On failure: execute compensations in reverse order | Rollback executor |

```
Step 1: Agent A writes code       | Compensate: git revert A's commits
Step 2: Agent B writes tests      | Compensate: delete test files  
Step 3: Agent C updates docs      | Compensate: revert doc changes
Step 4: Merge to main             | Compensate: revert merge commit
```

### 23.5 Planner/Worker/Judge (Cursor 2.0 Pattern)

| Task | Description | Deliverable |
|------|-------------|-------------|
| 23.5.1 | Planner agents: explore codebase, create task decompositions | Planner role |
| 23.5.2 | Worker agents: execute tasks, focus only on implementation | Worker role |
| 23.5.3 | Judge agent: evaluate completion, decide continue/stop | Judge role |
| 23.5.4 | Evidence: scales to hundreds of workers (Cursor production) | Scaling proof |
| 23.5.5 | Key insight: "many improvements came from removing complexity" | Simplicity principle |

**Exit Criteria:** Tournament improves solution quality by ≥10%; pair programming reduces defects; saga rollback works correctly.

---

# PART IV: ESCALATION & RELIABILITY

---

## Phase 24: Multi-Tier Escalation Architecture

**Objective:** Exhaust all agent resolution before human escalation; human is absolute last resort.

### 24.1 Escalation Tiers

| Tier | Handler | Trigger | Max Duration |
|------|---------|---------|--------------|
| **0** | Self-resolution | Initial attempt, retries with different approaches | 10 min |
| **1** | Peer review | Self-resolution failed; fresh context window | 10 min |
| **2** | Lead agent | Peer disagreement or uncertainty | 15 min |
| **3** | Agent committee | Lead uncertain; requires consensus | 15 min |
| **4** | Human sponsor | Committee failed; all automated options exhausted | Async (batched) |

### 24.2 Composite Escalation Score

```python
escalation_urgency = (
    0.30 × (1 - confidence_score) +      # Low confidence
    0.25 × (failure_count / max_failures) +  # Repeated failures
    0.25 × (elapsed_time / timeout) +     # Time pressure
    0.20 × risk_category_weight           # Security, production, etc.
)

# Thresholds:
# < 0.2: Continue autonomously
# 0.2-0.4: Peer review (Tier 1)
# 0.4-0.6: Lead agent (Tier 2)
# 0.6-0.8: Agent committee (Tier 3)
# > 0.8: Human escalation (Tier 4)
```

### 24.3 Lazy Escalation Prevention

| Task | Description | Deliverable |
|------|-------------|-------------|
| 24.3.1 | Mandatory checklist: ≥3 approaches documented before escalation | Checklist |
| 24.3.2 | Auto-reject: reject escalation if checklist incomplete | Gate logic |
| 24.3.3 | Causal influence tracking: flag agents that escalate without contributing | Tracker |
| 24.3.4 | Escalation cooldown: same issue can't re-escalate for 5 minutes | Rate limiter |
| 24.3.5 | Intercom insight: "more escalation guidance → more escalation" | Design principle |
| 24.3.6 | Minimize escalation offers: only when genuinely needed | Offer minimization |

### 24.4 Hard Gates (Always Human)

| Category | Examples | Rationale |
|----------|----------|-----------|
| **Credentials** | API keys, passwords, tokens, secrets | Security-critical |
| **Production** | Deploy, database migration, DNS changes | Irreversible |
| **External APIs** | Payments, emails, SMS, webhooks | Side effects |
| **Cost threshold** | >$100 API spend, >1hr compute | Budget control |
| **Data deletion** | DROP TABLE, rm -rf, S3 delete | Irreversible |
| **Dependencies** | Adding new npm/pip packages | Supply chain |
| **Audit** | Compliance logs, access control changes | Regulatory |
| **Schema changes** | Database schema migrations | High risk |

### 24.5 Never Escalate

| Category | Examples | Rationale |
|----------|----------|-----------|
| **Formatting** | Indentation, line length, naming | Style preference |
| **Implementation** | Which loop, variable names, within spec | Within boundaries |
| **Test fixes** | Iterating on failing tests | Self-correcting |
| **Patch deps** | Minor version bumps | Low risk |
| **Documentation** | README, comments, docstrings | Reversible |
| **Refactoring** | Routine code cleanup | Reversible |

### 24.6 Escalation Directory & Payload

```
$MESH_DIR/escalation/
├── queue/                      # Pending human review
│   └── esc-{id}.yaml
├── batched/                    # Grouped for digest
│   └── batch-{date}.yaml
├── blocking/                   # Sync (agent waits)
│   └── esc-{id}.yaml
└── resolved/
    └── esc-{id}.yaml
```

```yaml
# Escalation payload
id: esc-001
type: blocking | async
created_at: "2026-02-10T14:30:00Z"
agent: agent-{uuid}
task: task-{id}
tier_path: [0, 1, 2, 3]         # All tiers exhausted
urgency_score: 0.85
risk_category: production

summary: "Unable to resolve merge conflict in auth module"

context:
  attempts:
    - approach: "Auto-merge with ours strategy"
      result: "Semantic conflict in token validation"
    - approach: "Peer review from agent-xyz"
      result: "Peer also uncertain"
    - approach: "Committee vote"
      result: "2-1 split, no supermajority"
  files_affected: ["src/auth/tokens.ts"]
  confidence_scores:
    agent-abc: 0.4
    agent-xyz: 0.3
    agent-def: 0.6

suggested_actions:
  - "Accept agent-def's approach (highest confidence)"
  - "Manual merge with domain expertise"
  - "Defer and split task differently"
```

### 24.7 Devin Confidence Indicators Pattern

| Task | Description | Deliverable |
|------|-------------|-------------|
| 24.7.1 | Visual confidence indicators: 🟢 high, 🟡 medium, 🔴 low | UI pattern |
| 24.7.2 | 🟢: proceed automatically | Auto-proceed |
| 24.7.3 | 🟡: wait for approval | Approval wait |
| 24.7.4 | 🔴: pause and ask clarifying questions | Clarification mode |
| 24.7.5 | Evidence: Devin 67% PR merge rate (up from 34%) | Metric |

**Exit Criteria:** Human escalations occur only after all 4 tiers exhausted; escalation rate <5%; blocking escalations <1%.

---

## Phase 25: Confidence Quantification & Loop Detection

**Objective:** Reliable confidence assessment and stuck-state detection.

### 25.1 Ensemble Agreement as Confidence Proxy

| Task | Description | Deliverable |
|------|-------------|-------------|
| 25.1.1 | Formula: `confidence = (agreeing_agents / total) × avg_individual_confidence` | Calculator |
| 25.1.2 | Heterogeneous models: agreement across different architectures is strong signal | Design principle |
| 25.1.3 | Disagreement: at least some agents wrong, caution warranted | Disagreement handling |

### 25.2 Self-Consistency Checking

| Task | Description | Deliverable |
|------|-------------|-------------|
| 25.2.1 | Generate 3 responses at different temperatures | Multi-sample |
| 25.2.2 | Semantic clustering: group by similarity | Clustering |
| 25.2.3 | One dominant cluster → high confidence | Confidence mapping |
| 25.2.4 | Multiple clusters → low confidence | Low confidence flag |

### 25.3 Confidence Thresholds

| Threshold | Cross-Agent Agreement | Action |
|-----------|----------------------|--------|
| **High** | ≥80% | Auto-approve, proceed |
| **Medium** | 50-79% | Lightweight peer review, weighted vote |
| **Low** | <50% | Full debate protocol, consider escalation |

### 25.4 Agentic UQ Pattern (January 2026)

| Task | Description | Deliverable |
|------|-------------|-------------|
| 25.4.1 | System 1: Uncertainty-Aware Memory (implicit propagation) | Memory UQ |
| 25.4.2 | System 2: Uncertainty-Aware Reflection (targeted resolution) | Reflection UQ |
| 25.4.3 | Address Curse of Recursion: errors compound in long workflows | Recursion handling |
| 25.4.4 | Request more info when uncertainty high (before escalating) | Info gathering |

### 25.5 Loop and Stuck Detection

| Task | Description | Deliverable |
|------|-------------|-------------|
| 25.5.1 | Ralph Loop pattern: external verification tools as completion criteria | External verification |
| 25.5.2 | Detect repeated outputs: same or similar responses N times | Repetition detector |
| 25.5.3 | Progress monitoring: no measurable change in configurable window | Progress monitor |
| 25.5.4 | Token budget tracking: alert when approaching limits | Budget tracker |
| 25.5.5 | Stuck state → trigger escalation or task reassignment | Stuck handler |

### 25.6 MAST Failure Taxonomy

| Task | Description | Deliverable |
|------|-------------|-------------|
| 25.6.1 | Coordination failures: 36.94% of all failures | Coordination focus |
| 25.6.2 | Specification issues: ~79% when combined with coordination | Spec clarity |
| 25.6.3 | Mitigation: unambiguous resource ownership | Ownership rules |
| 25.6.4 | Each file/API/table belongs to exactly one agent | Ownership registry |

**Exit Criteria:** Confidence scores correlate with actual accuracy (≥0.7 correlation); stuck states detected within 2 minutes; no undetected infinite loops.

---

# PART V: OBSERVABILITY & INTEGRATION

---

## Phase 26: Observability, CLI & Tool Integration

**Objective:** Complete observability stack and integration with existing agent-harness.

### 26.1 Event Logging

```
$MESH_DIR/logs/
├── events.jsonl                # Append-only structured log
├── metrics.jsonl               # Periodic metrics snapshots
├── errors.jsonl                # Error events
└── decisions.jsonl             # All consensus/escalation decisions
```

| Task | Description | Deliverable |
|------|-------------|-------------|
| 26.1.1 | Event schema: timestamp, type, agent, task, payload, correlation_id | Schema |
| 26.1.2 | Event types: agent_registered, task_claimed, consensus_started, debate_triggered, escalation_created, etc. | Type registry |
| 26.1.3 | Correlation IDs: trace events across agents and decisions | Tracing |
| 26.1.4 | Log rotation: hourly, 7-day retention | Rotation policy |

### 26.2 Metrics Collection

| Metric | Description | Alert Threshold |
|--------|-------------|-----------------|
| **agent_count** | Active agents | <2 (degraded) |
| **task_throughput** | Tasks/hour | <5 (blocked) |
| **escalation_rate** | % to human | >10% (review) |
| **consensus_time_p95** | 95th pctl latency | >5 min (slow) |
| **debate_trigger_rate** | % triggering debate | >30% (over-debate) |
| **heartbeat_failures** | Departures/hour | >3 (instability) |
| **confidence_avg** | Mean confidence | <0.5 (uncertainty) |
| **test_pass_rate** | Post-modification | <90% (quality) |

### 26.3 Dashboard Components

| Task | Description | Deliverable |
|------|-------------|-------------|
| 26.3.1 | Agent status panel: active agents, capabilities, current task | Status panel |
| 26.3.2 | Task Kanban: visual flow through states | Kanban view |
| 26.3.3 | Consensus history: decisions, votes, outcomes | Decision log |
| 26.3.4 | Escalation queue: pending human reviews | Escalation panel |
| 26.3.5 | Reliability gauge: Six Sigma calculator output | Reliability view |
| 26.3.6 | Resource utilization: RAM, CPU, watches per agent | Resource view |

### 26.4 CLI Commands

```bash
# Agent management
mesh agents list                      # Show registered agents
mesh agents status {uuid}             # Detailed agent status
mesh agents capabilities              # Capability matrix
mesh agents inject {uuid} "command"  # Inject command via tmux

# Task management
mesh tasks list [--status=pending]    # List tasks
mesh tasks create --file=task.yaml    # Create task
mesh tasks assign {task-id} {agent}   # Manual assignment
mesh tasks cancel {task-id}           # Cancel task

# Consensus & debate
mesh consensus start --file=decision.yaml  # Start decision
mesh consensus status {id}                  # Check status
mesh debate start --topic="..."             # Force debate
mesh debate status {id}                     # Debate progress

# Escalation
mesh escalation queue                 # Show pending
mesh escalation resolve {id}          # Human resolves
mesh escalation batch                 # Generate digest

# Blackboard
mesh blackboard query "search term"   # Semantic search
mesh blackboard facts                 # List facts
mesh blackboard add --file=fact.yaml  # Add fact

# Monitoring
mesh status                           # Overall health
mesh metrics                          # Current metrics
mesh logs [--follow] [--type=error]   # Stream logs

# Worktree management
mesh worktree list                    # Active worktrees
mesh worktree cleanup                 # Remove orphans
mesh merge-queue status               # Merge queue state
```

### 26.5 Notification System

| Channel | Use Case | Implementation |
|---------|----------|----------------|
| **inotify** | Agent-to-agent | Watch inbox directories |
| **SIGUSR1** | Urgent preemption | Signal + flag file |
| **Desktop** | Blocking escalation | notify-send / osascript |
| **Slack/Discord** | Daily digest | Webhook integration |
| **Email** | Critical escalations | SMTP integration |

### 26.6 MCP & A2A Protocol Integration

| Task | Description | Deliverable |
|------|-------------|-------------|
| 26.6.1 | MCP server: expose mesh coordination as MCP tools | MCP server |
| 26.6.2 | Tools: claim_task, submit_result, request_review, escalate | MCP tools |
| 26.6.3 | A2A compatibility: Agent Cards at `.well-known/agent.json` | A2A adapter |
| 26.6.4 | AGENTS.md: project-level agent coordination config | AGENTS.md support |

### 26.7 Integration with Agent-Harness

| Harness Component | Mesh Integration |
|-------------------|------------------|
| Phase 8 file locks | Task claiming uses same primitives |
| Phase 8 intent broadcasting | Pre-edit notifications to mesh |
| Phase 9 cache sharing | Shared blackboard embeddings |
| Phase 10 tracing | Correlation IDs propagated |
| HLC timestamps | Lamport clocks for ordering |
| Conflict detection | Merge queue integration |

### 26.8 Tool-Specific Adapters

| Tool | Adapter | Key Integration |
|------|---------|-----------------|
| **Claude Code** | `adapters/claude-code.sh` | TeammateTool interop, inbox format |
| **Cursor** | `adapters/cursor.sh` | Background Agents API, worktree detection |
| **Aider** | `adapters/aider.sh` | Watch-files, architect mode |
| **Cline** | `adapters/cline.sh` | VS Code IPC, standalone mode |
| **Devin** | `adapters/devin.sh` | Confidence indicators, PR workflow |
| **Codex** | `adapters/codex.sh` | macOS app integration |

**Exit Criteria:** Complete visibility into mesh operations; MCP server functional; CLI provides full control; adapters work for all supported tools.

---

## Implementation Timeline

| Phase | Duration | Dependencies | Key Deliverables |
|-------|----------|--------------|------------------|
| **11** | 1 week | Harness 1-10 | IPC primitives, tmpfs, WAL |
| **12** | 1 week | 11 | Process discovery, registry, heartbeat |
| **13** | 1.5 weeks | 12 | tmux injection, readiness detection |
| **14** | 1 week | 12 | Context injection, AGENT.md |
| **15** | 1 week | 12 | Git worktree isolation, merge queue |
| **16** | 1 week | 15 | Sandboxing, autonomy tiers |
| **17** | 0.5 weeks | 12 | Resource management |
| **18** | 1 week | 12 | Leader election, federation |
| **19** | 1.5 weeks | 18 | Task management, Kanban, Contract Net |
| **20** | 1 week | 19 | Blackboard, autonomous volunteering |
| **21** | 1.5 weeks | 18, 20 | Consensus, voting, BFT |
| **22** | 1 week | 21 | Gated debate, stability detection |
| **23** | 1 week | 21, 22 | Collaboration patterns |
| **24** | 1.5 weeks | 21, 22, 23 | Escalation architecture |
| **25** | 1 week | 21 | Confidence quantification, loop detection |
| **26** | 1.5 weeks | All | Observability, CLI, adapters |

**Total: 17.5 weeks** (some phases can parallelize)

**Critical Path:** 11 → 12 → 18 → 19 → 21 → 24 → 26

---

## Success Metrics

| Metric | Target | Measurement |
|--------|--------|-------------|
| Human escalation rate | <5% | Tier 4 / total decisions |
| Consensus accuracy | >90% | Retrospective review |
| Task throughput | ≥10/hour | Completed tasks/hour |
| P95 consensus latency | <2 min | Time to decision |
| Agent utilization | >70% | Task time / total time |
| Test pass rate | >95% | Post-agent test runs |
| Permission prompts | ≤5/hour | Manual approvals |
| Command injection reliability | >99% | Successful injections |
| Stuck detection | <2 min | Time to detect |
| Debate trigger precision | ≥70% | Useful debates / total |

---

## Risk Mitigation

| Risk | Impact | Mitigation |
|------|--------|------------|
| Correlated failures (homogeneous) | Consensus fails | Enforce 2+ model families |
| Escalation creep | Human overload | Weekly review, tune thresholds |
| Worktree proliferation | Disk exhaustion | Aggressive cleanup, limits |
| Debate over-triggering | Latency | Gate classifier, round cap |
| Consensus deadlock | Stuck decisions | Timeout + lead fallback |
| Sandbox escape | Security | Audit, minimal privilege |
| tmux race conditions | Lost commands | Reliable injection pattern |
| inotify exhaustion | Missed events | Monitor watches, fallback |
| Split-brain leader | Coordination chaos | Epoch numbers, fencing |
| Memory pressure | OOM kills | Per-agent limits, degradation |

---

## Appendix: Complete Directory Structure

```
$MESH_DIR/                              # /tmp/agent-mesh or /dev/shm/agent-mesh
├── agents/                             # Agent manifests
│   └── agent-{uuid}.yaml
├── heartbeats/                         # Touch files
│   └── agent-{uuid}
├── capabilities/                       # Indexed lookups
│   ├── by-language/
│   ├── by-framework/
│   └── by-specialization/
├── discovery/
│   └── process-map.json
├── election/                           # Leader election
│   ├── coordinator.json
│   └── candidates/
├── maildir/                            # Per-agent message queues
│   └── agent-{uuid}/
│       ├── tmp/
│       ├── new/
│       └── cur/
├── tasks/                              # Kanban filesystem
│   ├── pending/
│   ├── claimed/
│   ├── in_progress/
│   ├── blocked/
│   ├── review/
│   ├── done/
│   ├── failed/
│   └── index.yaml
├── claims/                             # Atomic task claims
│   └── task-{id}/
│       ├── owner
│       └── lease_expires
├── contracts/                          # Contract Net
│   ├── cfp/
│   ├── bids/
│   └── awards/
├── consensus/                          # Voting
│   ├── active/
│   ├── proposals/
│   ├── votes/
│   └── results/
├── debate/                             # Multi-agent debate
│   ├── sessions/
│   └── active/
├── escalation/                         # Human escalation
│   ├── queue/
│   ├── batched/
│   ├── blocking/
│   └── resolved/
├── blackboard/                         # Shared knowledge
│   ├── codebase_map.md
│   ├── facts.json
│   ├── decisions.jsonl
│   ├── errors.jsonl
│   ├── api_contracts.yaml
│   ├── embeddings.db
│   └── requests/
├── reviews/                            # Code review
│   └── pr-{id}/
├── merge-queue/                        # Branch merging
│   ├── pending/
│   ├── in_progress/
│   ├── conflicts/
│   └── completed/
├── spec/                               # Living spec
│   └── LIVING_SPEC.md
├── wal/                                # Write-ahead log
│   └── {seq}.log
├── logs/                               # Event logging
│   ├── events.jsonl
│   ├── metrics.jsonl
│   ├── errors.jsonl
│   └── decisions.jsonl
└── workspaces/                         # Per-agent scratch
    └── agent-{uuid}/
```

---

## Appendix B: Protocol Quick Reference

### B.1 Atomic Task Claiming

```bash
claim_task() {
    local task_id="$1"
    local claim_dir="$MESH_DIR/claims/task-$task_id"
    
    # Atomic mkdir - EEXIST means already claimed
    if mkdir "$claim_dir" 2>/dev/null; then
        echo "$AGENT_UUID" > "$claim_dir/owner"
        date -d "+30 seconds" +%s > "$claim_dir/lease_expires"
        
        # Move task to claimed state
        mv "$MESH_DIR/tasks/pending/task-$task_id.yaml" \
           "$MESH_DIR/tasks/claimed/" 2>/dev/null
        return 0
    else
        return 1  # Already claimed
    fi
}

renew_lease() {
    local task_id="$1"
    local claim_dir="$MESH_DIR/claims/task-$task_id"
    
    if [ -f "$claim_dir/owner" ] && [ "$(cat "$claim_dir/owner")" = "$AGENT_UUID" ]; then
        date -d "+30 seconds" +%s > "$claim_dir/lease_expires"
        return 0
    fi
    return 1
}
```

### B.2 Reliable tmux Injection

```bash
inject_command() {
    local target="$1"
    local command="$2"
    local delay="${3:-1.5}"
    
    # Escape special characters
    command=$(printf '%s' "$command" | sed "s/'/'\\\\''/g")
    
    # Send command text (literal mode)
    tmux send-keys -t "$target" -l "$command"
    
    # Critical delay - prevents 15% command loss
    sleep "$delay"
    
    # Send Enter
    tmux send-keys -t "$target" Enter
}

wait_for_ready() {
    local target="$1"
    local pattern="$2"
    local timeout="${3:-120}"
    local interval="${4:-1}"
    local elapsed=0
    
    while [ "$elapsed" -lt "$timeout" ]; do
        local output=$(tmux capture-pane -p -t "$target" -S -10 2>/dev/null)
        if echo "$output" | grep -qE "$pattern"; then
            return 0
        fi
        sleep "$interval"
        elapsed=$((elapsed + interval))
    done
    return 1  # Timeout
}

# Agent-specific readiness patterns
declare -A READY_PATTERNS=(
    ["aider"]="^aider>|^───"
    ["claude-code"]="^>|claude>"
    ["shell"]="\\$|%|#"
)
```

### B.3 Heartbeat Protocol

```bash
heartbeat_loop() {
    local heartbeat_file="$MESH_DIR/heartbeats/$AGENT_UUID"
    
    while true; do
        touch "$heartbeat_file"
        sleep 5
    done
}

check_agent_alive() {
    local agent_uuid="$1"
    local heartbeat_file="$MESH_DIR/heartbeats/$agent_uuid"
    local threshold=15  # seconds
    
    if [ ! -f "$heartbeat_file" ]; then
        return 1  # No heartbeat file
    fi
    
    local mtime=$(stat -c %Y "$heartbeat_file")
    local now=$(date +%s)
    
    if [ $((now - mtime)) -gt "$threshold" ]; then
        return 1  # Stale
    fi
    return 0  # Alive
}
```

### B.4 Consensus Vote Submission

```bash
submit_vote() {
    local decision_id="$1"
    local vote="$2"           # approve | reject | abstain
    local confidence="$3"     # 0.0 - 1.0
    local justification="$4"
    
    local vote_dir="$MESH_DIR/consensus/votes/$decision_id"
    mkdir -p "$vote_dir"
    
    # Calculate weight
    local base_weight=1.0
    local specialization_boost=$(get_specialization_boost "$decision_id")
    local reputation=$(get_reputation_score)
    local weight=$(echo "$base_weight * $confidence * $specialization_boost * $reputation" | bc -l)
    
    cat > "$vote_dir/$AGENT_UUID.yaml" << EOF
agent: $AGENT_UUID
vote: $vote
confidence: $confidence
weight: $weight
justification: "$justification"
timestamp: $(date -Iseconds)
EOF
}

tally_votes() {
    local decision_id="$1"
    local threshold="$2"      # 0.5, 0.66, or 1.0
    
    local vote_dir="$MESH_DIR/consensus/votes/$decision_id"
    local total_weight=0
    local approve_weight=0
    
    for vote_file in "$vote_dir"/*.yaml; do
        local weight=$(yq '.weight' "$vote_file")
        local vote=$(yq '.vote' "$vote_file")
        
        total_weight=$(echo "$total_weight + $weight" | bc -l)
        if [ "$vote" = "approve" ]; then
            approve_weight=$(echo "$approve_weight + $weight" | bc -l)
        fi
    done
    
    local ratio=$(echo "$approve_weight / $total_weight" | bc -l)
    
    if (( $(echo "$ratio >= $threshold" | bc -l) )); then
        echo "approved"
    else
        echo "rejected"
    fi
}
```

### B.5 Escalation Trigger

```bash
calculate_urgency() {
    local confidence="$1"
    local failures="$2"
    local max_failures="$3"
    local elapsed="$4"
    local timeout="$5"
    local risk_weight="$6"
    
    echo "0.30 * (1 - $confidence) + \
          0.25 * ($failures / $max_failures) + \
          0.25 * ($elapsed / $timeout) + \
          0.20 * $risk_weight" | bc -l
}

trigger_escalation() {
    local task_id="$1"
    local urgency="$2"
    local context="$3"
    
    local tier
    if (( $(echo "$urgency < 0.2" | bc -l) )); then
        return 0  # No escalation needed
    elif (( $(echo "$urgency < 0.4" | bc -l) )); then
        tier=1  # Peer review
    elif (( $(echo "$urgency < 0.6" | bc -l) )); then
        tier=2  # Lead agent
    elif (( $(echo "$urgency < 0.8" | bc -l) )); then
        tier=3  # Committee
    else
        tier=4  # Human
    fi
    
    if [ "$tier" -eq 4 ]; then
        # Human escalation
        local esc_file="$MESH_DIR/escalation/queue/esc-$(uuidgen).yaml"
        cat > "$esc_file" << EOF
id: $(basename "$esc_file" .yaml)
type: async
agent: $AGENT_UUID
task: $task_id
urgency_score: $urgency
created_at: $(date -Iseconds)
context: |
$context
EOF
        # Desktop notification
        notify-send "Mesh Escalation" "Human review required for $task_id"
    else
        # Agent escalation
        escalate_to_tier "$tier" "$task_id" "$context"
    fi
}
```

### B.6 Maildir Message Send/Receive

```bash
send_message() {
    local recipient="$1"
    local msg_type="$2"
    local payload="$3"
    
    local msg_id="$(date +%s).$$.$RANDOM"
    local maildir="$MESH_DIR/maildir/$recipient"
    local tmp_file="$maildir/tmp/$msg_id"
    local new_file="$maildir/new/$msg_id"
    
    mkdir -p "$maildir/tmp" "$maildir/new" "$maildir/cur"
    
    cat > "$tmp_file" << EOF
{
    "id": "$msg_id",
    "from": "$AGENT_UUID",
    "to": "$recipient",
    "type": "$msg_type",
    "timestamp": "$(date -Iseconds)",
    "payload": $payload
}
EOF
    
    # Atomic move to new/
    mv "$tmp_file" "$new_file"
}

receive_messages() {
    local maildir="$MESH_DIR/maildir/$AGENT_UUID"
    
    for msg_file in "$maildir/new"/*; do
        [ -f "$msg_file" ] || continue
        
        # Move to cur/ for processing
        local basename=$(basename "$msg_file")
        mv "$msg_file" "$maildir/cur/$basename"
        
        # Process message
        local msg=$(cat "$maildir/cur/$basename")
        process_message "$msg"
        
        # Archive or delete
        rm "$maildir/cur/$basename"
    done
}
```

### B.7 WAL Operations

```bash
wal_write() {
    local operation="$1"
    local data="$2"
    
    local seq=$(get_next_seq)
    local wal_file="$MESH_DIR/wal/$seq.log"
    
    cat > "$wal_file" << EOF
{
    "seq": $seq,
    "operation": "$operation",
    "data": $data,
    "timestamp": "$(date -Iseconds)",
    "status": "pending"
}
EOF
    
    echo "$seq"
}

wal_complete() {
    local seq="$1"
    local wal_file="$MESH_DIR/wal/$seq.log"
    
    # Mark as complete
    local tmp=$(mktemp)
    jq '.status = "complete"' "$wal_file" > "$tmp"
    mv "$tmp" "$wal_file"
}

wal_recover() {
    for wal_file in "$MESH_DIR/wal"/*.log; do
        [ -f "$wal_file" ] || continue
        
        local status=$(jq -r '.status' "$wal_file")
        if [ "$status" = "pending" ]; then
            local operation=$(jq -r '.operation' "$wal_file")
            local data=$(jq '.data' "$wal_file")
            
            # Replay or rollback based on operation type
            if is_idempotent "$operation"; then
                replay_operation "$operation" "$data"
            else
                rollback_operation "$operation" "$data"
            fi
        fi
    done
}
```

---

## Appendix C: Tool-Specific Integration Details

### C.1 Claude Code Agent Teams

**Internal Architecture:**
- TeammateTool with 13 operations: spawnTeam, spawn, write, broadcast, listTeams, listTeammates, assignTask, getTask, getProgress, requestShutdown, setContext, getContext, sendMessage
- Inbox messaging: `~/.claude/teams/{team-name}/messages/{agent-id}/`
- Task tracking: shared task list with states and dependencies
- Warning: "Using both Task tool sub-agents and TeammateTool simultaneously creates confusion"

**Mesh Integration:**
```yaml
# adapters/claude-code.yaml
type: claude-code
detection:
  cmdline_patterns:
    - "node.*claude"
    - "claude-code"
  binary_paths:
    - ~/.local/bin/claude
    - /usr/local/bin/claude
    
interop:
  # Read Claude Code's internal state (read-only)
  team_dir: ~/.claude/teams/
  observe_tasks: true
  observe_messages: false  # Privacy
  
  # Inject via tmux, not internal APIs
  injection_method: tmux
  
  # Respect internal hierarchy
  treat_as_atomic: true
  no_internal_manipulation: true
```

### C.2 Cursor 2.0 Background Agents

**Internal Architecture:**
- Up to 8 parallel agents in cloud VMs
- Branch-based isolation per agent
- POST `https://api.cursor.com/v0/agents` for task submission
- `.cursor/plans/` for plan storage
- Sub-30-second turns with custom Composer model

**Mesh Integration:**
```yaml
# adapters/cursor.yaml
type: cursor
detection:
  cmdline_patterns:
    - "Cursor"
    - "cursor.*--type=renderer"
  process_tree: electron
  
interop:
  # Cursor manages its own worktrees
  respect_cursor_branches: true
  
  # Background Agents API (if accessible)
  api_endpoint: https://api.cursor.com/v0/agents
  
  # Plan observation
  plans_dir: .cursor/plans/
  
  # Remote debugging (optional)
  cdp_port: 9222
```

### C.3 Aider

**Internal Architecture:**
- Single-agent CLI, no internal multi-agent
- Architect mode: two-LLM (propose/edit) is self-contained
- Watch-files with `# AI:` markers
- Native git integration (auto-commits)

**Mesh Integration:**
```yaml
# adapters/aider.yaml
type: aider
detection:
  cmdline_patterns:
    - "python.*aider"
    - "aider"
    
interop:
  # Aider manages its own commits
  git_integration: aider-native
  
  # Context injection via watch files
  watch_file: .aider.watch
  marker_format: "# AI:"
  
  # Read-only context
  context_files:
    - AGENT.md
    - .aider.conf.yml
```

### C.4 Cline / Continue.dev

**Internal Architecture:**
- VS Code extension with IPC socket
- Cline standalone: `cline-core.ts` with gRPC
- Continue.dev: `cn` headless CLI
- Socket: `/run/user/<UID>/vscode-ipc-<UUID>.sock`

**Mesh Integration:**
```yaml
# adapters/cline.yaml
type: cline
detection:
  extension_paths:
    - ~/.vscode/extensions/saoudrizwan.claude-dev-*
    - ~/.vscode/extensions/continue.*
  env_vars:
    - VSCODE_IPC_HOOK_CLI
    
interop:
  # VS Code IPC
  ipc_socket: auto-detect
  
  # Standalone mode preferred for mesh
  prefer_standalone: true
  standalone_command: npx cline-core
  
  # Context injection
  rules_file: .clinerules
```

### C.5 Devin 2.0

**Internal Architecture:**
- Cloud-hosted agent with web UI
- Confidence indicators: 🟢🟡🔴
- PR-based workflow
- 67% PR merge rate (Nov 2025)

**Mesh Integration:**
```yaml
# adapters/devin.yaml
type: devin
detection:
  # Devin runs in cloud, detect via API or webhook
  api_endpoint: https://api.devin.ai/v1
  
interop:
  # Observe Devin's confidence indicators
  confidence_mapping:
    green: 0.8-1.0
    yellow: 0.5-0.8
    red: 0.0-0.5
    
  # PR-based coordination
  pr_workflow: true
  
  # Webhook for status updates
  webhook_endpoint: /devin/status
```

### C.6 OpenAI Codex

**Internal Architecture:**
- macOS Codex App with project organization
- Built-in worktrees
- Automations (scheduled background tasks)
- Review queue for completed work
- GPT-5.3-Codex: 57% SWE-Bench Pro

**Mesh Integration:**
```yaml
# adapters/codex.yaml
type: codex
detection:
  cmdline_patterns:
    - "Codex"
    - "codex-agent"
  app_bundle: /Applications/Codex.app
  
interop:
  # Respect Codex worktrees
  worktree_integration: codex-native
  
  # Review queue observation
  review_queue: ~/.codex/review/
  
  # Automation coordination
  automations_dir: ~/.codex/automations/
```

---

## Appendix D: Research Citations (April 2025 - February 2026)

### D.1 Byzantine Consensus & Reliability

| Paper | Date | Key Finding |
|-------|------|-------------|
| CP-WBFT (Zheng & Tian) | Nov 2025 | 85.7% Byzantine fault tolerance via confidence weighting |
| Aegean (Ruan et al.) | Dec 2025 | 1.2-20× latency reduction with incremental quorum |
| DecentLLMs (Jo & Park) | Jul 2025 | Leaderless BFT with geometric median aggregation |
| Six Sigma Agent (Patel et al.) | Jan 2026 | 14,700× reliability improvement, O(p^{⌈n/2⌉}) formula |

### D.2 Multi-Agent Debate

| Paper | Date | Key Finding |
|-------|------|-------------|
| Wu et al. (Logic Puzzles) | Nov 2025 | Diversity > structure; majority pressure harms accuracy |
| Adaptive Stability (Hu et al.) | Oct 2025 | KS-test stopping criterion, NeurIPS 2025 |
| iMAD (Fan et al.) | Nov 2025 | 41 features for debate gating, selective triggering |
| Tool-MAD | Jan 2026 | 35% improvement with tool diversity per agent |
| ACL 2025 (Kaesberg et al.) | Jul 2025 | Voting > consensus for most tasks |

### D.3 Agentic Systems

| Paper | Date | Key Finding |
|-------|------|-------------|
| Agentic UQ | Jan 2026 | System 1/2 uncertainty, Curse of Recursion |
| LbMAS Blackboard | Sep 2025 | 13-57% improvement with autonomous volunteering |
| MAST Failure Taxonomy | Mar 2025 | 36.94% coordination failures, ownership critical |
| Levels of Autonomy (Feng et al.) | Jun 2025 | Autonomy certificates, boundary-based approval |

### D.4 Production Systems

| System | Date | Key Learning |
|--------|------|--------------|
| Claude Code Agent Teams | Feb 2026 | TeammateTool, inbox messaging, atomic unit model |
| Cursor 2.0 | Oct 2025 | 8 parallel agents, Planner/Worker/Judge |
| Devin 2.0 | Apr 2025 | Confidence indicators, 67% merge rate |
| Augment Intent | Feb 2026 | Living Spec, Context Engine MCP |
| GitHub Agent HQ | Feb 2026 | Vendor-neutral multi-agent orchestration |
| Anthropic Sandboxing | Oct 2025 | 84% prompt reduction, bubblewrap/seatbelt |

### D.5 Protocols & Standards

| Protocol | Date | Status |
|----------|------|--------|
| MCP | Nov 2025 | 97M monthly downloads, Linux Foundation |
| A2A | Jun 2025 | 150+ supporting orgs, Linux Foundation |
| AGENTS.md | Dec 2025 | Agentic AI Foundation |

---

## Appendix E: Integration Checklist

### E.1 Agent-Harness Integration

- [ ] Phase 11 IPC builds on harness `lib/core.sh` primitives
- [ ] Phase 11 WAL integrates with harness event sourcing
- [ ] Phase 12 discovery uses harness process monitoring patterns
- [ ] Phase 15 worktrees integrate with harness file coordination (Phase 8)
- [ ] Phase 16 sandbox policies align with harness security model
- [ ] Phase 19 task DAG uses harness HLC timestamps
- [ ] Phase 21 consensus uses harness tracing correlation IDs
- [ ] Phase 26 metrics feed into harness observability stack
- [ ] All phases respect harness tmpfs management
- [ ] CLI commands follow harness `bin/harness` patterns

### E.2 Tool Compatibility Matrix

| Tool | Detection | Injection | Context | Worktree | Status |
|------|-----------|-----------|---------|----------|--------|
| Claude Code | ✓ cmdline | ✓ tmux | ✓ CLAUDE.md | ✓ native | Ready |
| Cursor | ✓ electron | ✓ tmux/CDP | ✓ .cursorrules | ✓ native | Ready |
| Aider | ✓ python | ✓ tmux | ✓ watch-files | ✓ mesh | Ready |
| Cline | ✓ extension | ✓ IPC/tmux | ✓ .clinerules | ✓ mesh | Ready |
| Devin | ✓ API | ✓ API | ✓ PR comments | ✓ cloud | Partial |
| Codex | ✓ app | ✓ tmux | ✓ AGENT.md | ✓ native | Ready |

### E.3 Pre-Flight Checks

```bash
mesh preflight

# Checks:
# ✓ tmpfs mounted with sufficient space
# ✓ inotify watches available (>10000)
# ✓ tmux installed and accessible
# ✓ Git version ≥2.20 (worktree support)
# ✓ bubblewrap/seatbelt available
# ✓ Required directories created
# ✓ WAL initialized
# ✓ At least one supported agent detected
```

---

## Appendix F: Configuration Reference

### F.1 Main Configuration

```yaml
# ~/.config/agent-mesh/config.yaml

mesh:
  directory: /tmp/agent-mesh    # Or /dev/shm/agent-mesh
  tmpfs_size_mb: 256
  
agents:
  heartbeat_interval_s: 5
  heartbeat_failure_threshold_s: 15
  grace_period_s: 30
  max_agents: 10
  
tasks:
  default_timeout_s: 3600
  lease_duration_s: 30
  lease_renewal_s: 15
  
consensus:
  default_threshold: 0.66       # Supermajority
  proposal_timeout_s: 60
  max_debate_rounds: 3
  require_heterogeneous: true
  min_model_families: 2
  
escalation:
  tier1_max_duration_s: 600     # 10 min
  tier2_max_duration_s: 600     # 10 min
  tier3_max_duration_s: 900     # 15 min
  tier4_max_duration_s: 900     # 15 min
  daily_digest_hour: 9          # 9 AM local
  
sandbox:
  enabled: true
  runtime: auto                 # bubblewrap | seatbelt | none
  network_allowlist:
    - github.com
    - api.anthropic.com
    - api.openai.com
    
notifications:
  desktop: true
  slack_webhook: null
  email: null
  
logging:
  level: info
  retention_days: 7
  rotation_hours: 1
```

### F.2 Per-Project Configuration

```yaml
# .mesh/config.yaml (project root)

project:
  name: my-project
  
autonomy:
  tier: 4                       # Max autonomy tier without human
  test_gate: true
  coverage_threshold: 0.8
  
ownership:
  # Explicit file ownership prevents conflicts
  src/auth/: agent-security-specialist
  src/api/: agent-backend
  src/ui/: agent-frontend
  tests/: any
  docs/: any
  
hard_gates:
  - path: .env*
  - path: secrets/
  - command: "npm publish"
  - command: "git push.*main"
  
custom_escalation:
  database_changes: human       # Always human for DB
  api_contracts: committee      # Committee for API changes
```

### F.3 Agent Manifest Template

```yaml
# $MESH_DIR/agents/agent-{uuid}.yaml

id: agent-{uuid}
type: claude-code
pid: 12345
started_at: "2026-02-10T12:00:00Z"
working_directory: /home/user/project
worktree: .mesh/worktrees/agent-{uuid}
branch: mesh/{uuid}/main

capabilities:
  languages:
    - typescript
    - python
  frameworks:
    - react
    - fastapi
  specializations:
    - security
    - testing
    
operational_design_domain:
  allowed_paths:
    - src/
    - tests/
  forbidden_paths:
    - .env
    - secrets/
  allowed_commands:
    - npm test
    - pytest
  forbidden_commands:
    - rm -rf
    - DROP TABLE
  max_file_size_kb: 1000
  
priority: 100
status: idle
current_task: null

reputation:
  accuracy_30d: 0.92
  escalation_rate_30d: 0.03
  tasks_completed_30d: 47
```

---

## Appendix G: Glossary

| Term | Definition |
|------|------------|
| **A2A** | Agent2Agent protocol (Google, Linux Foundation) |
| **Aegean** | Consensus protocol with incremental quorum detection |
| **Atomic claim** | Using `mkdir` EEXIST semantics for lock-free claiming |
| **Blackboard** | Shared knowledge base all agents read/write |
| **BFT** | Byzantine Fault Tolerance |
| **Bully Algorithm** | Leader election where highest-priority candidate wins |
| **Contract Net** | Task allocation via CFP → Bids → Award |
| **CP-WBFT** | Confidence Probe-Weighted BFT |
| **Hard gate** | Action that always requires human approval |
| **HLC** | Hybrid Logical Clock (Lamport + physical time) |
| **Lease** | Time-limited lock with expiry and renewal |
| **Living Spec** | Evolving requirements document (Augment pattern) |
| **Maildir** | Message queue format: tmp/ → new/ → cur/ |
| **MAST** | Multi-Agent System Failure Taxonomy |
| **MCP** | Model Context Protocol (Anthropic, Linux Foundation) |
| **ODD** | Operational Design Domain (capability boundaries) |
| **Saga** | Error recovery pattern with compensating transactions |
| **Six Sigma** | 3.4 defects per million opportunities |
| **Stigmergy** | Indirect coordination through environment |
| **tmpfs** | RAM-backed filesystem for fast IPC |
| **WAL** | Write-Ahead Log for crash recovery |
| **Worktree** | Git feature for multiple working directories |

---

## Appendix H: Quick Start

```bash
# 1. Initialize mesh
mesh init

# 2. Verify prerequisites
mesh preflight

# 3. Start watchdog daemon
mesh daemon start

# 4. View discovered agents
mesh agents list

# 5. Create a task
cat > task.yaml << EOF
title: "Fix authentication bug"
description: "Users getting logged out unexpectedly"
priority: P1
EOF
mesh tasks create --file=task.yaml

# 6. Monitor progress
mesh status --watch

# 7. View escalation queue (if any)
mesh escalation queue

# 8. Check logs
mesh logs --follow
```
