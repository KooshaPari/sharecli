# Agent-Harness Phase 6+ Multi-Agent Coordination Plan
## Shared-Directory Architecture for Collaborative AI Agents

**Document Version:** 1.0  
**Date:** 2026-02-08  
**Status:** Planning Phase  
**Prerequisite:** Phases 1-5 Complete (3,039 lines implemented)

---

## Executive Summary

This document extends agent-harness for **true multi-agent collaboration in a single shared directory**. Unlike worktree-based isolation (which fragments the codebase and prevents unified testing), this architecture embraces shared state while providing:

- **Parallel git operations** via `GIT_INDEX_FILE` temporary staging
- **Smart file coordination** with optimistic concurrency control
- **Intelligent merge** using AST-aware algorithms (Mergiraf)
- **Request coalescing** to eliminate redundant work across agents
- **Resource isolation** without filesystem separation

**Key Insight:** Git's apparent single-writer limitation is actually confined to two narrow chokepoints—the index lock and per-ref locks—both reducible to nanosecond-scale critical sections using plumbing commands.

---

## Architecture Overview

```
┌──────────────────────────────────────────────────────────────────────────────┐
│                        SHARED DIRECTORY ARCHITECTURE                          │
├──────────────────────────────────────────────────────────────────────────────┤
│                                                                               │
│  ┌─────────┐  ┌─────────┐  ┌─────────┐  ┌─────────┐                         │
│  │ Agent A │  │ Agent B │  │ Agent C │  │ Human   │                         │
│  │ (Claude)│  │ (Cursor)│  │ (Aider) │  │ (root)  │                         │
│  └────┬────┘  └────┬────┘  └────┬────┘  └────┬────┘                         │
│       │            │            │            │                               │
│       └────────────┴─────┬──────┴────────────┘                               │
│                          │                                                    │
│  ┌───────────────────────▼────────────────────────────────────────────────┐  │
│  │                     HARNESS COORDINATION LAYER                          │  │
│  │  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐  ┌────────────┐  │  │
│  │  │ Git Plumbing │  │ File Claims  │  │ Request      │  │ Resource   │  │  │
│  │  │ Manager      │  │ Registry     │  │ Coalescer    │  │ Allocator  │  │  │
│  │  │              │  │              │  │              │  │            │  │  │
│  │  │ GIT_INDEX_   │  │ OCC + lease  │  │ singleflight │  │ ports,     │  │  │
│  │  │ FILE per     │  │ version vec  │  │ dedup        │  │ TMPDIR,    │  │  │
│  │  │ agent        │  │ HLC clocks   │  │ thundering   │  │ env vars   │  │  │
│  │  └──────────────┘  └──────────────┘  └──────────────┘  └────────────┘  │  │
│  │                                                                         │  │
│  │  ┌──────────────────────────────────────────────────────────────────┐  │  │
│  │  │                    SMART MERGE ENGINE                             │  │  │
│  │  │  histogram diff → AST merge (Mergiraf) → auto-resolution rules   │  │  │
│  │  └──────────────────────────────────────────────────────────────────┘  │  │
│  └─────────────────────────────────────────────────────────────────────────┘  │
│                          │                                                    │
│  ┌───────────────────────▼────────────────────────────────────────────────┐  │
│  │                     SINGLE SHARED FILESYSTEM                            │  │
│  │                                                                         │  │
│  │   .git/           src/           tests/          package.json          │  │
│  │   (shared)        (shared)       (shared)        (shared)              │  │
│  │                                                                         │  │
│  │   + One server instance for all features                               │  │
│  │   + Unified test runs                                                  │  │
│  │   + No merge-back complexity                                           │  │
│  └─────────────────────────────────────────────────────────────────────────┘  │
└──────────────────────────────────────────────────────────────────────────────┘
```

---

## Phase 6: Git Plumbing & Parallel Commits (3-4 days)

### 6.1 Temporary Index Manager

**Problem:** `git add` and `git commit` acquire `.git/index.lock` for their entire duration, blocking all other agents.

**Solution:** Each agent gets a private `GIT_INDEX_FILE`, with commits built using plumbing commands.

```bash
# In lib/core.sh

# Configuration
HARNESS_GIT_PARALLEL="${HARNESS_GIT_PARALLEL:-1}"
HARNESS_GIT_INDEX_DIR="${HARNESS_GIT_INDEX_DIR:-$HARNESS_VAR/git-indices}"

# Initialize per-agent temporary index
harness::git::init_agent_index() {
    local agent_id="$1"
    local index_path="${HARNESS_GIT_INDEX_DIR}/${agent_id}.index"
    
    mkdir -p "$HARNESS_GIT_INDEX_DIR"
    
    # Copy current index or read from HEAD
    if [[ -f .git/index ]]; then
        cp .git/index "$index_path"
    else
        GIT_INDEX_FILE="$index_path" git read-tree HEAD
    fi
    
    echo "$index_path"
}

# Stage files to agent's private index (no lock contention)
harness::git::stage() {
    local agent_id="$1"
    shift
    local files=("$@")
    
    local index_path="${HARNESS_GIT_INDEX_DIR}/${agent_id}.index"
    
    for file in "${files[@]}"; do
        # Hash object (parallel-safe, writes to .git/objects/)
        local blob_hash
        blob_hash=$(git hash-object -w "$file")
        
        # Update private index (no main index lock)
        GIT_INDEX_FILE="$index_path" git update-index \
            --cacheinfo "100644,$blob_hash,$file"
    done
}

# Build commit using plumbing (minimal lock time)
harness::git::commit_plumbing() {
    local agent_id="$1"
    local message="$2"
    local target_branch="${3:-HEAD}"
    
    local index_path="${HARNESS_GIT_INDEX_DIR}/${agent_id}.index"
    local max_retries=5
    local retry=0
    
    while [[ $retry -lt $max_retries ]]; do
        # Phase 1: Build tree from private index (parallel-safe)
        local tree_hash
        tree_hash=$(GIT_INDEX_FILE="$index_path" git write-tree)
        
        # Phase 2: Get current HEAD (parallel-safe read)
        local parent_hash
        parent_hash=$(git rev-parse "$target_branch")
        
        # Phase 3: Create commit object (parallel-safe, content-addressed)
        local commit_hash
        commit_hash=$(echo "$message" | git commit-tree "$tree_hash" -p "$parent_hash")
        
        # Phase 4: Atomic CAS ref update (nanoseconds of lock time)
        if git update-ref "refs/heads/${target_branch#refs/heads/}" \
                          "$commit_hash" "$parent_hash" 2>/dev/null; then
            harness::log INFO "git:commit" "Agent $agent_id committed $commit_hash"
            
            # Sync main index with new HEAD
            git read-tree HEAD
            
            # Clean up agent index
            rm -f "$index_path"
            return 0
        fi
        
        # CAS failed - another agent committed, rebase and retry
        harness::log WARN "git:commit" "CAS failed, rebasing (attempt $((retry+1)))"
        
        # Refresh private index with new HEAD
        GIT_INDEX_FILE="$index_path" git read-tree HEAD
        
        # Re-apply our changes
        GIT_INDEX_FILE="$index_path" git checkout-index --all --force
        
        ((retry++))
        sleep "0.$((RANDOM % 100))"  # Jitter to prevent convoy
    done
    
    harness::log ERROR "git:commit" "Failed after $max_retries retries"
    return 1
}
```

