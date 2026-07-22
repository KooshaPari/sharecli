#!/usr/bin/env bash
# =============================================================================
# agent-harness/lib/core.sh — Core library for agent command harness
#
# Provides process-tree detection, rule parsing, caching, and execution
# strategies (coalesce, queue, debounce, passthrough) for deduplicating
# concurrent commands issued by isolated AI agent processes.
# =============================================================================

set -uo pipefail

# ---------------------------------------------------------------------------
# Paths — all derived from HARNESS_HOME (set by the dispatcher)
# ---------------------------------------------------------------------------
HARNESS_HOME="${HARNESS_HOME:?HARNESS_HOME must be set}"
HARNESS_PROXY="${HARNESS_HOME}/proxy"
HARNESS_LIB="${HARNESS_HOME}/lib"
HARNESS_ETC="${HARNESS_HOME}/etc"
HARNESS_VAR="${HARNESS_HOME}/var"
HARNESS_CACHE="${HARNESS_VAR}/cache"
HARNESS_LOCKS="${HARNESS_VAR}/locks"
HARNESS_LOG="${HARNESS_VAR}/log/harness.log"

# Source optional subsystems
[[ -f "${HARNESS_LIB}/readcache.sh" ]] && source "${HARNESS_LIB}/readcache.sh"

# ---------------------------------------------------------------------------
# Configuration (can be overridden via environment)
# ---------------------------------------------------------------------------
HARNESS_LOCK_TIMEOUT="${HARNESS_LOCK_TIMEOUT:-30}"         # Max seconds to wait for lock
HARNESS_STALE_THRESHOLD="${HARNESS_STALE_THRESHOLD:-0}"    # Seconds before stale-while-revalidate triggers (0=disabled)
HARNESS_COMPRESS_THRESHOLD="${HARNESS_COMPRESS_THRESHOLD:-10240}"  # Compress cache entries > 10KB
HARNESS_METRICS_FILE="${HARNESS_VAR}/metrics.dat"          # Metrics counters file
HARNESS_QUEUE_DIR="${HARNESS_VAR}/queue"                   # Priority queue state directory
HARNESS_AGENT_STATS="${HARNESS_VAR}/agent_stats.dat"       # Per-agent usage tracking

# Hierarchical cache configuration
HARNESS_L1_CACHE="${HARNESS_L1_CACHE:-/dev/shm/harness-cache}"  # L1: Memory (tmpfs)
HARNESS_L1_MAX_SIZE="${HARNESS_L1_MAX_SIZE:-104857600}"         # L1 max size: 100MB
HARNESS_L1_TTL="${HARNESS_L1_TTL:-60}"                          # L1 TTL: 60 seconds
HARNESS_L2_CACHE="${HARNESS_CACHE}"                             # L2: Disk (var/cache)
HARNESS_CACHE_HIERARCHY="${HARNESS_CACHE_HIERARCHY:-1}"         # Enable hierarchical cache (0=disabled)

# I/O scheduler configuration
HARNESS_IONICE_ENABLED="${HARNESS_IONICE_ENABLED:-1}"           # Use ionice for disk I/O priority
HARNESS_IONICE_CLASS="${HARNESS_IONICE_CLASS:-2}"               # 1=realtime, 2=best-effort, 3=idle
HARNESS_IONICE_LEVEL="${HARNESS_IONICE_LEVEL:-4}"               # 0-7 (0=highest, 7=lowest)

# Priority levels (lower number = higher priority)
declare -A HARNESS_PRIORITY_LEVELS=(
    [critical]=0
    [high]=1
    [normal]=2
    [low]=3
    [background]=4
)

# I/O priority mapping (priority level → ionice class:level)
declare -A HARNESS_IO_PRIORITY=(
    [critical]="1:0"    # Realtime, highest
    [high]="2:0"        # Best-effort, highest
    [normal]="2:4"      # Best-effort, middle
    [low]="2:7"         # Best-effort, lowest
    [background]="3:0"  # Idle class
)

# Default priorities for command types
declare -A HARNESS_CMD_PRIORITIES=(
    [git]=high        # Git operations often block other work
    [ruff]=normal
    [mypy]=normal
    [pytest]=low      # Tests can wait
    [cargo]=normal
    [npm]=normal
    [make]=normal
)

# ---------------------------------------------------------------------------
# Logging
# ---------------------------------------------------------------------------
harness::log() {
    local level="$1"; shift
    local msg="$*"
    mkdir -p "$(dirname "$HARNESS_LOG")" 2>/dev/null
    printf '[%s] [%s] [pid=%s] %s\n' \
        "$(date '+%Y-%m-%dT%H:%M:%S%z')" "$level" "$$" "$msg" \
        >> "$HARNESS_LOG" 2>/dev/null
}

# ---------------------------------------------------------------------------
# Metrics tracking (Prometheus-compatible counters)
# ---------------------------------------------------------------------------
harness::metrics::init() {
    mkdir -p "$(dirname "$HARNESS_METRICS_FILE")" 2>/dev/null
    [[ -f "$HARNESS_METRICS_FILE" ]] || cat > "$HARNESS_METRICS_FILE" <<'EOF'
cache_hits 0
cache_misses 0
cache_stale_hits 0
lock_timeouts 0
lock_waits 0
commands_executed 0
bytes_compressed 0
refresh_triggered 0
queue_acquired 0
queue_timeouts 0
l1_hits 0
l1_writes 0
l1_evictions 0
l1_promotions 0
l2_hits 0
intents_broadcast 0
intent_conflicts 0
deadlocks_detected 0
deadlocks_resolved 0
EOF
}

harness::metrics::incr() {
    local metric="$1"
    local amount="${2:-1}"
    [[ -f "$HARNESS_METRICS_FILE" ]] || harness::metrics::init
    
    local lock="${HARNESS_VAR}/metrics.lock"
    exec 250>"$lock"
    flock -x 250 2>/dev/null || return
    
    if grep -q "^${metric} " "$HARNESS_METRICS_FILE" 2>/dev/null; then
        local current
        current="$(awk -v m="$metric" '$1==m {print $2}' "$HARNESS_METRICS_FILE")"
        local new=$((current + amount))
        sed -i "s/^${metric} .*/${metric} ${new}/" "$HARNESS_METRICS_FILE"
    else
        echo "${metric} ${amount}" >> "$HARNESS_METRICS_FILE"
    fi
    
    flock -u 250
}

harness::metrics::get() {
    [[ -f "$HARNESS_METRICS_FILE" ]] || { echo "0"; return; }
    awk -v m="$1" '$1==m {print $2; exit}' "$HARNESS_METRICS_FILE" 2>/dev/null || echo "0"
}

harness::metrics::dump() {
    # Prometheus-compatible output
    [[ -f "$HARNESS_METRICS_FILE" ]] || harness::metrics::init
    
    echo "# HELP harness_cache_hits_total Total cache hits"
    echo "# TYPE harness_cache_hits_total counter"
    echo "harness_cache_hits_total $(harness::metrics::get cache_hits)"
    echo ""
    echo "# HELP harness_cache_misses_total Total cache misses"
    echo "# TYPE harness_cache_misses_total counter"
    echo "harness_cache_misses_total $(harness::metrics::get cache_misses)"
    echo ""
    echo "# HELP harness_cache_stale_hits_total Stale cache hits (served while revalidating)"
    echo "# TYPE harness_cache_stale_hits_total counter"
    echo "harness_cache_stale_hits_total $(harness::metrics::get cache_stale_hits)"
    echo ""
    echo "# HELP harness_l1_hits_total L1 (memory) cache hits"
    echo "# TYPE harness_l1_hits_total counter"
    echo "harness_l1_hits_total $(harness::metrics::get l1_hits)"
    echo ""
    echo "# HELP harness_l1_writes_total L1 cache writes"
    echo "# TYPE harness_l1_writes_total counter"
    echo "harness_l1_writes_total $(harness::metrics::get l1_writes)"
    echo ""
    echo "# HELP harness_l1_evictions_total L1 cache evictions"
    echo "# TYPE harness_l1_evictions_total counter"
    echo "harness_l1_evictions_total $(harness::metrics::get l1_evictions)"
    echo ""
    echo "# HELP harness_l1_promotions_total L2 to L1 promotions"
    echo "# TYPE harness_l1_promotions_total counter"
    echo "harness_l1_promotions_total $(harness::metrics::get l1_promotions)"
    echo ""
    echo "# HELP harness_l2_hits_total L2 (disk) cache hits"
    echo "# TYPE harness_l2_hits_total counter"
    echo "harness_l2_hits_total $(harness::metrics::get l2_hits)"
    echo ""
    echo "# HELP harness_lock_timeouts_total Lock acquisition timeouts"
    echo "# TYPE harness_lock_timeouts_total counter"
    echo "harness_lock_timeouts_total $(harness::metrics::get lock_timeouts)"
    echo ""
    echo "# HELP harness_lock_waits_total Times a process had to wait for lock"
    echo "# TYPE harness_lock_waits_total counter"
    echo "harness_lock_waits_total $(harness::metrics::get lock_waits)"
    echo ""
    echo "# HELP harness_commands_executed_total Commands executed (not cached)"
    echo "# TYPE harness_commands_executed_total counter"
    echo "harness_commands_executed_total $(harness::metrics::get commands_executed)"
    echo ""
    echo "# HELP harness_refresh_triggered_total Background refresh operations"
    echo "# TYPE harness_refresh_triggered_total counter"
    echo "harness_refresh_triggered_total $(harness::metrics::get refresh_triggered)"
    echo ""
    echo "# HELP harness_queue_acquired_total Queue slots successfully acquired"
    echo "# TYPE harness_queue_acquired_total counter"
    echo "harness_queue_acquired_total $(harness::metrics::get queue_acquired)"
    echo ""
    echo "# HELP harness_queue_timeouts_total Queue acquisition timeouts"
    echo "# TYPE harness_queue_timeouts_total counter"
    echo "harness_queue_timeouts_total $(harness::metrics::get queue_timeouts)"
    echo ""
    echo "# HELP harness_cache_hit_ratio Cache hit ratio"
    echo "# TYPE harness_cache_hit_ratio gauge"
    local hits misses total ratio
    hits="$(harness::metrics::get cache_hits)"
    misses="$(harness::metrics::get cache_misses)"
    total=$((hits + misses))
    if [[ $total -gt 0 ]]; then
        ratio="$(awk "BEGIN{printf \"%.4f\", ${hits}/${total}}")"
    else
        ratio="0"
    fi
    echo "harness_cache_hit_ratio $ratio"
    echo ""
    echo "# HELP harness_l1_size_bytes L1 cache size in bytes"
    echo "# TYPE harness_l1_size_bytes gauge"
    echo "harness_l1_size_bytes $(harness::cache::l1_size 2>/dev/null || echo 0)"
    echo ""
    echo "# HELP harness_intents_broadcast_total Intent broadcasts"
    echo "# TYPE harness_intents_broadcast_total counter"
    echo "harness_intents_broadcast_total $(harness::metrics::get intents_broadcast)"
    echo ""
    echo "# HELP harness_intent_conflicts_total Intent conflicts detected"
    echo "# TYPE harness_intent_conflicts_total counter"
    echo "harness_intent_conflicts_total $(harness::metrics::get intent_conflicts)"
    echo ""
    echo "# HELP harness_deadlocks_detected_total Deadlocks detected"
    echo "# TYPE harness_deadlocks_detected_total counter"
    echo "harness_deadlocks_detected_total $(harness::metrics::get deadlocks_detected)"
    echo ""
    echo "# HELP harness_deadlocks_resolved_total Deadlocks resolved"
    echo "# TYPE harness_deadlocks_resolved_total counter"
    echo "harness_deadlocks_resolved_total $(harness::metrics::get deadlocks_resolved)"
}

