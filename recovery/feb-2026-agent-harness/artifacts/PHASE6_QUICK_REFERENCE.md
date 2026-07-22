# Agent-Harness Phase 6+ Quick Reference

## TL;DR: Key Techniques

### Git Parallel Commits (Zero Lock Contention)
```bash
# Instead of: git add && git commit (holds index.lock for seconds)
# Do this: (holds ref lock for nanoseconds)

# 1. Hash files to objects (parallel-safe)
BLOB=$(git hash-object -w myfile.py)

# 2. Build tree in private index (no main lock)
TMPIDX=$(mktemp)
GIT_INDEX_FILE="$TMPIDX" git read-tree HEAD
GIT_INDEX_FILE="$TMPIDX" git update-index --cacheinfo 100644,$BLOB,myfile.py
TREE=$(GIT_INDEX_FILE="$TMPIDX" git write-tree)

# 3. Create commit object (parallel-safe)
PARENT=$(git rev-parse HEAD)
COMMIT=$(echo "message" | git commit-tree $TREE -p $PARENT)

# 4. Atomic CAS ref update (nanoseconds)
git update-ref refs/heads/main $COMMIT $PARENT || retry
```

### File Coordination (OCC Pattern)
```bash
# Optimistic Concurrency Control
version=$(stat -c '%Y-%s-%i' file.txt)  # Read version
# ... do work ...
new_version=$(stat -c '%Y-%s-%i' file.txt)
if [[ "$version" == "$new_version" ]]; then
    mv temp.txt file.txt  # Atomic rename
else
    # Retry - file changed
fi
```

### Request Coalescing (Singleflight)
```bash
# First requester executes, others wait and share result
FLIGHT_DIR=/tmp/flights
KEY=$(echo "ruff check ." | sha256sum | cut -c1-16)

if mkdir "$FLIGHT_DIR/$KEY" 2>/dev/null; then
    # We're the executor
    ruff check . > "$FLIGHT_DIR/$KEY/out" 2>&1
    # Others will read our result
else
    # Wait for result
    while [[ ! -f "$FLIGHT_DIR/$KEY/out" ]]; do sleep 0.1; done
    cat "$FLIGHT_DIR/$KEY/out"
fi
```

### Resource Isolation
```bash
# Per-agent temp directory
export TMPDIR=$(mktemp -d /tmp/agent-${AGENT_ID}-XXXXXX)
trap "rm -rf $TMPDIR" EXIT

# Dynamic port allocation
PORT=0  # Let OS assign
# Or: find unused port in range
for p in {3000..3100}; do
    ss -tlnp | grep -q ":$p " || { PORT=$p; break; }
done
```

---

## Phase Implementation Order

```
Phase 6: Git Plumbing (3-4 days)
├── GIT_INDEX_FILE per agent
├── Plumbing commit builder  
├── CAS retry with jitter
└── Git operation classifier

Phase 7: Smart Merge (2-3 days)
├── Mergiraf integration
├── Conflict prediction
└── Auto-resolve rules

Phase 8: File Coordination (2-3 days)
├── Claims registry (lease-based)
├── OCC write wrapper
└── HLC version vectors

Phase 9: Cache Sharing (2-3 days)
├── Singleflight dedup
├── inotify invalidation
└── Heat-based prioritization

Phase 10: Resource Isolation (1-2 days)
├── Per-agent TMPDIR
├── Dynamic port allocation
└── Environment isolation

Phase 11: Communication (2-3 days)
├── Intent broadcasting
├── Task coordination
└── Agent messaging

Phase 12: Observability (1-2 days)
├── PIPE_BUF-aware logging
└── Multi-agent dashboard
```

---

## Key Configurations

```bash
# Add to ~/.bashrc or harness config

# Enable parallel git
export HARNESS_GIT_PARALLEL=1

# Enable request coalescing
export HARNESS_SINGLEFLIGHT=1

# Enable file claims
export HARNESS_CLAIM_TTL=60

# Enable resource isolation
export HARNESS_ISOLATE_TMPDIR=1
export HARNESS_ISOLATE_PORTS=1
export HARNESS_PORT_RANGE="3000-3100"
```

---

## Command Concurrency Matrix

| Command Type | Parallel-Safe? | Strategy |
|--------------|---------------|----------|
| `git log/show/blame` | ✅ Yes | Direct execute |
| `git status/diff` | ⚠️ Mostly | Shared lock |
| `git hash-object` | ✅ Yes | Direct execute |
| `git add/commit` | ❌ No | GIT_INDEX_FILE |
| `git push/pull` | ❌ No | Queue serialize |
| `ruff/eslint check` | ✅ Yes | Cache + coalesce |
| `npm install` | ❌ No | Queue serialize |
| `pytest/jest` | ⚠️ Depends | Port isolation |

---

## Conflict Resolution Priority

1. **Avoid**: Partition work so agents don't touch same files
2. **Detect**: Predict conflicts before commit
3. **Auto-resolve**: Import unions, JSON merge, identical changes
4. **AST merge**: Mergiraf for structural merge
5. **Human**: Only truly semantic conflicts need review

---

## Anti-Patterns to Avoid

❌ **Don't**: Use git worktrees for week-long tasks  
✅ **Do**: Single directory + coordination layer

❌ **Don't**: Each agent runs its own `npm install`  
✅ **Do**: Coalesce identical commands, share results

❌ **Don't**: Serialize all git operations  
✅ **Do**: Classify by type, parallelize reads/objects

❌ **Don't**: Full CRDTs for batch file edits  
✅ **Do**: OCC + three-way merge at commit time

❌ **Don't**: Poll for file changes  
✅ **Do**: inotify/FSEvents for instant invalidation