### 6.2 Git Operation Classification

```bash
# Classify git commands by their concurrency properties

harness::git::classify() {
    local cmd="$1"
    shift
    local args=("$@")
    
    case "$cmd" in
        # Fully parallel-safe (pure reads against immutable objects)
        log|show|cat-file|rev-parse|rev-list|ls-tree|ls-files|blame|diff-tree)
            echo "PARALLEL"
            ;;
        
        # Read-mostly (may refresh index stat cache)
        status|diff)
            echo "READ_MOSTLY"
            ;;
        
        # Object creation (parallel-safe writes to content-addressed store)
        hash-object|mktree|write-tree|commit-tree)
            echo "OBJECT_CREATE"
            ;;
        
        # Index writers (need private index or serialization)
        add|rm|mv|checkout|reset|stash|merge|rebase|cherry-pick)
            echo "INDEX_WRITE"
            ;;
        
        # Ref writers (need CAS or serialization)
        commit|update-ref|branch|tag)
            echo "REF_WRITE"
            ;;
        
        # Global operations (need exclusive access)
        gc|repack|prune|fsck)
            echo "EXCLUSIVE"
            ;;
        
        # Network operations (can run in parallel, but may update refs)
        fetch|pull|push)
            echo "NETWORK"
            ;;
        
        *)
            echo "UNKNOWN"
            ;;
    esac
}

# Route git commands through appropriate handler
harness::git::execute() {
    local agent_id="${HARNESS_AGENT_ID:-default}"
    local cmd="$1"
    shift
    
    local class
    class=$(harness::git::classify "$cmd" "$@")
    
    case "$class" in
        PARALLEL|OBJECT_CREATE)
            # Execute directly, no coordination needed
            git "$cmd" "$@"
            ;;
        
        READ_MOSTLY)
            # Execute with shared lock (multiple readers OK)
            flock -s .git/index.lock git "$cmd" "$@"
            ;;
        
        INDEX_WRITE)
            if [[ "$HARNESS_GIT_PARALLEL" == "1" ]]; then
                # Use private index
                local index
                index=$(harness::git::init_agent_index "$agent_id")
                GIT_INDEX_FILE="$index" git "$cmd" "$@"
            else
                # Serialize through queue
                harness::git::queue_execute "$cmd" "$@"
            fi
            ;;
        
        REF_WRITE)
            if [[ "$cmd" == "commit" && "$HARNESS_GIT_PARALLEL" == "1" ]]; then
                # Use plumbing commit
                harness::git::commit_plumbing "$agent_id" "$(git log -1 --format=%B)" HEAD
            else
                harness::git::queue_execute "$cmd" "$@"
            fi
            ;;
        
        EXCLUSIVE)
            # Acquire exclusive lock, wait for all agents to pause
            harness::git::exclusive_execute "$cmd" "$@"
            ;;
        
        NETWORK)
            # Serialize network ops to avoid ref conflicts
            harness::git::queue_execute "$cmd" "$@"
            ;;
        
        *)
            # Unknown - be safe, serialize
            harness::git::queue_execute "$cmd" "$@"
            ;;
    esac
}
```

### 6.3 Commit Queue with Fair Ordering

```bash
# FIFO commit queue with CAS retry logic

HARNESS_COMMIT_QUEUE="${HARNESS_VAR}/commit-queue"

harness::git::queue_execute() {
    local cmd="$1"
    shift
    
    mkdir -p "$HARNESS_COMMIT_QUEUE"
    
    # Join queue with ticket
    local ticket
    ticket=$(date +%s%N)_${HARNESS_AGENT_ID:-$$}
    touch "${HARNESS_COMMIT_QUEUE}/${ticket}"
    
    # Wait for our turn (FIFO)
    while true; do
        local first
        first=$(ls -1 "$HARNESS_COMMIT_QUEUE" | head -1)
        if [[ "$first" == "$ticket" ]]; then
            break
        fi
        sleep 0.05
    done
    
    # Execute with lock
    local result
    flock .git/index.lock git "$cmd" "$@"
    result=$?
    
    # Leave queue
    rm -f "${HARNESS_COMMIT_QUEUE}/${ticket}"
    
    return $result
}
```

### 6.4 Scoped Commits (Different Files, Parallel Commits)

```bash
# Allow parallel commits when agents modify different files

harness::git::scoped_commit() {
    local agent_id="$1"
    local message="$2"
    shift 2
    local files=("$@")
    
    # Check for conflicts with other pending commits
    local conflicts=()
    for pending in "$HARNESS_COMMIT_QUEUE"/*; do
        [[ -f "$pending" ]] || continue
        local pending_files
        pending_files=$(cat "${pending}.files" 2>/dev/null || echo "")
        
        for file in "${files[@]}"; do
            if grep -qF "$file" <<< "$pending_files"; then
                conflicts+=("$file")
            fi
        done
    done
    
    if [[ ${#conflicts[@]} -gt 0 ]]; then
        harness::log WARN "git:scoped" "Conflict detected: ${conflicts[*]}"
        # Fall back to queued commit
        harness::git::queue_execute commit -m "$message" -- "${files[@]}"
        return $?
    fi
    
    # No conflicts - use parallel plumbing commit
    # Stage only our files
    harness::git::stage "$agent_id" "${files[@]}"
    
    # Commit with CAS
    harness::git::commit_plumbing "$agent_id" "$message"
}
```