# ---------------------------------------------------------------------------
# Hierarchical Cache System
#
# Two-level cache hierarchy:
#   L1: Memory (tmpfs/dev/shm) — Ultra-fast, limited size, short TTL
#   L2: Disk (var/cache)       — Larger, persistent, longer TTL
#
# Read path:  L1 hit? → return : L2 hit? → promote to L1, return : miss
# Write path: Write to L1 (if fits) and L2
# ---------------------------------------------------------------------------

harness::cache::init() {
    mkdir -p "$HARNESS_L1_CACHE" "$HARNESS_L2_CACHE" 2>/dev/null
}

# Get current L1 cache size in bytes
harness::cache::l1_size() {
    du -sb "$HARNESS_L1_CACHE" 2>/dev/null | cut -f1 || echo "0"
}

# Evict oldest entries from L1 to make room
harness::cache::l1_evict() {
    local target_free="$1"  # bytes to free
    local current_size
    current_size="$(harness::cache::l1_size)"
    
    local need_to_free=$((current_size + target_free - HARNESS_L1_MAX_SIZE))
    [[ $need_to_free -le 0 ]] && return 0
    
    harness::log DEBUG "L1 eviction needed: ${need_to_free} bytes"
    
    # Find oldest files and remove them until we have enough space
    local freed=0
    while IFS= read -r file; do
        [[ -f "$file" ]] || continue
        local file_size
        file_size="$(stat -c%s "$file" 2>/dev/null || echo 0)"
        rm -f "$file"
        freed=$((freed + file_size))
        harness::metrics::incr l1_evictions
        [[ $freed -ge $need_to_free ]] && break
    done < <(find "$HARNESS_L1_CACHE" -type f -printf '%T@ %p\n' 2>/dev/null | sort -n | cut -d' ' -f2-)
    
    harness::log DEBUG "L1 evicted ${freed} bytes"
}

# Check if entry exists in L1 and is fresh
harness::cache::l1_check() {
    local cache_key="$1"
    local ttl="${2:-$HARNESS_L1_TTL}"
    
    local rc_file="${HARNESS_L1_CACHE}/${cache_key}.rc"
    [[ -f "$rc_file" ]] || return 1
    
    local age
    age="$(harness::file_age "$rc_file")"
    [[ "$age" -lt "$ttl" ]]
}

# Read from L1 cache
harness::cache::l1_read() {
    local cache_key="$1"
    local out_var="$2"
    local err_var="$3"
    local rc_var="$4"
    
    local out_file="${HARNESS_L1_CACHE}/${cache_key}.out"
    local err_file="${HARNESS_L1_CACHE}/${cache_key}.err"
    local rc_file="${HARNESS_L1_CACHE}/${cache_key}.rc"
    
    [[ -f "$rc_file" ]] || return 1
    
    # Read values
    local out_content err_content rc_content
    out_content="$(cat "$out_file" 2>/dev/null)"
    err_content="$(cat "$err_file" 2>/dev/null)"
    rc_content="$(cat "$rc_file" 2>/dev/null)"
    
    # Set output variables using nameref
    printf -v "$out_var" '%s' "$out_content"
    printf -v "$err_var" '%s' "$err_content"
    printf -v "$rc_var" '%s' "$rc_content"
    
    return 0
}