---

## Phase 7: Smart Merge Engine (2-3 days)

### 7.1 Merge Driver Integration

```bash
# Configure intelligent merge drivers

harness::merge::setup() {
    # Install Mergiraf if available
    if ! command -v mergiraf &>/dev/null; then
        harness::log WARN "merge:setup" "Mergiraf not found, using histogram diff only"
        harness::log INFO "merge:setup" "Install: cargo install mergiraf"
    fi
    
    # Configure git to use histogram diff (better for code)
    git config diff.algorithm histogram
    
    # Set up merge drivers in .gitattributes
    cat >> .gitattributes << 'EOF'
# AST-aware merge for code files
*.py merge=mergiraf
*.js merge=mergiraf
*.ts merge=mergiraf
*.tsx merge=mergiraf
*.rs merge=mergiraf
*.go merge=mergiraf
*.java merge=mergiraf
*.c merge=mergiraf
*.cpp merge=mergiraf
*.h merge=mergiraf

# Union merge for additive files
CHANGELOG.md merge=union
*.changelog merge=union

# JSON/YAML structured merge
*.json merge=json-merge
*.yaml merge=yaml-merge
*.yml merge=yaml-merge
package.json merge=npm-merge

# Lock files - keep ours and regenerate
package-lock.json merge=ours
yarn.lock merge=ours
Cargo.lock merge=ours
poetry.lock merge=ours
EOF

    # Configure merge drivers
    git config merge.mergiraf.driver 'mergiraf merge --git %O %A %B -s %S -p %P'
    git config merge.mergiraf.name 'AST-aware merge via Mergiraf'
    
    # JSON merge driver
    git config merge.json-merge.driver 'harness merge json %O %A %B %P'
    git config merge.json-merge.name 'Structured JSON merge'
    
    # NPM package.json merge (combines dependencies)
    git config merge.npm-merge.driver 'harness merge npm %O %A %B %P'
    git config merge.npm-merge.name 'NPM package.json merge'
}
```

### 7.2 Proactive Conflict Detection

```bash
# Detect conflicts BEFORE they happen

harness::merge::predict_conflicts() {
    local agent_id="$1"
    local target_branch="${2:-main}"
    
    # Get files modified by this agent
    local agent_files
    agent_files=$(git diff --name-only HEAD)
    
    # Check each other agent's pending changes
    local conflicts=()
    
    for other_index in "$HARNESS_GIT_INDEX_DIR"/*.index; do
        [[ -f "$other_index" ]] || continue
        [[ "$other_index" == *"${agent_id}.index" ]] && continue
        
        local other_files
        other_files=$(GIT_INDEX_FILE="$other_index" git diff --name-only HEAD)
        
        # Check for overlapping files
        local overlap
        overlap=$(comm -12 <(echo "$agent_files" | sort) <(echo "$other_files" | sort))
        
        if [[ -n "$overlap" ]]; then
            # Do trial merge to check for real conflicts
            while IFS= read -r file; do
                if ! harness::merge::trial_merge "$file" "$other_index"; then
                    conflicts+=("$file")
                fi
            done <<< "$overlap"
        fi
    done
    
    if [[ ${#conflicts[@]} -gt 0 ]]; then
        printf '%s\n' "${conflicts[@]}"
        return 1
    fi
    return 0
}

harness::merge::trial_merge() {
    local file="$1"
    local other_index="$2"
    
    local base other ours
    base=$(git show HEAD:"$file" 2>/dev/null || echo "")
    ours=$(cat "$file" 2>/dev/null || echo "")
    other=$(GIT_INDEX_FILE="$other_index" git show :0:"$file" 2>/dev/null || echo "")
    
    # Three-way merge to temp
    local result
    result=$(mktemp)
    
    if git merge-file -p \
        <(echo "$ours") \
        <(echo "$base") \
        <(echo "$other") > "$result" 2>/dev/null; then
        rm -f "$result"
        return 0  # Clean merge
    else
        rm -f "$result"
        return 1  # Conflict
    fi
}
```

### 7.3 Auto-Resolution Rules

```bash
# Automatic resolution for common conflict patterns

HARNESS_AUTO_RESOLVE="${HARNESS_VAR}/auto-resolve"

harness::merge::auto_resolve() {
    local file="$1"
    local base="$2"
    local ours="$3"
    local theirs="$4"
    local output="$5"
    
    local ext="${file##*.}"
    
    case "$ext" in
        py)
            harness::merge::resolve_python "$base" "$ours" "$theirs" "$output"
            ;;
        js|ts|tsx)
            harness::merge::resolve_javascript "$base" "$ours" "$theirs" "$output"
            ;;
        json)
            harness::merge::resolve_json "$base" "$ours" "$theirs" "$output"
            ;;
        *)
            return 1  # No auto-resolution available
            ;;
    esac
}

harness::merge::resolve_python() {
    local base="$1" ours="$2" theirs="$3" output="$4"
    
    # Extract import sections
    local base_imports ours_imports theirs_imports
    base_imports=$(grep -E '^(import |from .+ import )' "$base" | sort -u)
    ours_imports=$(grep -E '^(import |from .+ import )' "$ours" | sort -u)
    theirs_imports=$(grep -E '^(import |from .+ import )' "$theirs" | sort -u)
    
    # Union of imports (additive)
    local merged_imports
    merged_imports=$(sort -u <<< "$ours_imports"$'\n'"$theirs_imports")
    
    # Check if conflict is import-only
    local base_body ours_body theirs_body
    base_body=$(grep -vE '^(import |from .+ import |#|$)' "$base")
    ours_body=$(grep -vE '^(import |from .+ import |#|$)' "$ours")
    theirs_body=$(grep -vE '^(import |from .+ import |#|$)' "$theirs")
    
    if [[ "$ours_body" == "$theirs_body" ]]; then
        # Only imports differ - take union
        {
            echo "$merged_imports"
            echo ""
            echo "$ours_body"
        } > "$output"
        harness::log INFO "merge:auto" "Auto-resolved import conflict in $file"
        return 0
    fi
    
    return 1  # Non-trivial conflict
}

harness::merge::resolve_json() {
    local base="$1" ours="$2" theirs="$3" output="$4"
    
    # Use jq to merge JSON objects
    if command -v jq &>/dev/null; then
        # Deep merge: theirs * ours (ours wins on conflict)
        jq -s '.[0] * .[1] * .[2]' "$base" "$theirs" "$ours" > "$output" 2>/dev/null
        return $?
    fi
    
    return 1
}
```

---

## Phase 8: File-Level Coordination (2-3 days)

### 8.1 File Claims Registry

```bash
# Lease-based file claims with version tracking

HARNESS_CLAIMS="${HARNESS_VAR}/claims"
HARNESS_CLAIM_TTL="${HARNESS_CLAIM_TTL:-60}"  # Default 60 second lease

harness::claim::init() {
    mkdir -p "$HARNESS_CLAIMS"
    
    # Start background cleanup
    harness::claim::cleanup_daemon &
}

harness::claim::acquire() {
    local agent_id="$1"
    local filepath="$2"
    local claim_type="${3:-write}"  # read|write|exclusive
    
    local claim_key
    claim_key=$(echo "$filepath" | sha256sum | cut -c1-16)
    local claim_file="${HARNESS_CLAIMS}/${claim_key}"
    
    local now
    now=$(date +%s)
    local expires=$((now + HARNESS_CLAIM_TTL))
    
    # Atomic claim acquisition
    (
        flock -x 200
        
        if [[ -f "$claim_file" ]]; then
            local existing
            existing=$(cat "$claim_file")
            local existing_expires existing_type existing_agent
            IFS=: read -r existing_agent existing_type existing_expires _ <<< "$existing"
            
            # Check if existing claim is expired
            if [[ "$existing_expires" -gt "$now" ]]; then
                # Check compatibility
                if [[ "$existing_type" == "exclusive" ]] || 
                   [[ "$claim_type" == "exclusive" ]] ||
                   [[ "$existing_type" == "write" && "$claim_type" == "write" ]]; then
                    echo "BLOCKED:$existing_agent"
                    return 1
                fi
            fi
        fi
        
        # Record file version at claim time
        local version
        if [[ -f "$filepath" ]]; then
            version=$(stat -c '%Y-%s-%i' "$filepath" 2>/dev/null || echo "new")
        else
            version="nonexistent"
        fi
        
        # Write claim
        echo "${agent_id}:${claim_type}:${expires}:${version}:${filepath}" > "$claim_file"
        
    ) 200>"${claim_file}.lock"
    
    harness::log DEBUG "claim:acquire" "Agent $agent_id claimed $filepath ($claim_type)"
    return 0
}

harness::claim::release() {
    local agent_id="$1"
    local filepath="$2"
    
    local claim_key
    claim_key=$(echo "$filepath" | sha256sum | cut -c1-16)
    local claim_file="${HARNESS_CLAIMS}/${claim_key}"
    
    (
        flock -x 200
        
        if [[ -f "$claim_file" ]]; then
            local existing_agent
            existing_agent=$(cut -d: -f1 < "$claim_file")
            
            if [[ "$existing_agent" == "$agent_id" ]]; then
                rm -f "$claim_file"
            fi
        fi
    ) 200>"${claim_file}.lock"
}

harness::claim::verify_unchanged() {
    local filepath="$1"
    
    local claim_key
    claim_key=$(echo "$filepath" | sha256sum | cut -c1-16)
    local claim_file="${HARNESS_CLAIMS}/${claim_key}"
    
    if [[ ! -f "$claim_file" ]]; then
        return 1  # No claim
    fi
    
    local claimed_version
    claimed_version=$(cut -d: -f4 < "$claim_file")
    
    local current_version
    if [[ -f "$filepath" ]]; then
        current_version=$(stat -c '%Y-%s-%i' "$filepath" 2>/dev/null || echo "new")
    else
        current_version="nonexistent"
    fi
    
    [[ "$claimed_version" == "$current_version" ]]
}
```

### 8.2 Optimistic Concurrency Control

```bash
# OCC wrapper for file operations

harness::occ::write() {
    local agent_id="$1"
    local filepath="$2"
    local content_source="$3"  # file path or "-" for stdin
    
    local max_retries=3
    local retry=0
    
    while [[ $retry -lt $max_retries ]]; do
        # Phase 1: Acquire claim and record version
        if ! harness::claim::acquire "$agent_id" "$filepath" "write"; then
            local blocker
            blocker=$(harness::claim::acquire "$agent_id" "$filepath" "write" 2>&1 | grep BLOCKED | cut -d: -f2)
            harness::log WARN "occ:write" "Blocked by $blocker, waiting..."
            sleep 1
            ((retry++))
            continue
        fi
        
        # Phase 2: Perform write to temp file
        local temp_file
        temp_file=$(mktemp "${filepath}.tmp.XXXXXX")
        
        if [[ "$content_source" == "-" ]]; then
            cat > "$temp_file"
        else
            cp "$content_source" "$temp_file"
        fi
        
        # Phase 3: Validate version unchanged
        if ! harness::claim::verify_unchanged "$filepath"; then
            harness::log WARN "occ:write" "File changed during edit, retrying"
            rm -f "$temp_file"
            harness::claim::release "$agent_id" "$filepath"
            ((retry++))
            continue
        fi
        
        # Phase 4: Atomic rename
        mv "$temp_file" "$filepath"
        
        # Phase 5: Release claim
        harness::claim::release "$agent_id" "$filepath"
        
        harness::log DEBUG "occ:write" "Successfully wrote $filepath"
        return 0
    done
    
    harness::log ERROR "occ:write" "Failed after $max_retries retries"
    return 1
}
```

### 8.3 Version Vectors for Causality Tracking