# Write to L1 cache
harness::cache::l1_write() {
    local cache_key="$1"
    local out_content="$2"
    local err_content="$3"
    local rc_content="$4"
    
    # Calculate total size
    local total_size=$(( ${#out_content} + ${#err_content} + ${#rc_content} ))
    
    # Skip L1 if entry is too large (>10% of L1 max)
    local max_entry=$((HARNESS_L1_MAX_SIZE / 10))
    if [[ $total_size -gt $max_entry ]]; then
        harness::log DEBUG "L1 skip: entry too large (${total_size} > ${max_entry})"
        return 0
    fi
    
    # Evict if necessary
    harness::cache::l1_evict "$total_size"
    
    # Write files
    mkdir -p "$HARNESS_L1_CACHE" 2>/dev/null
    echo "$out_content" > "${HARNESS_L1_CACHE}/${cache_key}.out"
    echo "$err_content" > "${HARNESS_L1_CACHE}/${cache_key}.err"
    echo "$rc_content" > "${HARNESS_L1_CACHE}/${cache_key}.rc"
    
    harness::metrics::incr l1_writes
}

# Promote entry from L2 to L1
harness::cache::promote_to_l1() {
    local cache_key="$1"
    
    local l2_out="${HARNESS_L2_CACHE}/${cache_key}.out"
    local l2_err="${HARNESS_L2_CACHE}/${cache_key}.err"
    local l2_rc="${HARNESS_L2_CACHE}/${cache_key}.rc"
    
    # Handle compressed files
    if [[ -f "${l2_out}.zst" ]]; then
        l2_out="${l2_out}.zst"
        l2_err="${l2_err}.zst"
        l2_rc="${l2_rc}.zst"
    fi
    
    [[ -f "$l2_rc" ]] || return 1
    
    # Read L2 content (decompress if needed)
    local out_content err_content rc_content
    if [[ "$l2_out" == *.zst ]]; then
        out_content="$(zstd -dqc "$l2_out" 2>/dev/null)"
        err_content="$(zstd -dqc "$l2_err" 2>/dev/null)"
        rc_content="$(zstd -dqc "$l2_rc" 2>/dev/null)"
    else
        out_content="$(cat "$l2_out" 2>/dev/null)"
        err_content="$(cat "$l2_err" 2>/dev/null)"
        rc_content="$(cat "$l2_rc" 2>/dev/null)"
    fi
    
    # Write to L1
    harness::cache::l1_write "$cache_key" "$out_content" "$err_content" "$rc_content"
    harness::metrics::incr l1_promotions
}

# Check L2 cache (existing disk cache)
harness::cache::l2_check() {
    local cache_key="$1"
    local ttl="$2"
    
    local rc_file="${HARNESS_L2_CACHE}/${cache_key}.rc"
    [[ -f "$rc_file" || -f "${rc_file}.zst" ]] || return 1
    
    local check_file="$rc_file"
    [[ -f "${rc_file}.zst" ]] && check_file="${rc_file}.zst"
    
    local age
    age="$(harness::file_age "$check_file")"
    [[ "$age" -lt "$ttl" ]]
}

# ---------------------------------------------------------------------------
# I/O Scheduler Integration
#
# Uses ionice to set I/O priority based on command priority.
# Higher priority commands get better disk access.
# ---------------------------------------------------------------------------

harness::io::get_priority() {
    local priority="${1:-normal}"
    echo "${HARNESS_IO_PRIORITY[$priority]:-2:4}"
}

# Wrap a command with ionice for I/O priority
harness::io::wrap_command() {
    local priority="$1"; shift
    local cmd="$1"; shift
    
    if [[ "$HARNESS_IONICE_ENABLED" != "1" ]] || ! command -v ionice &>/dev/null; then
        # ionice not available, run command directly
        "$cmd" "$@"
        return $?
    fi
    
    local io_spec
    io_spec="$(harness::io::get_priority "$priority")"
    local io_class="${io_spec%%:*}"
    local io_level="${io_spec##*:}"
    
    harness::log DEBUG "I/O priority: class=${io_class} level=${io_level} for $cmd"
    
    ionice -c "$io_class" -n "$io_level" "$cmd" "$@"
}

# Set I/O priority for current process
harness::io::set_priority() {
    local priority="${1:-normal}"
    
    if [[ "$HARNESS_IONICE_ENABLED" != "1" ]] || ! command -v ionice &>/dev/null; then
        return 0
    fi
    
    local io_spec
    io_spec="$(harness::io::get_priority "$priority")"
    local io_class="${io_spec%%:*}"
    local io_level="${io_spec##*:}"
    
    ionice -c "$io_class" -n "$io_level" -p $$ 2>/dev/null
}

# ---------------------------------------------------------------------------
# Negative Stat Cache
#
# Tracks files that returned ENOENT (not found) to avoid repeated lookups.
# Useful for commands that check for optional config files.
# ---------------------------------------------------------------------------

HARNESS_NEGSTAT_CACHE="${HARNESS_L1_CACHE}/negstat"
HARNESS_NEGSTAT_TTL="${HARNESS_NEGSTAT_TTL:-5}"  # 5 second TTL for negative entries

harness::negstat::init() {
    mkdir -p "$HARNESS_NEGSTAT_CACHE" 2>/dev/null
}

# Record a negative stat result
harness::negstat::record() {
    local path="$1"
    local hash
    hash="$(echo "$path" | sha256sum | cut -d' ' -f1)"
    
    harness::negstat::init
    touch "${HARNESS_NEGSTAT_CACHE}/${hash}"
}

# Check if path is known to not exist
harness::negstat::check() {
    local path="$1"
    local hash
    hash="$(echo "$path" | sha256sum | cut -d' ' -f1)"
    
    local cache_file="${HARNESS_NEGSTAT_CACHE}/${hash}"
    [[ -f "$cache_file" ]] || return 1
    
    local age
    age="$(harness::file_age "$cache_file")"
    [[ "$age" -lt "$HARNESS_NEGSTAT_TTL" ]]
}

# Clear negative stat cache
harness::negstat::clear() {
    rm -rf "${HARNESS_NEGSTAT_CACHE:?}/"* 2>/dev/null
}

# ---------------------------------------------------------------------------
# Agent Coordination System
#
# Provides inter-agent communication and conflict resolution:
#   - Intent broadcasting: agents signal planned file operations
#   - Deadlock detection: identifies and resolves lock cycles
#   - Fair share scheduling: ensures equitable resource distribution
# ---------------------------------------------------------------------------

HARNESS_COORD_DIR="${HARNESS_VAR}/coordination"
HARNESS_INTENTS_DIR="${HARNESS_COORD_DIR}/intents"
HARNESS_LOCKS_GRAPH="${HARNESS_COORD_DIR}/locks.graph"
HARNESS_AGENT_SHARES="${HARNESS_COORD_DIR}/shares.dat"
HARNESS_CONFLICT_LOG="${HARNESS_COORD_DIR}/conflicts.log"

# Default share allocation (can be overridden in etc/shares.conf)
declare -A HARNESS_DEFAULT_SHARES=(
    [claude]=25
    [cursor]=25
    [copilot]=25
    [aider]=25
    [unknown]=10
)

harness::coord::init() {
    mkdir -p "$HARNESS_INTENTS_DIR" "$HARNESS_COORD_DIR" 2>/dev/null
    [[ -f "$HARNESS_AGENT_SHARES" ]] || harness::coord::init_shares
}

harness::coord::init_shares() {
    for agent in "${!HARNESS_DEFAULT_SHARES[@]}"; do
        echo "$agent ${HARNESS_DEFAULT_SHARES[$agent]} 0" >> "$HARNESS_AGENT_SHARES"
    done
}

# ---------------------------------------------------------------------------
# Intent Broadcasting
#
# Agents broadcast their intentions before executing file operations.
# Other agents can check for conflicts and adjust their behavior.
# ---------------------------------------------------------------------------

# Broadcast an intent to modify files
# Usage: harness::intent::broadcast <agent> <action> <estimated_ms> <files...>
harness::intent::broadcast() {
    local agent="$1"; shift
    local action="$1"; shift      # read, write, delete
    local est_duration="$1"; shift
    local files=("$@")
    
    harness::coord::init
    
    local intent_id="${agent}.$$.$RANDOM"
    local intent_file="${HARNESS_INTENTS_DIR}/${intent_id}"
    local now
    now="$(date +%s)"
    local expires=$((now + (est_duration / 1000) + 5))  # Add 5s buffer
    
    {
        echo "agent=$agent"
        echo "action=$action"
        echo "pid=$$"
        echo "created=$now"
        echo "expires=$expires"
        echo "duration_ms=$est_duration"
        for f in "${files[@]}"; do
            echo "file=$f"
        done
    } > "$intent_file"
    
    harness::log DEBUG "INTENT broadcast id=${intent_id} action=${action} files=${#files[@]}"
    harness::metrics::incr intents_broadcast
    
    echo "$intent_id"
}

# Clear an intent (call when operation completes)
harness::intent::clear() {
    local intent_id="$1"
    rm -f "${HARNESS_INTENTS_DIR}/${intent_id}" 2>/dev/null
    harness::log DEBUG "INTENT cleared id=${intent_id}"
}

# Check for conflicting intents on files
# Returns 0 if conflict exists, 1 if clear
harness::intent::check_conflicts() {
    local action="$1"; shift
    local files=("$@")
    
    harness::coord::init
    
    local now
    now="$(date +%s)"
    local found_conflict=1
    
    for intent_file in "${HARNESS_INTENTS_DIR}"/*; do
        [[ -f "$intent_file" ]] || continue
        
        # Parse intent
        local intent_agent intent_action intent_expires intent_pid
        while IFS='=' read -r key value; do
            case "$key" in
                agent) intent_agent="$value" ;;
                action) intent_action="$value" ;;
                expires) intent_expires="$value" ;;
                pid) intent_pid="$value" ;;
            esac
        done < "$intent_file"
        
        # Skip expired intents
        if [[ "$intent_expires" -lt "$now" ]]; then
            rm -f "$intent_file"
            continue
        fi
        
        # Skip our own process
        [[ "$intent_pid" == "$$" ]] && continue
        
        # Check if process is still alive
        if ! kill -0 "$intent_pid" 2>/dev/null; then
            rm -f "$intent_file"
            continue
        fi
        
        # Check for file conflicts
        # Write-write and read-write conflicts matter
        if [[ "$action" == "write" || "$intent_action" == "write" ]]; then
            local intent_files=()
            while IFS='=' read -r key value; do
                [[ "$key" == "file" ]] && intent_files+=("$value")
            done < "$intent_file"
            
            for my_file in "${files[@]}"; do
                for their_file in "${intent_files[@]}"; do
                    if [[ "$my_file" == "$their_file" ]]; then
                        harness::log WARN "CONFLICT detected: ${action} on ${my_file} conflicts with ${intent_agent}'s ${intent_action}"
                        echo "${now} ${action} ${my_file} ${intent_agent} ${intent_action}" >> "$HARNESS_CONFLICT_LOG"
                        harness::metrics::incr intent_conflicts
                        found_conflict=0
                    fi
                done
            done
        fi
    done
    
    return $found_conflict
}

# Get all active intents (for debugging/monitoring)
harness::intent::list() {
    harness::coord::init
    
    local now
    now="$(date +%s)"
    
    echo "# Active Intents"
    for intent_file in "${HARNESS_INTENTS_DIR}"/*; do
        [[ -f "$intent_file" ]] || continue
        
        local intent_id
        intent_id="$(basename "$intent_file")"
        
        local agent action expires pid files=()
        while IFS='=' read -r key value; do
            case "$key" in
                agent) agent="$value" ;;
                action) action="$value" ;;
                expires) expires="$value" ;;
                pid) pid="$value" ;;
                file) files+=("$value") ;;
            esac
        done < "$intent_file"
        
        local ttl=$((expires - now))
        [[ $ttl -lt 0 ]] && continue
        
        echo "  ${intent_id}: agent=${agent} action=${action} ttl=${ttl}s files=${#files[@]}"
    done
}

# ---------------------------------------------------------------------------
# Deadlock Detection
#
# Tracks lock dependencies between agents and detects cycles.
# Uses a wait-for graph approach.
# ---------------------------------------------------------------------------

# Record that an agent is holding a lock
harness::deadlock::record_hold() {
    local agent="$1"
    local lock_id="$2"
    
    harness::coord::init
    
    local lock_file="${HARNESS_COORD_DIR}/hold.${agent}.${lock_id}"
    echo "$$ $(date +%s)" > "$lock_file"
}

# Record that an agent is waiting for a lock
harness::deadlock::record_wait() {
    local agent="$1"
    local lock_id="$2"
    local holder="$3"
    
    harness::coord::init
    
    local wait_file="${HARNESS_COORD_DIR}/wait.${agent}.${lock_id}"
    echo "$holder $$ $(date +%s)" > "$wait_file"
}

# Clear lock records for an agent
harness::deadlock::clear() {
    local agent="$1"
    local lock_id="$2"
    
    rm -f "${HARNESS_COORD_DIR}/hold.${agent}.${lock_id}" 2>/dev/null
    rm -f "${HARNESS_COORD_DIR}/wait.${agent}.${lock_id}" 2>/dev/null
}

# Detect deadlock cycles
# Returns 0 if deadlock detected, 1 if clear
harness::deadlock::detect() {
    harness::coord::init
    
    # Build wait-for graph
    declare -A waits_for  # waits_for[agent] = "agent1 agent2 ..."
    
    for wait_file in "${HARNESS_COORD_DIR}"/wait.*; do
        [[ -f "$wait_file" ]] || continue
        
        local basename
        basename="$(basename "$wait_file")"
        # Format: wait.agent.lock_id
        local agent="${basename#wait.}"
        agent="${agent%%.*}"
        
        local holder pid ts
        read -r holder pid ts < "$wait_file" 2>/dev/null || continue
        
        # Check if waiter is still alive
        if ! kill -0 "$pid" 2>/dev/null; then
            rm -f "$wait_file"
            continue
        fi
        
        # Find who holds the lock they're waiting for
        for hold_file in "${HARNESS_COORD_DIR}"/hold.*; do
            [[ -f "$hold_file" ]] || continue
            local hold_basename
            hold_basename="$(basename "$hold_file")"
            local hold_agent="${hold_basename#hold.}"
            hold_agent="${hold_agent%%.*}"
            
            if [[ "$hold_agent" == "$holder" ]]; then
                waits_for[$agent]+="$hold_agent "
            fi
        done
    done
    
    # Detect cycles using DFS
    declare -A visited
    declare -A in_stack
    
    _dfs_cycle() {
        local node="$1"
        visited[$node]=1
        in_stack[$node]=1
        
        for neighbor in ${waits_for[$node]}; do
            if [[ -z "${visited[$neighbor]}" ]]; then
                if _dfs_cycle "$neighbor"; then
                    return 0
                fi
            elif [[ "${in_stack[$neighbor]}" == "1" ]]; then
                harness::log ERROR "DEADLOCK detected: cycle involving $node → $neighbor"
                harness::metrics::incr deadlocks_detected
                return 0
            fi
        done
        
        in_stack[$node]=0
        return 1
    }
    
    for agent in "${!waits_for[@]}"; do
        if [[ -z "${visited[$agent]}" ]]; then
            if _dfs_cycle "$agent"; then
                return 0
            fi
        fi
    done
    
    return 1
}

# Attempt to resolve a deadlock by killing the youngest waiter
harness::deadlock::resolve() {
    harness::coord::init
    
    local youngest_pid=""
    local youngest_ts=0
    local youngest_file=""
    
    for wait_file in "${HARNESS_COORD_DIR}"/wait.*; do
        [[ -f "$wait_file" ]] || continue
        
        local holder pid ts
        read -r holder pid ts < "$wait_file" 2>/dev/null || continue
        
        if [[ "$ts" -gt "$youngest_ts" ]]; then
            youngest_ts="$ts"
            youngest_pid="$pid"
            youngest_file="$wait_file"
        fi
    done
    
    if [[ -n "$youngest_pid" ]]; then
        harness::log WARN "DEADLOCK resolution: aborting pid=$youngest_pid (youngest waiter)"
        kill -TERM "$youngest_pid" 2>/dev/null
        rm -f "$youngest_file"
        harness::metrics::incr deadlocks_resolved
        return 0
    fi
    
    return 1
}

# ---------------------------------------------------------------------------
# Fair Share Scheduling
#
# Tracks resource usage per agent and adjusts priorities to ensure
# equitable distribution of execution time.
# ---------------------------------------------------------------------------

# Update agent's resource usage
harness::fairshare::record_usage() {
    local agent="$1"
    local duration_ms="$2"
    
    harness::coord::init
    
    local lock="${HARNESS_COORD_DIR}/shares.lock"
    exec 252>"$lock"
    flock -x 252 2>/dev/null || return
    
    local temp_file="${HARNESS_AGENT_SHARES}.tmp"
    local found=0
    
    while IFS=' ' read -r a share used; do
        if [[ "$a" == "$agent" ]]; then
            used=$((used + duration_ms))
            found=1
        fi
        echo "$a $share $used"
    done < "$HARNESS_AGENT_SHARES" > "$temp_file"
    
    if [[ $found -eq 0 ]]; then
        echo "$agent 10 $duration_ms" >> "$temp_file"
    fi
    
    mv -f "$temp_file" "$HARNESS_AGENT_SHARES"
    flock -u 252
}

# Get priority adjustment based on fair share
# Returns a number to add to base priority (negative = higher priority)
harness::fairshare::get_adjustment() {
    local agent="$1"
    
    [[ -f "$HARNESS_AGENT_SHARES" ]] || { echo "0"; return; }
    
    # Calculate total shares and usage
    local total_shares=0
    local total_used=0
    local agent_share=10
    local agent_used=0
    
    while IFS=' ' read -r a share used; do
        total_shares=$((total_shares + share))
        total_used=$((total_used + used))
        if [[ "$a" == "$agent" ]]; then
            agent_share="$share"
            agent_used="$used"
        fi
    done < "$HARNESS_AGENT_SHARES"
    
    [[ $total_shares -eq 0 ]] && { echo "0"; return; }
    [[ $total_used -eq 0 ]] && { echo "0"; return; }
    
    # Calculate expected vs actual usage
    local expected_pct=$((agent_share * 100 / total_shares))
    local actual_pct=$((agent_used * 100 / total_used))
    
    # Difference determines adjustment
    # Over quota = positive adjustment (lower priority)
    # Under quota = negative adjustment (higher priority)
    local diff=$((actual_pct - expected_pct))
    
    # Scale: every 10% deviation = 1 priority level
    local adjustment=$((diff / 10))
    
    # Clamp to [-3, +3]
    [[ $adjustment -lt -3 ]] && adjustment=-3
    [[ $adjustment -gt 3 ]] && adjustment=3
    
    echo "$adjustment"
}

# Reset usage counters (call periodically, e.g., every minute)
harness::fairshare::reset() {
    harness::coord::init
    
    local lock="${HARNESS_COORD_DIR}/shares.lock"
    exec 252>"$lock"
    flock -x 252 2>/dev/null || return
    
    local temp_file="${HARNESS_AGENT_SHARES}.tmp"
    
    while IFS=' ' read -r agent share used; do
        # Decay usage by 50% instead of full reset (smoothing)
        used=$((used / 2))
        echo "$agent $share $used"
    done < "$HARNESS_AGENT_SHARES" > "$temp_file"
    
    mv -f "$temp_file" "$HARNESS_AGENT_SHARES"
    flock -u 252
    
    harness::log DEBUG "Fair share counters decayed"
}

# Show fair share status
harness::fairshare::status() {
    [[ -f "$HARNESS_AGENT_SHARES" ]] || { echo "No fair share data"; return; }
    
    local total_shares=0
    local total_used=0
    
    while IFS=' ' read -r a share used; do
        total_shares=$((total_shares + share))
        total_used=$((total_used + used))
    done < "$HARNESS_AGENT_SHARES"
    
    echo "# Fair Share Status"
    printf "%-12s %6s %8s %8s %8s\n" "Agent" "Share" "Used(ms)" "Expected" "Actual"
    printf "%-12s %6s %8s %8s %8s\n" "-----" "-----" "--------" "--------" "------"
    
    while IFS=' ' read -r agent share used; do
        local expected_pct=0
        local actual_pct=0
        [[ $total_shares -gt 0 ]] && expected_pct=$((share * 100 / total_shares))
        [[ $total_used -gt 0 ]] && actual_pct=$((used * 100 / total_used))
        printf "%-12s %5d%% %8d %7d%% %7d%%\n" "$agent" "$share" "$used" "$expected_pct" "$actual_pct"
    done < "$HARNESS_AGENT_SHARES"
}

# ---------------------------------------------------------------------------
# Coordination Cleanup (background task)
# ---------------------------------------------------------------------------

harness::coord::cleanup() {
    harness::coord::init
    
    local now
    now="$(date +%s)"
    local cleaned=0
    
    # Clean expired intents
    for intent_file in "${HARNESS_INTENTS_DIR}"/*; do
        [[ -f "$intent_file" ]] || continue
        
        local expires
        expires="$(grep '^expires=' "$intent_file" 2>/dev/null | cut -d= -f2)"
        if [[ -n "$expires" && "$expires" -lt "$now" ]]; then
            rm -f "$intent_file"
            ((cleaned++))
        fi
    done
    
    # Clean stale lock records (process dead)
    for record in "${HARNESS_COORD_DIR}"/hold.* "${HARNESS_COORD_DIR}"/wait.*; do
        [[ -f "$record" ]] || continue
        
        local pid
        pid="$(awk '{print $1}' "$record" 2>/dev/null)"
        if [[ -n "$pid" ]] && ! kill -0 "$pid" 2>/dev/null; then
            rm -f "$record"
            ((cleaned++))
        fi
    done
    
    [[ $cleaned -gt 0 ]] && harness::log DEBUG "Coordination cleanup: removed $cleaned stale records"
}

# ---------------------------------------------------------------------------
# Interactive Dashboard
#
# Terminal-based real-time monitoring of harness activity.
# Shows agents, cache stats, lock contention, and recent activity.
# ---------------------------------------------------------------------------

HARNESS_DASHBOARD_REFRESH="${HARNESS_DASHBOARD_REFRESH:-2}"  # Refresh interval in seconds

harness::dashboard::render() {
    local width="${1:-80}"
    local now
    now="$(date +%s)"
    
    # Clear screen and move cursor to top
    printf '\033[2J\033[H'
    
    # Header
    printf '┌'
    printf '─%.0s' $(seq 1 $((width - 2)))
    printf '┐\n'
    printf '│  %-*s │\n' $((width - 5)) "agent-harness dashboard                              $(date '+%H:%M:%S')  [q]uit [r]efresh"
    printf '├'
    printf '─%.0s' $(seq 1 $((width - 2)))
    printf '┤\n'
    
    # Cache Stats Section
    printf '│  \033[1mCACHE PERFORMANCE\033[0m%-*s│\n' $((width - 21)) ""
    local hits misses l1_hits l2_hits hit_rate
    hits="$(harness::metrics::get cache_hits)"
    misses="$(harness::metrics::get cache_misses)"
    l1_hits="$(harness::metrics::get l1_hits)"
    l2_hits="$(harness::metrics::get l2_hits)"
    local total=$((hits + misses))
    if [[ $total -gt 0 ]]; then
        hit_rate=$((hits * 100 / total))
    else
        hit_rate=0
    fi
    
    # Progress bar for hit rate
    local bar_width=30
    local filled=$((hit_rate * bar_width / 100))
    local empty=$((bar_width - filled))
    local bar=""
    for ((i=0; i<filled; i++)); do bar+="▓"; done
    for ((i=0; i<empty; i++)); do bar+="░"; done
    
    printf '│    Hit Rate: %s %3d%%%-*s│\n' "$bar" "$hit_rate" $((width - 50)) ""
    printf '│    L1 (mem): %-8d  L2 (disk): %-8d  Total: %-8d%-*s│\n' "$l1_hits" "$l2_hits" "$hits" $((width - 60)) ""
    printf '│    Misses:   %-8d  Commands:  %-8d%-*s│\n' "$misses" "$(harness::metrics::get commands_executed)" $((width - 47)) ""
    printf '│%-*s│\n' $((width - 2)) ""
    
    # Fair Share Section
    printf '│  \033[1mFAIR SHARE ALLOCATION\033[0m%-*s│\n' $((width - 25)) ""
    if [[ -f "$HARNESS_AGENT_SHARES" ]]; then
        local total_used=0
        while IFS=' ' read -r agent share used; do
            total_used=$((total_used + used))
        done < "$HARNESS_AGENT_SHARES"
        
        while IFS=' ' read -r agent share used; do
            local actual_pct=0
            [[ $total_used -gt 0 ]] && actual_pct=$((used * 100 / total_used))
            local share_bar_filled=$((actual_pct * 20 / 100))
            local share_bar=""
            for ((i=0; i<share_bar_filled && i<20; i++)); do share_bar+="▓"; done
            for ((i=share_bar_filled; i<20; i++)); do share_bar+="░"; done
            printf '│    %-10s %s %3d%% (quota: %2d%%)%-*s│\n' "$agent" "$share_bar" "$actual_pct" "$share" $((width - 52)) ""
        done < "$HARNESS_AGENT_SHARES"
    else
        printf '│    (no agent activity recorded)%-*s│\n' $((width - 35)) ""
    fi
    printf '│%-*s│\n' $((width - 2)) ""
    
    # Active Intents Section
    printf '│  \033[1mACTIVE INTENTS\033[0m%-*s│\n' $((width - 18)) ""
    local intent_count=0
    for intent_file in "${HARNESS_INTENTS_DIR}"/*; do
        [[ -f "$intent_file" ]] || continue
        
        local agent action expires
        while IFS='=' read -r key value; do
            case "$key" in
                agent) agent="$value" ;;
                action) action="$value" ;;
                expires) expires="$value" ;;
            esac
        done < "$intent_file"
        
        local ttl=$((expires - now))
        [[ $ttl -lt 0 ]] && continue
        
        printf '│    %-10s %-8s (expires in %ds)%-*s│\n' "$agent" "$action" "$ttl" $((width - 42)) ""
        ((intent_count++))
        [[ $intent_count -ge 5 ]] && break
    done
    [[ $intent_count -eq 0 ]] && printf '│    (no active intents)%-*s│\n' $((width - 27)) ""
    printf '│%-*s│\n' $((width - 2)) ""
    
    # Queue Status Section
    printf '│  \033[1mQUEUE STATUS\033[0m%-*s│\n' $((width - 16)) ""
    local queue_acquired queue_timeouts
    queue_acquired="$(harness::metrics::get queue_acquired)"
    queue_timeouts="$(harness::metrics::get queue_timeouts)"
    printf '│    Acquired: %-8d  Timeouts: %-8d%-*s│\n' "$queue_acquired" "$queue_timeouts" $((width - 43)) ""
    
    local waiting=0
    for entry in "${HARNESS_QUEUE_DIR}"/*; do
        [[ -f "$entry" ]] && ((waiting++))
    done
    printf '│    Currently waiting: %-8d%-*s│\n' "$waiting" $((width - 35)) ""
    printf '│%-*s│\n' $((width - 2)) ""
    
    # Coordination Health
    printf '│  \033[1mCOORDINATION HEALTH\033[0m%-*s│\n' $((width - 23)) ""
    local conflicts deadlocks
    conflicts="$(harness::metrics::get intent_conflicts)"
    deadlocks="$(harness::metrics::get deadlocks_detected)"
    
    if harness::deadlock::detect 2>/dev/null; then
        printf '│    \033[31m⚠ DEADLOCK DETECTED\033[0m%-*s│\n' $((width - 24)) ""
    else
        printf '│    ✓ No deadlock%-*s│\n' $((width - 21)) ""
    fi
    printf '│    Conflicts: %-8d  Deadlocks resolved: %-8d%-*s│\n' "$conflicts" "$(harness::metrics::get deadlocks_resolved)" $((width - 53)) ""
    
    # Footer
    printf '├'
    printf '─%.0s' $(seq 1 $((width - 2)))
    printf '┤\n'
    printf '│  Last refresh: %s%-*s│\n' "$(date '+%Y-%m-%d %H:%M:%S')" $((width - 25)) ""
    printf '└'
    printf '─%.0s' $(seq 1 $((width - 2)))
    printf '┘\n'
}

harness::dashboard::run() {
    # Hide cursor
    printf '\033[?25l'
    
    # Trap to restore cursor on exit
    trap 'printf "\033[?25h"; exit 0' INT TERM
    
    while true; do
        harness::dashboard::render 80
        
        # Non-blocking read with timeout
        if read -t "$HARNESS_DASHBOARD_REFRESH" -n 1 key 2>/dev/null; then
            case "$key" in
                q|Q) break ;;
                r|R) continue ;;  # Immediate refresh
            esac
        fi
    done
    
    # Restore cursor
    printf '\033[?25h'
}

# ---------------------------------------------------------------------------
# Self-Tuning System
#
# Analyzes metrics and suggests/applies optimizations automatically.
# Can run in suggest-only or auto-apply mode.
# ---------------------------------------------------------------------------

HARNESS_AUTOTUNE_ENABLED="${HARNESS_AUTOTUNE_ENABLED:-0}"
HARNESS_AUTOTUNE_INTERVAL="${HARNESS_AUTOTUNE_INTERVAL:-300}"  # 5 minutes

# Analyze current performance and generate recommendations
harness::autotune::analyze() {
    local recommendations=()
    
    # Get metrics
    local hits misses l1_hits l2_hits
    hits="$(harness::metrics::get cache_hits)"
    misses="$(harness::metrics::get cache_misses)"
    l1_hits="$(harness::metrics::get l1_hits)"
    l2_hits="$(harness::metrics::get l2_hits)"
    local total=$((hits + misses))
    
    local hit_rate=0
    [[ $total -gt 0 ]] && hit_rate=$((hits * 100 / total))
    
    local l1_rate=0
    [[ $hits -gt 0 ]] && l1_rate=$((l1_hits * 100 / hits))
    
    local lock_timeouts lock_waits
    lock_timeouts="$(harness::metrics::get lock_timeouts)"
    lock_waits="$(harness::metrics::get lock_waits)"
    
    local queue_timeouts
    queue_timeouts="$(harness::metrics::get queue_timeouts)"
    
    local conflicts
    conflicts="$(harness::metrics::get intent_conflicts)"
    
    # Analysis 1: Low hit rate
    if [[ $total -gt 100 && $hit_rate -lt 50 ]]; then
        recommendations+=("LOW_HIT_RATE:Hit rate is ${hit_rate}% (below 50%). Consider increasing TTL values in rules.conf")
    fi
    
    # Analysis 2: L1 cache underutilized
    if [[ $hits -gt 100 && $l1_rate -lt 30 ]]; then
        recommendations+=("L1_UNDERUSED:Only ${l1_rate}% of hits from L1. Consider increasing HARNESS_L1_MAX_SIZE or HARNESS_L1_TTL")
    fi
    
    # Analysis 3: High lock contention
    if [[ $lock_waits -gt 50 ]]; then
        local contention_rate=$((lock_waits * 100 / total))
        if [[ $contention_rate -gt 20 ]]; then
            recommendations+=("HIGH_CONTENTION:Lock contention at ${contention_rate}%. Consider using 'queue' strategy with higher max_concurrent")
        fi
    fi
    
    # Analysis 4: Lock timeouts
    if [[ $lock_timeouts -gt 10 ]]; then
        recommendations+=("LOCK_TIMEOUTS:${lock_timeouts} lock timeouts detected. Consider increasing HARNESS_LOCK_TIMEOUT")
    fi
    
    # Analysis 5: Queue timeouts
    if [[ $queue_timeouts -gt 5 ]]; then
        recommendations+=("QUEUE_TIMEOUTS:${queue_timeouts} queue timeouts. Consider increasing max_concurrent for bottleneck commands")
    fi
    
    # Analysis 6: Intent conflicts
    if [[ $conflicts -gt 10 ]]; then
        recommendations+=("INTENT_CONFLICTS:${conflicts} intent conflicts. Agents may need better coordination or longer debounce_ms")
    fi
    
    # Analysis 7: L1 cache size
    local l1_size l1_max
    l1_size="$(harness::cache::l1_size 2>/dev/null || echo 0)"
    l1_max="${HARNESS_L1_MAX_SIZE:-104857600}"
    local l1_usage=$((l1_size * 100 / l1_max))
    if [[ $l1_usage -gt 90 ]]; then
        recommendations+=("L1_FULL:L1 cache at ${l1_usage}% capacity. Consider increasing HARNESS_L1_MAX_SIZE")
    fi
    
    # Analysis 8: L1 evictions
    local l1_evictions
    l1_evictions="$(harness::metrics::get l1_evictions)"
    if [[ $l1_evictions -gt 100 ]]; then
        recommendations+=("L1_EVICTIONS:${l1_evictions} L1 evictions. Cache thrashing may be occurring. Increase L1 size or reduce entry sizes")
    fi
    
    # Return recommendations
    printf '%s\n' "${recommendations[@]}"
}

# Display recommendations
harness::autotune::report() {
    echo "╔══════════════════════════════════════════════════════════════╗"
    echo "║              agent-harness Auto-Tune Report                  ║"
    echo "╚══════════════════════════════════════════════════════════════╝"
    echo ""
    
    local rec_count=0
    while IFS=: read -r code message; do
        [[ -z "$code" ]] && continue
        ((rec_count++))
        
        # Color-code by severity
        case "$code" in
            LOW_HIT_RATE|HIGH_CONTENTION|LOCK_TIMEOUTS)
                printf '\033[33m⚠ %s\033[0m\n' "$message"
                ;;
            L1_FULL|QUEUE_TIMEOUTS|INTENT_CONFLICTS)
                printf '\033[31m✗ %s\033[0m\n' "$message"
                ;;
            *)
                printf '  %s\n' "$message"
                ;;
        esac
        echo ""
    done < <(harness::autotune::analyze)
    
    if [[ $rec_count -eq 0 ]]; then
        echo "✓ No issues detected. System is performing well."
        echo ""
    fi
    
    # Show current config
    echo "── Current Configuration ──"
    echo "  HARNESS_LOCK_TIMEOUT=${HARNESS_LOCK_TIMEOUT:-30}"
    echo "  HARNESS_L1_MAX_SIZE=${HARNESS_L1_MAX_SIZE:-104857600}"
    echo "  HARNESS_L1_TTL=${HARNESS_L1_TTL:-60}"
    echo "  HARNESS_COMPRESS_THRESHOLD=${HARNESS_COMPRESS_THRESHOLD:-10240}"
    echo "  HARNESS_CACHE_HIERARCHY=${HARNESS_CACHE_HIERARCHY:-1}"
    echo "  HARNESS_IONICE_ENABLED=${HARNESS_IONICE_ENABLED:-1}"
}

# Apply automatic fixes where safe
harness::autotune::apply() {
    local applied=0
    
    while IFS=: read -r code message; do
        [[ -z "$code" ]] && continue
        
        case "$code" in
            L1_FULL)
                # Safe to increase L1 size
                local new_size=$((HARNESS_L1_MAX_SIZE * 2))
                echo "AUTO-FIX: Increasing L1 cache size to $((new_size / 1048576))MB"
                echo "export HARNESS_L1_MAX_SIZE=$new_size" >> "${HARNESS_HOME}/etc/autotune.env"
                ((applied++))
                ;;
            LOCK_TIMEOUTS)
                # Safe to increase lock timeout
                local new_timeout=$((HARNESS_LOCK_TIMEOUT * 2))
                [[ $new_timeout -gt 120 ]] && new_timeout=120
                echo "AUTO-FIX: Increasing lock timeout to ${new_timeout}s"
                echo "export HARNESS_LOCK_TIMEOUT=$new_timeout" >> "${HARNESS_HOME}/etc/autotune.env"
                ((applied++))
                ;;
        esac
    done < <(harness::autotune::analyze)
    
    if [[ $applied -gt 0 ]]; then
        echo ""
        echo "Applied $applied automatic fixes. Source etc/autotune.env to activate:"
        echo "  source ${HARNESS_HOME}/etc/autotune.env"
    else
        echo "No automatic fixes applied. Review recommendations manually."
    fi
}

# Generate optimized rules.conf based on observed patterns
harness::autotune::generate_rules() {
    echo "# Auto-generated rules.conf optimizations"
    echo "# Generated: $(date)"
    echo "# Based on analysis of current metrics"
    echo ""
    
    # Analyze which commands have high miss rates
    # (This would require per-command tracking, simplified here)
    
    echo "# Recommendations based on current workload:"
    echo ""
    
    local hit_rate
    local hits misses
    hits="$(harness::metrics::get cache_hits)"
    misses="$(harness::metrics::get cache_misses)"
    local total=$((hits + misses))
    [[ $total -gt 0 ]] && hit_rate=$((hits * 100 / total))
    
    if [[ ${hit_rate:-0} -lt 50 ]]; then
        echo "# Low hit rate detected (${hit_rate}%) — suggest longer TTLs:"
        echo "# ruff:check          coalesce    ttl=30  stale=10"
        echo "# mypy                coalesce    ttl=60  stale=20"
    fi
    
    local lock_waits
    lock_waits="$(harness::metrics::get lock_waits)"
    if [[ $lock_waits -gt 50 ]]; then
        echo ""
        echo "# High lock contention detected — suggest queue strategy:"
        echo "# git:status          queue       max_concurrent=2"
    fi
}

# Benchmark current configuration
harness::autotune::benchmark() {
    echo "Running performance benchmark..."
    echo ""
    
    # Store initial metrics
    local start_hits start_misses
    start_hits="$(harness::metrics::get cache_hits)"
    start_misses="$(harness::metrics::get cache_misses)"
    
    # Run a series of test operations
    local test_count=10
    local start_time end_time
    start_time="$(date +%s%N)"
    
    for ((i=0; i<test_count; i++)); do
        # Simulate cache operations
        harness::cache::l1_check "benchmark_test_$i" 60 >/dev/null 2>&1
    done
    
    end_time="$(date +%s%N)"
    local duration_ns=$((end_time - start_time))
    local duration_ms=$((duration_ns / 1000000))
    local avg_latency=$((duration_ms / test_count))
    
    echo "── Benchmark Results ──"
    echo "  Operations:     $test_count"
    echo "  Total time:     ${duration_ms}ms"
    echo "  Avg latency:    ${avg_latency}ms/op"
    echo ""
    
    # Ensure directories exist
    mkdir -p "$HARNESS_L1_CACHE" "$HARNESS_L2_CACHE" 2>/dev/null
    
    # L1 cache speed test
    echo "── L1 Cache Test ──"
    local l1_start l1_end l1_duration
    l1_start="$(date +%s%N)"
    for ((i=0; i<100; i++)); do
        echo "test" > "${HARNESS_L1_CACHE}/benchmark_${i}.tmp"
        cat "${HARNESS_L1_CACHE}/benchmark_${i}.tmp" >/dev/null
        rm -f "${HARNESS_L1_CACHE}/benchmark_${i}.tmp"
    done
    l1_end="$(date +%s%N)"
    l1_duration=$(( (l1_end - l1_start) / 1000000 ))
    echo "  100 write/read/delete cycles: ${l1_duration}ms"
    echo "  Avg per cycle: $((l1_duration / 100))ms"
    echo ""
    
    # L2 cache speed test
    echo "── L2 Cache Test ──"
    local l2_start l2_end l2_duration
    l2_start="$(date +%s%N)"
    for ((i=0; i<100; i++)); do
        echo "test" > "${HARNESS_L2_CACHE}/benchmark_${i}.tmp"
        cat "${HARNESS_L2_CACHE}/benchmark_${i}.tmp" >/dev/null
        rm -f "${HARNESS_L2_CACHE}/benchmark_${i}.tmp"
    done
    l2_end="$(date +%s%N)"
    l2_duration=$(( (l2_end - l2_start) / 1000000 ))
    echo "  100 write/read/delete cycles: ${l2_duration}ms"
    echo "  Avg per cycle: $((l2_duration / 100))ms"
    echo ""
    
    # Speedup ratio
    if [[ $l2_duration -gt 0 && $l1_duration -gt 0 ]]; then
        local speedup=$((l2_duration * 100 / l1_duration))
        echo "  L1 vs L2 speedup: ${speedup}% faster"
    fi
}

# ---------------------------------------------------------------------------
# Priority Queue System
#
# Implements fair scheduling with priority levels. Higher priority commands
# get executed first, but lower priority commands won't starve thanks to
# aging (priority boost based on wait time).
# ---------------------------------------------------------------------------
harness::queue::init() {
    mkdir -p "$HARNESS_QUEUE_DIR" 2>/dev/null
}

# Get the detected agent name from process tree
harness::get_agent_name() {
    local agents_file="${HARNESS_ETC}/agents.conf"
    [[ -f "$agents_file" ]] || { echo "unknown"; return; }

    local pid=$PPID
    local hops=0
    local max_hops=32

    while [[ $pid -gt 1 && $hops -lt $max_hops ]]; do
        local comm cmdline
        comm="$(harness::get_comm "$pid" 2>/dev/null)" || break
        cmdline="$(harness::get_cmdline "$pid" 2>/dev/null)" || cmdline=""

        while IFS= read -r pattern; do
            pattern="${pattern%%#*}"
            pattern="${pattern// /}"
            [[ -z "$pattern" ]] && continue
            if [[ "$comm" == *"$pattern"* ]] || [[ "$cmdline" == *"$pattern"* ]]; then
                echo "$pattern"
                return
            fi
        done < "$agents_file"

        local new_pid
        new_pid="$(harness::get_ppid "$pid" 2>/dev/null)" || break
        [[ "$new_pid" == "$pid" || -z "$new_pid" ]] && break
        pid="$new_pid"
        ((hops++))
    done
    echo "unknown"
}

# Track agent resource usage for fair scheduling
harness::agent::track_usage() {
    local agent="$1"
    local cpu_ms="${2:-0}"
    
    mkdir -p "$(dirname "$HARNESS_AGENT_STATS")" 2>/dev/null
    local lock="${HARNESS_VAR}/agent_stats.lock"
    exec 251>"$lock"
    flock -x 251 2>/dev/null || return
    
    local now
    now="$(date +%s)"
    local window=60  # Track usage over 60 second window
    
    # Read current stats, filter out old entries, add new entry
    local temp_file="${HARNESS_AGENT_STATS}.tmp"
    {
        # Keep entries from last 60 seconds
        if [[ -f "$HARNESS_AGENT_STATS" ]]; then
            awk -v now="$now" -v window="$window" '$1 > (now - window)' "$HARNESS_AGENT_STATS"
        fi
        # Add new entry
        echo "$now $agent $cpu_ms"
    } > "$temp_file"
    mv -f "$temp_file" "$HARNESS_AGENT_STATS"
    
    flock -u 251
}

# Get agent's recent usage (for fair scheduling)
harness::agent::get_usage() {
    local agent="$1"
    [[ -f "$HARNESS_AGENT_STATS" ]] || { echo "0"; return; }
    
    local now window=60
    now="$(date +%s)"
    
    awk -v now="$now" -v window="$window" -v agent="$agent" '
        $1 > (now - window) && $2 == agent { sum += $3 }
        END { print sum + 0 }
    ' "$HARNESS_AGENT_STATS"
}

# Calculate effective priority (base priority + aging + fairness adjustment + fair share)
harness::queue::effective_priority() {
    local base_priority="$1"
    local enqueue_time="$2"
    local agent="$3"
    
    local now
    now="$(date +%s)"
    local wait_time=$((now - enqueue_time))
    
    # Aging: reduce priority number (increase priority) by 1 for every 5 seconds waiting
    local aging_boost=$((wait_time / 5))
    
    # Short-term fairness: agents using more resources recently get lower priority
    local agent_usage
    agent_usage="$(harness::agent::get_usage "$agent")"
    local fairness_penalty=0
    if [[ "$agent_usage" -gt 10000 ]]; then
        fairness_penalty=2
    elif [[ "$agent_usage" -gt 5000 ]]; then
        fairness_penalty=1
    fi
    
    # Long-term fair share: adjust based on quota vs actual usage
    local fairshare_adj
    fairshare_adj="$(harness::fairshare::get_adjustment "$agent" 2>/dev/null || echo 0)"
    
    local effective=$((base_priority - aging_boost + fairness_penalty + fairshare_adj))
    # Clamp to valid range [0, 10]
    [[ $effective -lt 0 ]] && effective=0
    [[ $effective -gt 10 ]] && effective=10
    
    echo "$effective"
}

# Enqueue a command request
harness::queue::enqueue() {
    local cmd_name="$1"
    local priority="${2:-normal}"
    local agent="${3:-unknown}"
    
    harness::queue::init
    
    local priority_num="${HARNESS_PRIORITY_LEVELS[$priority]:-2}"
    local now
    now="$(date +%s)"
    local ticket="${now}.$$"
    local entry_file="${HARNESS_QUEUE_DIR}/${cmd_name}.${ticket}"
    
    echo "${priority_num} ${now} ${agent} $$" > "$entry_file"
    echo "$ticket"
}

# Dequeue: get the highest priority waiting request
harness::queue::dequeue() {
    local cmd_name="$1"
    
    harness::queue::init
    
    local best_ticket=""
    local best_priority=999
    local best_file=""
    
    for entry in "${HARNESS_QUEUE_DIR}/${cmd_name}."*; do
        [[ -f "$entry" ]] || continue
        
        local priority enqueue_time agent pid
        read -r priority enqueue_time agent pid < "$entry" 2>/dev/null || continue
        
        # Check if the process is still alive
        if ! kill -0 "$pid" 2>/dev/null; then
            rm -f "$entry"
            continue
        fi
        
        local effective
        effective="$(harness::queue::effective_priority "$priority" "$enqueue_time" "$agent")"
        
        if [[ "$effective" -lt "$best_priority" ]]; then
            best_priority="$effective"
            best_ticket="$(basename "$entry" | sed "s/^${cmd_name}\.//")"
            best_file="$entry"
        fi
    done
    
    if [[ -n "$best_file" ]]; then
        rm -f "$best_file"
        echo "$best_ticket"
    fi
}

# Check if this ticket is next in queue
harness::queue::is_next() {
    local cmd_name="$1"
    local my_ticket="$2"
    
    local next_ticket
    next_ticket="$(harness::queue::peek "$cmd_name")"
    
    [[ "$next_ticket" == "$my_ticket" ]]
}

# Peek at the next ticket without removing
harness::queue::peek() {
    local cmd_name="$1"
    
    local best_ticket=""
    local best_priority=999
    
    for entry in "${HARNESS_QUEUE_DIR}/${cmd_name}."*; do
        [[ -f "$entry" ]] || continue
        
        local priority enqueue_time agent pid
        read -r priority enqueue_time agent pid < "$entry" 2>/dev/null || continue
        
        # Check if the process is still alive
        if ! kill -0 "$pid" 2>/dev/null; then
            rm -f "$entry"
            continue
        fi
        
        local effective
        effective="$(harness::queue::effective_priority "$priority" "$enqueue_time" "$agent")"
        
        if [[ "$effective" -lt "$best_priority" ]]; then
            best_priority="$effective"
            best_ticket="$(basename "$entry" | sed "s/^${cmd_name}\.//")"
        fi
    done
    
    echo "$best_ticket"
}

# Remove a ticket from the queue
harness::queue::remove() {
    local cmd_name="$1"
    local ticket="$2"
    rm -f "${HARNESS_QUEUE_DIR}/${cmd_name}.${ticket}" 2>/dev/null
}

# ---------------------------------------------------------------------------
# Semantic Coalescing
#
# Recognizes semantically equivalent commands that can share cache entries.
# For example, "ruff check ." and "ruff check src/" might be equivalent
# if src/ is the only Python directory in the project.
# ---------------------------------------------------------------------------

# Normalize command arguments for semantic comparison
harness::semantic::normalize() {
    local cmd="$1"; shift
    local args=("$@")
    
    case "$cmd" in
        ruff|mypy|pylint|flake8)
            # Normalize path arguments
            local normalized_args=()
            for arg in "${args[@]}"; do
                if [[ "$arg" == "." ]]; then
                    # "." is equivalent to listing all relevant directories
                    normalized_args+=("__PROJECT_ROOT__")
                elif [[ -d "$arg" ]]; then
                    # Normalize to absolute path
                    normalized_args+=("$(cd "$arg" 2>/dev/null && pwd)")
                else
                    normalized_args+=("$arg")
                fi
            done
            echo "${normalized_args[*]}"
            ;;
        git)
            # Git commands are usually position-sensitive, minimal normalization
            echo "${args[*]}"
            ;;
        *)
            # Default: no normalization
            echo "${args[*]}"
            ;;
    esac
}

# Check if two commands are semantically equivalent
harness::semantic::equivalent() {
    local cmd1="$1"
    local args1="$2"
    local cmd2="$3"
    local args2="$4"
    
    [[ "$cmd1" != "$cmd2" ]] && return 1
    
    local norm1 norm2
    norm1="$(harness::semantic::normalize "$cmd1" $args1)"
    norm2="$(harness::semantic::normalize "$cmd2" $args2)"
    
    [[ "$norm1" == "$norm2" ]]
}

# Generate a semantic cache key (may be same for equivalent commands)
harness::semantic::cache_key() {
    local cmd="$1"; shift
    local mode="$1"; shift
    # remaining: command args
    
    local normalized
    normalized="$(harness::semantic::normalize "$cmd" "$@")"
    
    # Use normalized args for cache key
    local base
    base="$(printf '%s\0' "$cmd" $normalized)"
    
    case "$mode" in
        args)
            printf '%s' "$base" | sha256sum | cut -d' ' -f1
            ;;
        git)
            local git_state=""
            if command -v git &>/dev/null; then
                git_state="$(git -C "$PWD" status --porcelain 2>/dev/null || true)"
                git_state+="$(git -C "$PWD" rev-parse HEAD 2>/dev/null || true)"
            fi
            printf '%s\0%s\0%s' "$base" "$PWD" "$git_state" | sha256sum | cut -d' ' -f1
            ;;
        *)  # "time" or default
            printf '%s\0%s' "$base" "$PWD" | sha256sum | cut -d' ' -f1
            ;;
    esac
}

# ---------------------------------------------------------------------------
# Process-tree agent detection
#
# Walks the parent-process chain looking for known AI agent process names.
# Returns 0 if an agent ancestor is found, 1 otherwise.
# Works on Linux (/proc) with macOS (ps) fallback.
# ---------------------------------------------------------------------------
harness::get_ppid() {
    local pid="$1"
    if [[ -r "/proc/${pid}/stat" ]]; then
        awk '{print $4}' "/proc/${pid}/stat" 2>/dev/null
    else
        ps -o ppid= -p "$pid" 2>/dev/null | tr -d ' '
    fi
}

harness::get_comm() {
    local pid="$1"
    if [[ -r "/proc/${pid}/comm" ]]; then
        cat "/proc/${pid}/comm" 2>/dev/null
    else
        ps -o comm= -p "$pid" 2>/dev/null | xargs 2>/dev/null
    fi
}

harness::get_cmdline() {
    local pid="$1"
    if [[ -r "/proc/${pid}/cmdline" ]]; then
        tr '\0' ' ' < "/proc/${pid}/cmdline" 2>/dev/null
    else
        ps -o args= -p "$pid" 2>/dev/null
    fi
}

harness::is_agent() {
    local agents_file="${HARNESS_ETC}/agents.conf"
    [[ -f "$agents_file" ]] || return 1

    # Load patterns (skip comments and blanks)
    local -a patterns=()
    while IFS= read -r line; do
        line="${line%%#*}"          # strip inline comments
        line="${line// /}"          # strip spaces
        [[ -z "$line" ]] && continue
        patterns+=("$line")
    done < "$agents_file"
    [[ ${#patterns[@]} -eq 0 ]] && return 1

    local pid=$PPID
    local hops=0
    local max_hops=32              # safety limit

    while [[ $pid -gt 1 && $hops -lt $max_hops ]]; do
        local comm cmdline
        comm="$(harness::get_comm "$pid")" || break
        cmdline="$(harness::get_cmdline "$pid")" || cmdline=""

        for pattern in "${patterns[@]}"; do
            if [[ "$comm" == *"$pattern"* ]] || [[ "$cmdline" == *"$pattern"* ]]; then
                harness::log DEBUG "Agent detected: pattern='$pattern' comm='$comm' pid=$pid"
                return 0
            fi
        done

        local new_pid
        new_pid="$(harness::get_ppid "$pid")" || break
        [[ "$new_pid" == "$pid" || -z "$new_pid" ]] && break
        pid="$new_pid"
        ((hops++))
    done
    return 1
}

# ---------------------------------------------------------------------------
# Real binary resolution
#
# Scans PATH to find the actual binary, skipping our proxy directory and
# anything that resolves back to the harness dispatcher.
# ---------------------------------------------------------------------------
harness::find_real() {
    local cmd="$1"

    # First check the pre-cached .real file (written at install time)
    local cached_real="${HARNESS_PROXY}/.${cmd}.real"
    if [[ -f "$cached_real" ]]; then
        local cached_path
        cached_path="$(cat "$cached_real")"
        if [[ -x "$cached_path" ]]; then
            echo "$cached_path"
            return 0
        fi
    fi

    # Fallback: scan PATH, skipping proxy dir
    local harness_real
    harness_real="$(readlink -f "${HARNESS_HOME}/bin/harness" 2>/dev/null || echo "")"
    local proxy_real
    proxy_real="$(readlink -f "${HARNESS_PROXY}" 2>/dev/null || echo "")"

    local IFS=':'
    for dir in $PATH; do
        # Skip our proxy directory
        local dir_real
        dir_real="$(readlink -f "$dir" 2>/dev/null || echo "$dir")"
        [[ "$dir_real" == "$proxy_real" ]] && continue

        local candidate="${dir}/${cmd}"
        if [[ -x "$candidate" ]]; then
            # Ensure it doesn't resolve back to our harness
            local cand_real
            cand_real="$(readlink -f "$candidate" 2>/dev/null || echo "$candidate")"
            [[ -n "$harness_real" && "$cand_real" == "$harness_real" ]] && continue

            echo "$candidate"
            return 0
        fi
    done
    return 1
}

# ---------------------------------------------------------------------------
# Cache key generation
#
# Modes:
#   "args"  — hash(command + args)                     [CWD-independent]
#   "time"  — hash(command + args + CWD)               [default, TTL-based]
#   "git"   — hash(command + args + CWD + git status)  [content-aware]
# ---------------------------------------------------------------------------
harness::cache_key() {
    local mode="$1"; shift
    local cmd="$1"; shift
    # rest of args: "$@"

    local base
    base="$(printf '%s\0' "$cmd" "$@")"

    case "$mode" in
        args)
            printf '%s' "$base" | sha256sum | cut -d' ' -f1
            ;;
        git)
            local git_state=""
            if command -v git &>/dev/null; then
                git_state="$(git -C "$PWD" status --porcelain 2>/dev/null || true)"
                git_state+="$(git -C "$PWD" rev-parse HEAD 2>/dev/null || true)"
            fi
            printf '%s\0%s\0%s' "$base" "$PWD" "$git_state" | sha256sum | cut -d' ' -f1
            ;;
        *)  # "time" or default
            printf '%s\0%s' "$base" "$PWD" | sha256sum | cut -d' ' -f1
            ;;
    esac
}

# ---------------------------------------------------------------------------
# Rule parser
#
# Reads rules.conf and returns the matching rule string for a given
# command:subcommand pair.  Supports exact match and wildcard (*).
#
# Format:  COMMAND[:SUBCOMMAND]  STRATEGY  [key=value ...]
# Returns: "STRATEGY key=value ..." or "passthrough" if no match.
# ---------------------------------------------------------------------------
harness::get_rule() {
    local cmd="$1"
    local subcmd="${2:-}"
    local rules_file="${HARNESS_ETC}/rules.conf"
    [[ -f "$rules_file" ]] || { echo "passthrough"; return; }

    local best_match=""

    while IFS= read -r line; do
        line="${line%%#*}"       # strip comments
        [[ -z "${line// /}" ]] && continue

        local pattern rest
        pattern="$(echo "$line" | awk '{print $1}')"
        rest="$(echo "$line" | cut -d' ' -f2-)"

        local pcmd psub
        if [[ "$pattern" == *:* ]]; then
            pcmd="${pattern%%:*}"
            psub="${pattern#*:}"
        else
            pcmd="$pattern"
            psub="*"
        fi

        if [[ "$pcmd" == "$cmd" ]]; then
            if [[ "$psub" == "$subcmd" ]]; then
                # Exact match — return immediately
                echo "$rest"
                return
            elif [[ "$psub" == "*" ]]; then
                # Wildcard match — keep as fallback
                best_match="$rest"
            fi
        fi
    done < "$rules_file"

    echo "${best_match:-passthrough}"
}

# ---------------------------------------------------------------------------
# Option parser helper
# Extracts key=value from a rule options string.
# ---------------------------------------------------------------------------
harness::opt() {
    local opts="$1"
    local key="$2"
    local default="$3"
    local val
    val="$(echo "$opts" | grep -oP "${key}=\K[^ ]+" 2>/dev/null || true)"
    echo "${val:-$default}"
}

# ---------------------------------------------------------------------------
# Platform-aware file age (seconds since modification)
# ---------------------------------------------------------------------------
harness::file_age() {
    local file="$1"
    local now mtime
    now="$(date +%s)"
    if stat -c %Y "$file" &>/dev/null; then
        mtime="$(stat -c %Y "$file")"       # GNU/Linux
    elif stat -f %m "$file" &>/dev/null; then
        mtime="$(stat -f %m "$file")"       # macOS/BSD
    else
        echo 999999
        return
    fi
    echo $(( now - mtime ))
}

# =============================================================================
# STRATEGIES
# =============================================================================

# ---------------------------------------------------------------------------
# Coalesce (single-flight with caching)
#
# Only one process executes the command at a time.  Concurrent arrivals
# block on an exclusive lock and receive the cached result once the
# executor finishes.  Optional debounce adds a short delay to batch
# near-simultaneous invocations.
#
# Features:
#   - Hierarchical cache: L1 (memory) + L2 (disk) for optimal performance
#   - Lock timeout: Prevents infinite waits (HARNESS_LOCK_TIMEOUT)
#   - Stale-while-revalidate: Serves stale cache while refreshing in background
#   - Compression: zstd compression for large outputs (HARNESS_COMPRESS_THRESHOLD)
#   - I/O priority: Uses ionice for disk I/O scheduling
#   - Metrics: Tracks hits, misses, timeouts, etc.
#
# Args: real_cmd cache_key ttl debounce_ms error_ttl [cmd_args...]
# ---------------------------------------------------------------------------
harness::strategy::coalesce() {
    local real_cmd="$1"; shift
    local cache_key="$1"; shift
    local ttl="$1"; shift
    local debounce_ms="$1"; shift
    local error_ttl="$1"; shift
    # remaining: "$@" = arguments to pass to real command

    # Initialize cache directories
    harness::cache::init
    mkdir -p "$HARNESS_LOCKS" 2>/dev/null

    # --- L1 Cache Check (no lock needed for read) ---
    if [[ "$HARNESS_CACHE_HIERARCHY" == "1" ]]; then
        if harness::cache::l1_check "$cache_key" "$ttl"; then
            local l1_out l1_err l1_rc
            if harness::cache::l1_read "$cache_key" l1_out l1_err l1_rc; then
                harness::log INFO "L1 HIT key=${cache_key:0:12}…"
                harness::metrics::incr cache_hits
                harness::metrics::incr l1_hits
                echo "$l1_out"
                echo "$l1_err" >&2
                return "$l1_rc"
            fi
        fi
    fi

    local lock="${HARNESS_LOCKS}/${cache_key}.lock"
    local out="${HARNESS_L2_CACHE}/${cache_key}.out"
    local err="${HARNESS_L2_CACHE}/${cache_key}.err"
    local rc="${HARNESS_L2_CACHE}/${cache_key}.rc"

    # --- Acquire lock with timeout ---
    exec 200>"$lock"
    local lock_start lock_waited=0
    lock_start="$(date +%s)"
    
    if ! flock -x -w "${HARNESS_LOCK_TIMEOUT}" 200 2>/dev/null; then
        harness::log WARN "LOCK TIMEOUT key=${cache_key:0:12}… timeout=${HARNESS_LOCK_TIMEOUT}s"
        harness::metrics::incr lock_timeouts
        # Fallback: execute without caching
        "$real_cmd" "$@"
        return $?
    fi
    
    local lock_end
    lock_end="$(date +%s)"
    lock_waited=$((lock_end - lock_start))
    [[ $lock_waited -gt 0 ]] && harness::metrics::incr lock_waits

    # --- Helper to read potentially compressed cache ---
    _read_cache() {
        local file="$1"
        if [[ -f "${file}.zst" ]]; then
            zstd -dqc "${file}.zst" 2>/dev/null
        elif [[ -f "$file" ]]; then
            cat "$file" 2>/dev/null
        fi
    }

    # --- Check L2 cache freshness ---
    local cache_exists=0
    [[ -f "$rc" || -f "${rc}.zst" ]] && cache_exists=1
    
    if [[ $cache_exists -eq 1 ]]; then
        local age effective_ttl cached_code stale_threshold
        age="$(harness::file_age "$rc" 2>/dev/null || harness::file_age "${rc}.zst" 2>/dev/null || echo 999999)"
        cached_code="$(_read_cache "$rc" || echo 255)"
        stale_threshold="${HARNESS_STALE_THRESHOLD:-0}"

        # Use shorter TTL for errors if configured
        if [[ "$cached_code" -ne 0 && "$error_ttl" -gt 0 ]]; then
            effective_ttl="$error_ttl"
        else
            effective_ttl="$ttl"
        fi

        if [[ "$age" -lt "$effective_ttl" ]]; then
            # --- Fresh L2 cache hit ---
            harness::log INFO "L2 HIT key=${cache_key:0:12}… age=${age}s ttl=${effective_ttl}s"
            harness::metrics::incr cache_hits
            harness::metrics::incr l2_hits
            
            # Promote to L1 for faster subsequent access
            if [[ "$HARNESS_CACHE_HIERARCHY" == "1" ]]; then
                harness::cache::promote_to_l1 "$cache_key" &
            fi
            
            # Check if we should trigger background refresh (stale-while-revalidate)
            if [[ "$stale_threshold" -gt 0 && "$age" -gt "$stale_threshold" ]]; then
                harness::log DEBUG "SWR: age=${age}s > stale=${stale_threshold}s — triggering background refresh"
                harness::_background_refresh "$cache_key" "$real_cmd" "$@" &
                harness::metrics::incr cache_stale_hits
            fi
            
            _read_cache "$out"
            _read_cache "$err" >&2
            flock -u 200
            return "$cached_code"
        fi
    fi

    # --- Cache miss ---
    harness::metrics::incr cache_misses

    # --- Optional debounce ---
    if [[ "$debounce_ms" -gt 0 ]]; then
        local sleep_s
        sleep_s="$(awk "BEGIN{printf \"%.3f\", ${debounce_ms}/1000}")"
        harness::log DEBUG "Debouncing ${sleep_s}s before execution"
        sleep "$sleep_s" 2>/dev/null || sleep 1
    fi

    # --- Pre-exec: warm file read cache if available ---
    if declare -f rc::pre_exec &>/dev/null; then
        local _rc_level="${HARNESS_READCACHE_LEVEL:-1}"
        local _rc_cmd _rc_subcmd
        _rc_cmd="$(basename "$real_cmd")"
        _rc_subcmd="${1:-}"
        rc::pre_exec "${PWD}" "$_rc_cmd" "$_rc_subcmd" "$_rc_level" 2>/dev/null
    fi

    # --- Set I/O priority for command execution ---
    local cmd_priority="${HARNESS_CMD_PRIORITIES[$(basename "$real_cmd")]:-normal}"
    harness::io::set_priority "$cmd_priority"

    # --- Execute the real command ---
    harness::log INFO "EXEC key=${cache_key:0:12}… cmd=$(basename "$real_cmd") $* io_priority=${cmd_priority}"
    harness::metrics::incr commands_executed

    "$real_cmd" "$@" >"${out}.tmp" 2>"${err}.tmp"
    local code=$?
    echo "$code" > "${rc}.tmp"

    # --- Read output for L1 cache and display ---
    local out_content err_content
    out_content="$(cat "${out}.tmp" 2>/dev/null)"
    err_content="$(cat "${err}.tmp" 2>/dev/null)"

    # --- Write to L1 cache (fast, in-memory) ---
    if [[ "$HARNESS_CACHE_HIERARCHY" == "1" ]]; then
        harness::cache::l1_write "$cache_key" "$out_content" "$err_content" "$code"
    fi

    # --- Compress and write to L2 if output exceeds threshold ---
    local out_size
    out_size="$(stat -c%s "${out}.tmp" 2>/dev/null || stat -f%z "${out}.tmp" 2>/dev/null || echo 0)"
    
    if [[ "$out_size" -gt "${HARNESS_COMPRESS_THRESHOLD}" ]] && command -v zstd &>/dev/null; then
        harness::log DEBUG "Compressing L2 output (${out_size} bytes)"
        zstd -q -f "${out}.tmp" -o "${out}.zst.tmp" && rm -f "${out}.tmp"
        zstd -q -f "${err}.tmp" -o "${err}.zst.tmp" && rm -f "${err}.tmp"
        zstd -q -f "${rc}.tmp" -o "${rc}.zst.tmp" && rm -f "${rc}.tmp"
        
        # Atomic rename (compressed)
        mv -f "${out}.zst.tmp" "${out}.zst" 2>/dev/null
        mv -f "${err}.zst.tmp" "${err}.zst" 2>/dev/null
        mv -f "${rc}.zst.tmp" "${rc}.zst" 2>/dev/null
        rm -f "$out" "$err" "$rc" 2>/dev/null  # Remove uncompressed versions
    else
        # Atomic rename (uncompressed)
        mv -f "${out}.tmp" "$out"
        mv -f "${err}.tmp" "$err"
        mv -f "${rc}.tmp" "$rc"
        rm -f "${out}.zst" "${err}.zst" "${rc}.zst" 2>/dev/null  # Remove compressed versions
    fi

    # --- Output (already captured in out_content/err_content) ---
    echo "$out_content"
    echo "$err_content" >&2

    flock -u 200
    return "$code"
}

# ---------------------------------------------------------------------------
# Background refresh (stale-while-revalidate helper)
#
# Runs in background to update cache while stale result is served.
# Uses non-blocking lock to avoid piling up refreshes.
# ---------------------------------------------------------------------------
harness::_background_refresh() {
    local cache_key="$1"; shift
    local real_cmd="$1"; shift
    
    local refresh_lock="${HARNESS_LOCKS}/${cache_key}.refresh"
    local out="${HARNESS_CACHE}/${cache_key}.out"
    local err="${HARNESS_CACHE}/${cache_key}.err"
    local rc="${HARNESS_CACHE}/${cache_key}.rc"
    
    # Non-blocking lock — if another refresh is running, exit silently
    exec 201>"$refresh_lock"
    flock -n 201 || return 0
    
    harness::log DEBUG "BG REFRESH starting key=${cache_key:0:12}…"
    harness::metrics::incr refresh_triggered
    
    # Execute and update cache
    "$real_cmd" "$@" > "${out}.new" 2> "${err}.new"
    echo $? > "${rc}.new"
    
    # Check if compression is warranted
    local out_size
    out_size="$(stat -c%s "${out}.new" 2>/dev/null || stat -f%z "${out}.new" 2>/dev/null || echo 0)"
    
    if [[ "$out_size" -gt "${HARNESS_COMPRESS_THRESHOLD}" ]] && command -v zstd &>/dev/null; then
        zstd -q -f "${out}.new" -o "${out}.zst" && rm -f "${out}.new" "$out"
        zstd -q -f "${err}.new" -o "${err}.zst" && rm -f "${err}.new" "$err"
        zstd -q -f "${rc}.new" -o "${rc}.zst" && rm -f "${rc}.new" "$rc"
    else
        mv -f "${out}.new" "$out"
        mv -f "${err}.new" "$err"
        mv -f "${rc}.new" "$rc"
        rm -f "${out}.zst" "${err}.zst" "${rc}.zst" 2>/dev/null
    fi
    
    harness::log DEBUG "BG REFRESH complete key=${cache_key:0:12}…"
    flock -u 201
}

# ---------------------------------------------------------------------------
# Queue (concurrency limiter with priority support)
#
# Allows up to N concurrent executions of the same command.  Additional
# callers block until a slot opens.  No caching — every caller runs the
# real command, just with bounded parallelism.
#
# When multiple processes are waiting, priority determines order:
#   - Higher priority commands get slots first
#   - Aging prevents starvation (waiting boosts priority)
#   - Fair scheduling penalizes agents using excessive resources
#
# Args: real_cmd cmd_name max_concurrent priority [cmd_args...]
# ---------------------------------------------------------------------------
harness::strategy::queue() {
    local real_cmd="$1"; shift
    local cmd_name="$1"; shift
    local max_concurrent="$1"; shift
    local priority="${1:-normal}"; shift

    mkdir -p "$HARNESS_LOCKS" "$HARNESS_QUEUE_DIR" 2>/dev/null
    
    # Detect calling agent for fair scheduling
    local agent
    agent="$(harness::get_agent_name)"
    
    # Enqueue ourselves
    local my_ticket
    my_ticket="$(harness::queue::enqueue "$cmd_name" "$priority" "$agent")"
    harness::log DEBUG "QUEUE enqueued ticket=${my_ticket} priority=${priority} agent=${agent}"
    
    # Track start time for usage accounting
    local start_time
    start_time="$(date +%s%3N 2>/dev/null || date +%s)000"

    local slot acquired=0
    local wait_iterations=0
    local max_wait_iterations=$((HARNESS_LOCK_TIMEOUT * 10))  # Check every 100ms
    
    while [[ $acquired -eq 0 && $wait_iterations -lt $max_wait_iterations ]]; do
        # Try to acquire a slot
        for (( slot = 0; slot < max_concurrent; slot++ )); do
            local lock_file="${HARNESS_LOCKS}/${cmd_name}.slot${slot}.lock"

            # Try to grab this slot non-blocking
            eval "exec $((210 + slot))>\"$lock_file\""
            if flock -n "$((210 + slot))" 2>/dev/null; then
                # Got a slot - but check if we should yield to higher priority
                local next_ticket
                next_ticket="$(harness::queue::peek "$cmd_name")"
                
                if [[ "$next_ticket" == "$my_ticket" || -z "$next_ticket" ]]; then
                    # We're highest priority or only one waiting
                    acquired=1
                    harness::queue::remove "$cmd_name" "$my_ticket"
                    harness::log INFO "QUEUE slot=${slot}/${max_concurrent} cmd=$cmd_name priority=${priority} agent=${agent}"
                    harness::metrics::incr queue_acquired
                    
                    "$real_cmd" "$@"
                    local code=$?
                    
                    # Track resource usage (short-term)
                    local end_time
                    end_time="$(date +%s%3N 2>/dev/null || date +%s)000"
                    local duration_ms=$((end_time - start_time))
                    harness::agent::track_usage "$agent" "$duration_ms"
                    
                    # Record fair share usage (long-term)
                    harness::fairshare::record_usage "$agent" "$duration_ms"
                    
                    flock -u "$((210 + slot))"
                    return "$code"
                else
                    # Higher priority request waiting - yield the slot
                    flock -u "$((210 + slot))"
                    harness::log DEBUG "QUEUE yielding slot=${slot} to higher priority ticket=${next_ticket}"
                fi
            fi
        done
        
        # No slots available or yielded, wait a bit
        sleep 0.1
        ((wait_iterations++))
        
        # Periodically log wait status
        if [[ $((wait_iterations % 50)) -eq 0 ]]; then
            harness::log DEBUG "QUEUE waiting iterations=${wait_iterations} cmd=$cmd_name ticket=${my_ticket}"
        fi
    done
    
    # Timeout or all attempts failed
    if [[ $acquired -eq 0 ]]; then
        harness::queue::remove "$cmd_name" "$my_ticket"
        
        if [[ $wait_iterations -ge $max_wait_iterations ]]; then
            harness::log WARN "QUEUE TIMEOUT cmd=$cmd_name after ${HARNESS_LOCK_TIMEOUT}s"
            harness::metrics::incr queue_timeouts
        fi
        
        # Fallback: execute without queue protection
        harness::log WARN "QUEUE FALLBACK executing without slot cmd=$cmd_name"
        "$real_cmd" "$@"
        return $?
    fi
}

# ---------------------------------------------------------------------------
# Priority Queue Strategy (explicit priority specification)
#
# Like queue, but allows explicit priority specification in rules.
# Useful for ensuring critical commands (git operations) run first.
#
# Args: real_cmd cmd_name max_concurrent priority [cmd_args...]
# ---------------------------------------------------------------------------
harness::strategy::priority_queue() {
    # Just delegates to queue with explicit priority
    harness::strategy::queue "$@"
}

# ---------------------------------------------------------------------------
# Debounce (delay without caching)
#
# Adds a short delay before execution.  Useful for mutating commands
# where caching is unsafe but you still want to absorb rapid-fire calls.
#
# Args: real_cmd debounce_ms [cmd_args...]
# ---------------------------------------------------------------------------
harness::strategy::debounce() {
    local real_cmd="$1"; shift
    local debounce_ms="$1"; shift

    if [[ "$debounce_ms" -gt 0 ]]; then
        local sleep_s
        sleep_s="$(awk "BEGIN{printf \"%.3f\", ${debounce_ms}/1000}")"
        sleep "$sleep_s" 2>/dev/null || sleep 1
    fi

    exec "$real_cmd" "$@"
}

# ---------------------------------------------------------------------------
# Cache stats (for `harness status` and `harness metrics`)
# ---------------------------------------------------------------------------
harness::cache_stats() {
    # L2 (disk) stats
    local l2_total l2_compressed
    l2_total="$(find "$HARNESS_L2_CACHE" -name '*.rc' -o -name '*.rc.zst' 2>/dev/null | wc -l)"
    l2_compressed="$(find "$HARNESS_L2_CACHE" -name '*.zst' 2>/dev/null | wc -l)"
    local l2_fresh=0
    while IFS= read -r rcfile; do
        local age
        age="$(harness::file_age "$rcfile")"
        [[ "$age" -lt 30 ]] && ((l2_fresh++))
    done < <(find "$HARNESS_L2_CACHE" -name '*.rc' -o -name '*.rc.zst' 2>/dev/null)
    
    # L1 (memory) stats
    local l1_total l1_size_kb
    l1_total="$(find "$HARNESS_L1_CACHE" -name '*.rc' 2>/dev/null | wc -l)"
    l1_size_kb="$(( $(harness::cache::l1_size 2>/dev/null || echo 0) / 1024 ))"
    
    # Overall hit rate
    local hits misses hit_rate l1_hits l2_hits
    hits="$(harness::metrics::get cache_hits)"
    misses="$(harness::metrics::get cache_misses)"
    l1_hits="$(harness::metrics::get l1_hits)"
    l2_hits="$(harness::metrics::get l2_hits)"
    if [[ $((hits + misses)) -gt 0 ]]; then
        hit_rate="$(awk "BEGIN{printf \"%.1f\", 100*${hits}/(${hits}+${misses})}")"
    else
        hit_rate="0.0"
    fi
    
    echo "L1[entries=${l1_total},size=${l1_size_kb}KB,hits=${l1_hits}] L2[entries=${l2_total},compressed=${l2_compressed},fresh=${l2_fresh},hits=${l2_hits}] total_hits=${hits} misses=${misses} hit_rate=${hit_rate}%"
}