```bash
# Hybrid Logical Clock implementation for file versioning

HARNESS_HLC_STATE="${HARNESS_VAR}/hlc_state"

harness::hlc::init() {
    mkdir -p "$HARNESS_HLC_STATE"
}

harness::hlc::tick() {
    local agent_id="$1"
    local state_file="${HARNESS_HLC_STATE}/${agent_id}"
    
    (
        flock -x 200
        
        local pt lc
        pt=$(date +%s%N | cut -c1-13)  # Millisecond precision
        
        if [[ -f "$state_file" ]]; then
            local prev_pt prev_lc
            IFS=: read -r prev_pt prev_lc < "$state_file"
            
            if [[ "$pt" -le "$prev_pt" ]]; then
                # Physical clock hasn't advanced, increment logical
                lc=$((prev_lc + 1))
                pt="$prev_pt"
            else
                lc=0
            fi
        else
            lc=0
        fi
        
        echo "${pt}:${lc}" > "$state_file"
        echo "${pt}:${lc}:${agent_id}"
        
    ) 200>"${state_file}.lock"
}

harness::hlc::compare() {
    local ts1="$1"  # Format: pt:lc:agent
    local ts2="$2"
    
    local pt1 lc1 pt2 lc2
    IFS=: read -r pt1 lc1 _ <<< "$ts1"
    IFS=: read -r pt2 lc2 _ <<< "$ts2"
    
    if [[ "$pt1" -lt "$pt2" ]]; then
        echo "BEFORE"
    elif [[ "$pt1" -gt "$pt2" ]]; then
        echo "AFTER"
    elif [[ "$lc1" -lt "$lc2" ]]; then
        echo "BEFORE"
    elif [[ "$lc1" -gt "$lc2" ]]; then
        echo "AFTER"
    else
        echo "CONCURRENT"
    fi
}
```

---

## Phase 9: Request Coalescing & Cache Sharing (2-3 days)

### 9.1 Singleflight Pattern

```bash
# Deduplicate identical concurrent requests

HARNESS_INFLIGHT="${HARNESS_VAR}/inflight"

harness::singleflight::execute() {
    local cache_key="$1"
    shift
    local cmd=("$@")
    
    mkdir -p "$HARNESS_INFLIGHT"
    
    local flight_file="${HARNESS_INFLIGHT}/${cache_key}"
    local result_file="${flight_file}.result"
    local waiters_file="${flight_file}.waiters"
    
    (
        flock -x 200
        
        if [[ -f "$flight_file" ]]; then
            # Request in flight - become a waiter
            echo "$$" >> "$waiters_file"
            echo "WAIT"
        else
            # First request - become the executor
            echo "$$" > "$flight_file"
            echo "EXECUTE"
        fi
        
    ) 200>"${flight_file}.lock"
    
    local role
    role=$(harness::singleflight::execute "$cache_key" "${cmd[@]}" | tail -1)
    
    if [[ "$role" == "EXECUTE" ]]; then
        # Execute and broadcast result
        local stdout_file stderr_file rc_file
        stdout_file=$(mktemp)
        stderr_file=$(mktemp)
        
        "${cmd[@]}" > "$stdout_file" 2> "$stderr_file"
        local rc=$?
        
        # Store result
        {
            echo "RC:$rc"
            echo "STDOUT:$stdout_file"
            echo "STDERR:$stderr_file"
        } > "$result_file"
        
        # Signal waiters
        if [[ -f "$waiters_file" ]]; then
            while read -r waiter_pid; do
                kill -USR1 "$waiter_pid" 2>/dev/null || true
            done < "$waiters_file"
        fi
        
        # Output our result
        cat "$stdout_file"
        cat "$stderr_file" >&2
        
        # Cleanup (delayed to allow waiters to read)
        (
            sleep 1
            rm -f "$flight_file" "$result_file" "$waiters_file" "$stdout_file" "$stderr_file"
        ) &
        
        return $rc
    else
        # Wait for result
        local timeout=30
        local waited=0
        
        while [[ ! -f "$result_file" ]] && [[ $waited -lt $timeout ]]; do
            sleep 0.1
            waited=$((waited + 1))
        done
        
        if [[ -f "$result_file" ]]; then
            # Read shared result
            local rc stdout_file stderr_file
            while IFS=: read -r key value; do
                case "$key" in
                    RC) rc="$value" ;;
                    STDOUT) stdout_file="$value" ;;
                    STDERR) stderr_file="$value" ;;
                esac
            done < "$result_file"
            
            cat "$stdout_file"
            cat "$stderr_file" >&2
            return "$rc"
        else
            # Timeout - execute ourselves
            harness::log WARN "singleflight" "Timeout waiting for primary, executing"
            "${cmd[@]}"
        fi
    fi
}
```

### 9.2 Cross-Agent Cache Invalidation

```bash
# Invalidate caches across all agents when files change

HARNESS_INVALIDATION="${HARNESS_VAR}/invalidation"

harness::cache::setup_watcher() {
    mkdir -p "$HARNESS_INVALIDATION"
    
    # Use inotify to watch for changes
    if command -v inotifywait &>/dev/null; then
        inotifywait -m -r -e modify,create,delete,move \
            --format '%w%f:%e' \
            "$PWD" 2>/dev/null | while read -r event; do
            
            local filepath event_type
            IFS=: read -r filepath event_type <<< "$event"
            
            harness::cache::invalidate_for_file "$filepath"
        done &
        
        echo $! > "${HARNESS_VAR}/watcher.pid"
    fi
}

harness::cache::invalidate_for_file() {
    local changed_file="$1"
    
    # Get dependency graph (which cache entries depend on this file)
    local deps_file="${HARNESS_VAR}/deps/${changed_file//\//_}"
    
    if [[ -f "$deps_file" ]]; then
        while read -r cache_key; do
            harness::cache::invalidate "$cache_key"
        done < "$deps_file"
    fi
    
    # Broadcast invalidation to all agents
    local invalidation_msg
    invalidation_msg="${changed_file}:$(date +%s%N)"
    echo "$invalidation_msg" >> "${HARNESS_INVALIDATION}/events"
}

harness::cache::check_invalidations() {
    local last_check="${HARNESS_VAR}/.last_invalidation_check"
    local last_ts=0
    
    [[ -f "$last_check" ]] && last_ts=$(cat "$last_check")
    
    if [[ -f "${HARNESS_INVALIDATION}/events" ]]; then
        while IFS=: read -r filepath ts; do
            if [[ "$ts" -gt "$last_ts" ]]; then
                harness::cache::invalidate_for_file "$filepath"
            fi
        done < "${HARNESS_INVALIDATION}/events"
    fi
    
    date +%s%N > "$last_check"
}
```

### 9.3 Recency-Based Cache Prioritization

```bash
# Heat scoring for cache entries

harness::cache::record_access() {
    local cache_key="$1"
    local agent_id="${2:-unknown}"
    
    local heat_file="${HARNESS_L1_CACHE}/.heat"
    
    (
        flock -x 200
        
        local now
        now=$(date +%s)
        
        # Load existing heat scores
        declare -A heat_scores
        if [[ -f "$heat_file" ]]; then
            while IFS=: read -r key score last_access agents; do
                heat_scores["$key"]="$score:$last_access:$agents"
            done < "$heat_file"
        fi
        
        # Update heat score for this key
        local existing="${heat_scores[$cache_key]:-0:0:}"
        local old_score old_time old_agents
        IFS=: read -r old_score old_time old_agents <<< "$existing"
        
        # Decay old score based on time
        local time_diff=$((now - old_time))
        local decayed_score
        decayed_score=$(echo "$old_score * e(-$time_diff / 300)" | bc -l 2>/dev/null || echo "$old_score")
        
        # Add boost for this access
        local new_score
        new_score=$(echo "$decayed_score + 1.0" | bc -l 2>/dev/null || echo "1.0")
        
        # Track which agents accessed this
        local new_agents="$old_agents"
        if [[ ! "$old_agents" =~ $agent_id ]]; then
            new_agents="${old_agents}${agent_id},"
        fi
        
        heat_scores["$cache_key"]="$new_score:$now:$new_agents"
        
        # Write back
        for key in "${!heat_scores[@]}"; do
            echo "$key:${heat_scores[$key]}"
        done > "$heat_file"
        
    ) 200>"${heat_file}.lock"
}

harness::cache::get_hot_entries() {
    local count="${1:-10}"
    local heat_file="${HARNESS_L1_CACHE}/.heat"
    
    if [[ -f "$heat_file" ]]; then
        sort -t: -k2 -rn "$heat_file" | head -n "$count" | cut -d: -f1
    fi
}
```

---

## Phase 10: Resource Isolation (1-2 days)

### 10.1 Per-Agent TMPDIR

```bash
# Isolated temp directories per agent

harness::resource::setup_agent() {
    local agent_id="$1"
    
    # Private temp directory
    local agent_tmpdir
    agent_tmpdir=$(mktemp -d "/tmp/harness-${agent_id}-XXXXXX")
    export TMPDIR="$agent_tmpdir"
    export TEMP="$agent_tmpdir"
    export TMP="$agent_tmpdir"
    
    # Cleanup on exit
    trap "rm -rf '$agent_tmpdir'" EXIT
    
    # Record for monitoring
    echo "$agent_tmpdir" > "${HARNESS_VAR}/agents/${agent_id}.tmpdir"
}
```

### 10.2 Dynamic Port Allocation

```bash
# Allocate non-conflicting ports for dev servers

HARNESS_PORT_REGISTRY="${HARNESS_VAR}/ports"

harness::port::allocate() {
    local agent_id="$1"
    local service_name="${2:-default}"
    
    mkdir -p "$HARNESS_PORT_REGISTRY"
    
    local port_file="${HARNESS_PORT_REGISTRY}/${agent_id}_${service_name}"
    
    (
        flock -x 200
        
        # Find an available port
        local port
        for port in $(seq 3000 3100); do
            local in_use=0
            
            # Check registry
            for existing in "$HARNESS_PORT_REGISTRY"/*; do
                [[ -f "$existing" ]] || continue
                if [[ $(cat "$existing") == "$port" ]]; then
                    in_use=1
                    break
                fi
            done
            
            # Check actual binding
            if [[ $in_use -eq 0 ]] && ! ss -tlnp | grep -q ":$port "; then
                echo "$port" > "$port_file"
                echo "$port"
                break
            fi
        done
        
    ) 200>"${HARNESS_PORT_REGISTRY}/.lock"
}

harness::port::release() {
    local agent_id="$1"
    local service_name="${2:-default}"
    
    rm -f "${HARNESS_PORT_REGISTRY}/${agent_id}_${service_name}"
}
```

### 10.3 Environment Isolation

```bash
# Per-agent environment variables

harness::env::setup_agent() {
    local agent_id="$1"
    local env_file="${HARNESS_VAR}/agents/${agent_id}.env"
    
    # Base environment (common to all)
    cat > "$env_file" << EOF
HARNESS_AGENT_ID=$agent_id
TMPDIR=$(harness::resource::setup_agent "$agent_id")
EOF
    
    # Agent-specific overrides
    local port
    port=$(harness::port::allocate "$agent_id" "dev")
    echo "PORT=$port" >> "$env_file"
    
    # Database isolation (separate schema/db per agent)
    echo "DATABASE_URL=postgres://localhost/myapp_${agent_id}" >> "$env_file"
    
    # Unique cache directories
    echo "NPM_CONFIG_CACHE=/tmp/harness-${agent_id}/npm" >> "$env_file"
    echo "PIP_CACHE_DIR=/tmp/harness-${agent_id}/pip" >> "$env_file"
}

harness::env::wrap_command() {
    local agent_id="$1"
    shift
    local cmd=("$@")
    
    local env_file="${HARNESS_VAR}/agents/${agent_id}.env"
    
    if [[ -f "$env_file" ]]; then
        env $(cat "$env_file" | xargs) "${cmd[@]}"
    else
        "${cmd[@]}"
    fi
}
```

---

## Phase 11: Inter-Agent Communication (2-3 days)

### 11.1 Intent Broadcasting Protocol

```bash
# Agents announce planned operations

HARNESS_INTENTS="${HARNESS_VAR}/intents"

harness::intent::broadcast() {
    local agent_id="$1"
    local intent_type="$2"  # edit|test|build|commit
    local target="$3"       # file path or "all"
    local duration="${4:-30}"  # estimated duration in seconds
    
    mkdir -p "$HARNESS_INTENTS"
    
    local now
    now=$(date +%s)
    local expires=$((now + duration))
    
    local intent_id="${now}_${agent_id}_${RANDOM}"
    
    cat > "${HARNESS_INTENTS}/${intent_id}" << EOF
agent=$agent_id
type=$intent_type
target=$target
started=$now
expires=$expires
status=active
EOF
    
    # Check for conflicts
    local conflicts=()
    for other_intent in "$HARNESS_INTENTS"/*; do
        [[ -f "$other_intent" ]] || continue
        [[ "$other_intent" == *"$intent_id"* ]] && continue
        
        local other_agent other_target other_type other_expires
        while IFS='=' read -r key value; do
            case "$key" in
                agent) other_agent="$value" ;;
                target) other_target="$value" ;;
                type) other_type="$value" ;;
                expires) other_expires="$value" ;;
            esac
        done < "$other_intent"
        
        # Skip expired intents
        [[ "$other_expires" -lt "$now" ]] && continue
        
        # Check for conflict
        if [[ "$target" == "$other_target" ]] || \
           [[ "$target" == "all" ]] || \
           [[ "$other_target" == "all" ]]; then
            
            if [[ "$intent_type" == "edit" && "$other_type" == "edit" ]] || \
               [[ "$intent_type" == "commit" ]] || \
               [[ "$other_type" == "commit" ]]; then
                conflicts+=("$other_agent:$other_type:$other_target")
            fi
        fi
    done
    
    if [[ ${#conflicts[@]} -gt 0 ]]; then
        printf '%s\n' "${conflicts[@]}"
        return 1
    fi
    
    echo "$intent_id"
    return 0
}

harness::intent::complete() {
    local intent_id="$1"
    
    if [[ -f "${HARNESS_INTENTS}/${intent_id}" ]]; then
        sed -i 's/status=active/status=completed/' "${HARNESS_INTENTS}/${intent_id}"
    fi
}

harness::intent::cancel() {
    local intent_id="$1"
    
    rm -f "${HARNESS_INTENTS}/${intent_id}"
}
```

### 11.2 Task Coordination (Claude Code Agent Teams Compatible)

```bash
# Shared task list with dependencies

HARNESS_TASKS="${HARNESS_VAR}/tasks"

harness::task::create() {
    local task_id="$1"
    local description="$2"
    local depends_on="${3:-}"  # comma-separated task IDs
    
    mkdir -p "$HARNESS_TASKS"
    
    cat > "${HARNESS_TASKS}/${task_id}" << EOF
id=$task_id
description=$description
depends_on=$depends_on
status=pending
assigned_to=
created_at=$(date -Iseconds)
completed_at=
EOF
}

harness::task::claim() {
    local agent_id="$1"
    local task_id="$2"
    
    local task_file="${HARNESS_TASKS}/${task_id}"
    
    (
        flock -x 200
        
        if [[ ! -f "$task_file" ]]; then
            echo "NOTFOUND"
            return 1
        fi
        
        local status assigned_to depends_on
        while IFS='=' read -r key value; do
            case "$key" in
                status) status="$value" ;;
                assigned_to) assigned_to="$value" ;;
                depends_on) depends_on="$value" ;;
            esac
        done < "$task_file"
        
        # Check if already claimed
        if [[ "$status" != "pending" ]]; then
            echo "ALREADY_CLAIMED:$assigned_to"
            return 1
        fi
        
        # Check dependencies
        if [[ -n "$depends_on" ]]; then
            IFS=',' read -ra deps <<< "$depends_on"
            for dep in "${deps[@]}"; do
                local dep_status
                dep_status=$(grep "^status=" "${HARNESS_TASKS}/${dep}" 2>/dev/null | cut -d= -f2)
                if [[ "$dep_status" != "completed" ]]; then
                    echo "BLOCKED:$dep"
                    return 1
                fi
            done
        fi
        
        # Claim the task
        sed -i "s/status=pending/status=in_progress/" "$task_file"
        sed -i "s/assigned_to=/assigned_to=$agent_id/" "$task_file"
        
        echo "CLAIMED"
        
    ) 200>"${task_file}.lock"
}

harness::task::complete() {
    local agent_id="$1"
    local task_id="$2"
    
    local task_file="${HARNESS_TASKS}/${task_id}"
    
    (
        flock -x 200
        
        sed -i "s/status=in_progress/status=completed/" "$task_file"
        sed -i "s/completed_at=/completed_at=$(date -Iseconds)/" "$task_file"
        
    ) 200>"${task_file}.lock"
    
    harness::log INFO "task:complete" "Agent $agent_id completed task $task_id"
}
```

### 11.3 Agent-to-Agent Messaging

```bash
# Inbox-based messaging between agents

HARNESS_INBOXES="${HARNESS_VAR}/inboxes"

harness::message::send() {
    local from_agent="$1"
    local to_agent="$2"
    local message="$3"
    local priority="${4:-normal}"  # urgent|normal|low
    
    mkdir -p "${HARNESS_INBOXES}/${to_agent}"
    
    local msg_id
    msg_id="$(date +%s%N)_${from_agent}"
    
    cat > "${HARNESS_INBOXES}/${to_agent}/${msg_id}" << EOF
from=$from_agent
to=$to_agent
priority=$priority
timestamp=$(date -Iseconds)
read=false
message=$message
EOF
}

harness::message::check() {
    local agent_id="$1"
    local unread_only="${2:-true}"
    
    local inbox="${HARNESS_INBOXES}/${agent_id}"
    
    if [[ ! -d "$inbox" ]]; then
        return 0
    fi
    
    for msg_file in "$inbox"/*; do
        [[ -f "$msg_file" ]] || continue
        
        if [[ "$unread_only" == "true" ]]; then
            local is_read
            is_read=$(grep "^read=" "$msg_file" | cut -d= -f2)
            [[ "$is_read" == "true" ]] && continue
        fi
        
        cat "$msg_file"
        echo "---"
    done
}

harness::message::mark_read() {
    local agent_id="$1"
    local msg_id="$2"
    
    local msg_file="${HARNESS_INBOXES}/${agent_id}/${msg_id}"
    
    if [[ -f "$msg_file" ]]; then
        sed -i 's/read=false/read=true/' "$msg_file"
    fi
}
```

---

## Phase 12: Output Handling & Observability (1-2 days)

### 12.1 Atomic Line-Buffered Logging

```bash
# PIPE_BUF-aware logging for concurrent agents

HARNESS_LOG_FILE="${HARNESS_VAR}/agents.jsonl"

harness::log::atomic() {
    local level="$1"
    local component="$2"
    local message="$3"
    local agent_id="${HARNESS_AGENT_ID:-unknown}"
    
    # Build JSON line (must be < 4096 bytes for atomic write)
    local json
    json=$(printf '{"ts":"%s","agent":"%s","level":"%s","component":"%s","msg":"%s"}\n' \
        "$(date -Iseconds)" \
        "$agent_id" \
        "$level" \
        "$component" \
        "${message:0:3800}")  # Truncate to ensure < PIPE_BUF
    
    # O_APPEND ensures atomic seek+write
    printf '%s' "$json" >> "$HARNESS_LOG_FILE"
}

# Force line-buffered mode for all agent subprocesses
harness::exec::line_buffered() {
    if command -v stdbuf &>/dev/null; then
        stdbuf -oL -eL "$@"
    else
        "$@"
    fi
}
```

### 12.2 Multi-Agent Dashboard Enhancement

```bash
# Add agent activity view to existing dashboard

harness::dashboard::render_agents() {
    local agents_dir="${HARNESS_VAR}/agents"
    
    echo "┌─ AGENT ACTIVITY ──────────────────────────────────────────────────┐"
    
    printf "│ %-10s │ %-8s │ %-10s │ %-8s │ %-18s │\n" \
        "AGENT" "PORT" "TMPDIR MB" "CLAIMS" "LAST ACTIVITY"
    echo "├────────────┼──────────┼────────────┼──────────┼────────────────────┤"
    
    for agent_file in "$agents_dir"/*.env; do
        [[ -f "$agent_file" ]] || continue
        
        local agent_id
        agent_id=$(basename "${agent_file%.env}")
        
        local port tmpdir claims last_activity
        port=$(grep "^PORT=" "$agent_file" | cut -d= -f2)
        
        local tmpdir_path
        tmpdir_path=$(cat "${agents_dir}/${agent_id}.tmpdir" 2>/dev/null || echo "N/A")
        local tmpdir_size="N/A"
        if [[ -d "$tmpdir_path" ]]; then
            tmpdir_size=$(du -sm "$tmpdir_path" 2>/dev/null | cut -f1)
        fi
        
        claims=$(ls -1 "$HARNESS_CLAIMS" 2>/dev/null | grep -c "$agent_id" || echo "0")
        
        last_activity=$(stat -c %Y "${agent_file}" 2>/dev/null || echo "0")
        last_activity=$(date -d "@$last_activity" +%H:%M:%S 2>/dev/null || echo "N/A")
        
        printf "│ %-10s │ %-8s │ %-10s │ %-8s │ %-18s │\n" \
            "$agent_id" "$port" "${tmpdir_size}MB" "$claims" "$last_activity"
    done
    
    echo "└────────────────────────────────────────────────────────────────────┘"
}
```

---

## Implementation Roadmap Summary

| Phase | Focus | Days | Key Deliverables |
|-------|-------|------|------------------|
| **6** | Git Plumbing | 3-4 | GIT_INDEX_FILE manager, plumbing commits, CAS retry |
| **7** | Smart Merge | 2-3 | Mergiraf integration, conflict prediction, auto-resolve |
| **8** | File Coordination | 2-3 | Claims registry, OCC, HLC version vectors |
| **9** | Cache Sharing | 2-3 | Singleflight, cross-agent invalidation, heat scoring |
| **10** | Resource Isolation | 1-2 | TMPDIR, ports, env vars per agent |
| **11** | Communication | 2-3 | Intents, tasks, messaging (Agent Teams compatible) |
| **12** | Observability | 1-2 | Atomic logging, enhanced dashboard |

**Total Estimate:** 14-20 days

---

## Configuration Reference

```bash
# New environment variables for Phase 6+

# Git Plumbing
HARNESS_GIT_PARALLEL=1                    # Enable parallel git operations
HARNESS_GIT_INDEX_DIR="$HARNESS_VAR/git-indices"
HARNESS_GIT_COMMIT_RETRIES=5              # CAS retry attempts

# Smart Merge
HARNESS_MERGE_DRIVER="mergiraf"           # AST merge tool
HARNESS_AUTO_RESOLVE=1                    # Enable auto-resolution
HARNESS_CONFLICT_PREDICTION=1             # Check before commit

# File Coordination
HARNESS_CLAIM_TTL=60                      # Lease duration in seconds
HARNESS_OCC_RETRIES=3                     # OCC write retries
HARNESS_HLC_ENABLED=1                     # Enable hybrid logical clocks

# Request Coalescing
HARNESS_SINGLEFLIGHT=1                    # Deduplicate identical requests
HARNESS_SINGLEFLIGHT_TIMEOUT=30           # Wait timeout for shared result

# Resource Isolation
HARNESS_ISOLATE_TMPDIR=1                  # Per-agent temp dirs
HARNESS_ISOLATE_PORTS=1                   # Dynamic port allocation
HARNESS_PORT_RANGE="3000-3100"            # Port allocation range

# Communication
HARNESS_INTENT_BROADCAST=1                # Enable intent announcements
HARNESS_TASK_COORDINATION=1               # Enable task system
HARNESS_AGENT_MESSAGING=1                 # Enable inbox system
```

---

## Appendix: Research Sources

1. **Git Internals**
   - `GIT_INDEX_FILE` parallel staging: git-scm.com/docs/git
   - `git update-ref` CAS semantics: git-scm.com/docs/git-update-ref
   - Plumbing commands: git-scm.com/book/en/v2/Git-Internals

2. **Merge Algorithms**
   - Histogram diff: jcoglan.com/2017/05/08/merging-with-diff3
   - Mergiraf AST merge: lwn.net/Articles/1042355
   - Three-way merge theory: Khanna, Kunal & Pierce

3. **Concurrency Control**
   - Hybrid Logical Clocks: cse.buffalo.edu/~demirbas/publications/hlc.pdf
   - Singleflight pattern: golang.org/x/sync/singleflight
   - Linux atomic primitives: man7.org/linux/man-pages/man2/open.2.html

4. **Multi-Agent Systems**
   - Claude Code Agent Teams: code.claude.com/docs/en/agent-teams
   - Anthropic's 16-agent C compiler: anthropic.com/engineering/building-c-compiler

5. **Shell Concurrency**
   - flock: man7.org/linux/man-pages/man1/flock.1.html
   - PIPE_BUF atomicity: man7.org/linux/man-pages/man7/pipe.7.html
