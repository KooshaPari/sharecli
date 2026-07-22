# Agent Mesh Coordination Layer — Phased WBS Plan

**Version:** 1.0  
**Date:** February 2026  
**Prerequisite:** agent-harness Phases 1-10 (core.sh, bin/harness)  
**Sponsor Model:** Human as executive sponsor (absolute last resort for decisions)

---

## Executive Summary

This plan implements a heterogeneous coding agent mesh where:
- Each CLI tool (Claude Code, Aider, Cursor, Cline) is treated as an **opaque atomic unit** with its own internal sub-hierarchy
- The mesh layer coordinates **between** these units, not within them
- Human escalation is architecturally minimized through boundary-based autonomy, confidence-weighted consensus, and multi-tier agent resolution
- Git worktree isolation provides filesystem safety; sandboxing provides execution safety

**Total Phases:** 8 (Phases 11-18, continuing from agent-harness)  
**Estimated Implementation:** 6-8 weeks

---

## Phase 11: Agent Registry & Process Discovery

**Objective:** Detect, register, and monitor heterogeneous coding agents on the local machine.

### 11.1 Process Scanner

| Task | Description | Deliverable |
|------|-------------|-------------|
| 11.1.1 | Implement `/proc` cmdline scanner for agent detection | `lib/discovery.sh` |
| 11.1.2 | Detection patterns: Claude Code (node/native binary), Aider (python), Cursor/Windsurf (electron), Cline (vscode extension path) | Pattern registry YAML |
| 11.1.3 | Extract working directory from `/proc/[pid]/cwd` | Project association |
| 11.1.4 | Real-time process monitoring via `forkstat` or polling fallback (5s interval) | `bin/mesh-watchdog` |

### 11.2 Agent Manifest System

| Task | Description | Deliverable |
|------|-------------|-------------|
| 11.2.1 | Define agent manifest schema (YAML): pid, type, capabilities, priority, ODD boundaries | `schemas/agent-manifest.yaml` |
| 11.2.2 | Self-registration: agents write manifests to `$MESH_DIR/agents/` on startup | Registration protocol |
| 11.2.3 | Capability declaration: file types, languages, frameworks, specializations | Capability taxonomy |
| 11.2.4 | Operational Design Domain (ODD): per-agent competency boundaries | ODD schema |

### 11.3 Heartbeat & Liveness

| Task | Description | Deliverable |
|------|-------------|-------------|
| 11.3.1 | Touch-file heartbeat at `$MESH_DIR/heartbeats/agent-{uuid}` | Heartbeat protocol |
| 11.3.2 | Heartbeat interval: 5 seconds, failure threshold: 15 seconds (3 missed) | Timing constants |
| 11.3.3 | Grace period: 30 seconds before orphan cleanup | Orphan detector |
| 11.3.4 | Stale agent cleanup: reclaim tasks, notify dependents | Cleanup daemon |

### 11.4 Directory Structure Bootstrap

```
$MESH_DIR/                              # /tmp/agent-mesh or /dev/shm/agent-mesh
├── agents/                             # Agent manifests
│   └── agent-{uuid}.yaml
├── heartbeats/                         # Touch files (check mtime)
│   └── agent-{uuid}
├── capabilities/                       # Indexed capability lookups
│   ├── by-language/
│   ├── by-framework/
│   └── by-specialization/
└── discovery/
    └── process-map.json                # PID → agent-uuid mapping
```

**Exit Criteria:** `mesh-watchdog` correctly identifies all running coding agents, maintains registry, detects agent departure within 15 seconds.

---

## Phase 12: Git Worktree Isolation & Workspace Management

**Objective:** Provide each agent with isolated filesystem via git worktrees.

### 12.1 Worktree Lifecycle

| Task | Description | Deliverable |
|------|-------------|-------------|
| 12.1.1 | Create worktree on agent registration: `git worktree add .mesh/worktrees/agent-{uuid} -b mesh/{uuid}` | `lib/worktree.sh` |
| 12.1.2 | Branch naming convention: `mesh/{agent-uuid}/{task-id}` | Branch schema |
| 12.1.3 | Worktree cleanup on agent departure (after grace period) | Cleanup hooks |
| 12.1.4 | Worktree health checks: detect corrupted or orphaned worktrees | Health monitor |

### 12.2 Branch Coordination

| Task | Description | Deliverable |
|------|-------------|-------------|
| 12.2.1 | Track active branches per agent in registry | Branch registry |
| 12.2.2 | Prevent branch collisions: atomic branch creation with retry | Collision avoidance |
| 12.2.3 | Merge queue: agents submit completed branches for integration | `$MESH_DIR/merge-queue/` |
| 12.2.4 | Conflict detection: pre-merge diff analysis | Conflict detector |

### 12.3 Integration with Existing Tools

| Task | Description | Deliverable |
|------|-------------|-------------|
| 12.3.1 | Claude Squad compatibility: respect existing worktree patterns | Compatibility layer |
| 12.3.2 | Cursor Background Agents: detect cloud VM worktrees | Remote detection |
| 12.3.3 | Aider: single-worktree mode (Aider manages its own commits) | Aider adapter |

**Exit Criteria:** Multiple agents operate in isolated worktrees, no file conflicts during parallel development, clean merge path to main.

---

## Phase 13: Boundary-Based Autonomy & Sandboxing

**Objective:** Replace per-action approval with boundary-based autonomy using sandboxing.

### 13.1 Sandbox Integration

| Task | Description | Deliverable |
|------|-------------|-------------|
| 13.1.1 | Integrate bubblewrap (Linux) / seatbelt (macOS) sandbox runtime | `lib/sandbox.sh` |
| 13.1.2 | Define filesystem boundaries: agent can only access own worktree + shared read-only dirs | Filesystem policy |
| 13.1.3 | Network boundaries: allowlist of approved domains per agent | Network policy |
| 13.1.4 | Process boundaries: prevent agent from spawning unrestricted subprocesses | Process policy |

### 13.2 Autonomy Tiers

| Tier | Description | Approval Model |
|------|-------------|----------------|
| **Tier 1** | Read-only operations, analysis, search | Auto-approve |
| **Tier 2** | File writes within own worktree | Auto-approve |
| **Tier 3** | Git commits within own branch | Auto-approve |
| **Tier 4** | Merge to shared branch, external API calls | Agent consensus or test-gate |
| **Tier 5** | Production deployment, credential access, irreversible ops | Human approval |

### 13.3 Test-Gated Autonomy

| Task | Description | Deliverable |
|------|-------------|-------------|
| 13.3.1 | Detect project test suite (pytest, jest, go test, etc.) | Test suite detector |
| 13.3.2 | Run tests after agent modifications | Test runner integration |
| 13.3.3 | Auto-approve if tests pass; escalate if tests fail after N retries | Test gate logic |
| 13.3.4 | Coverage delta tracking: flag if coverage drops significantly | Coverage monitor |

### 13.4 Hooks System

| Task | Description | Deliverable |
|------|-------------|-------------|
| 13.4.1 | PreToolUse hooks: check boundary before action | Hook framework |
| 13.4.2 | PostToolUse hooks: validate results, update state | Post-action hooks |
| 13.4.3 | Stop hooks: emergency halt conditions | Circuit breakers |
| 13.4.4 | Hook configuration: YAML-based policy definitions | Policy YAML |

**Exit Criteria:** Agents operate within defined boundaries without per-action prompts; permission prompts reduced by ≥80% vs baseline.

---

## Phase 14: Task Management & Kanban Filesystem

**Objective:** Implement filesystem-based task coordination with dependency tracking.

### 14.1 Task Schema

```yaml
# $MESH_DIR/tasks/{status}/task-{id}.yaml
id: task-001
title: "Implement user authentication"
description: "Add OAuth2 login flow with Google provider"
priority: P2                    # P1-P4 (Linear-style)
status: pending                 # pending, claimed, in_progress, blocked, review, done
assignee: null                  # agent-uuid when claimed
depends_on: [task-000]          # hard dependencies (block start)
soft_deps: [task-002]           # soft dependencies (can start with partial info)
created_at: "2026-02-10T12:00:00Z"
claimed_at: null
completed_at: null
artifacts:
  - path: "src/auth/oauth.ts"
    hash: "sha256:abc123..."
timeout_seconds: 3600           # 1 hour default
escalation_policy:
  after_600s: notify_lead       # 10 min: ping lead agent
  after_1800s: reassign         # 30 min: try different agent
  after_3600s: escalate_human   # 60 min: human review
```

### 14.2 Kanban Directory Structure

```
$MESH_DIR/tasks/
├── pending/                    # Ready to claim
├── claimed/                    # Claimed but not started
├── in_progress/                # Active work
├── blocked/                    # Waiting on dependencies
├── review/                     # Awaiting review/approval
├── done/                       # Completed
└── index.yaml                  # Generated read-only overview
```

### 14.3 Atomic Task Operations

| Task | Description | Deliverable |
|------|-------------|-------------|
| 14.3.1 | Claim: atomic `mkdir $MESH_DIR/claims/task-{id}/` (EEXIST = already claimed) | Claim protocol |
| 14.3.2 | Lease-based claiming: `lease_expires` timestamp, 30-second renewal | Lease manager |
| 14.3.3 | State transitions: atomic `rename()` between status directories | State machine |
| 14.3.4 | Dependency resolution: auto-unblock when predecessors complete | DAG walker |

### 14.4 Task Allocation Strategies

| Strategy | Use Case | Implementation |
|----------|----------|----------------|
| **Atomic claim** | Interchangeable tasks | First `mkdir` wins |
| **Contract Net** | Heterogeneous capabilities | CFP → Bids → Award |
| **Capability match** | Specialized work | Query capability index |
| **Load balance** | Even distribution | Least-loaded agent |

### 14.5 Contract Net Protocol

```
$MESH_DIR/contracts/
├── cfp/                        # Calls for proposals
│   └── cfp-{id}.yaml          # Task + requirements
├── bids/                       # Per-CFP bid directories
│   └── cfp-{id}/
│       └── agent-{uuid}.yaml  # Capability score + time estimate
└── awards/
    └── cfp-{id}.yaml          # Winning agent + rationale
```

**Exit Criteria:** Tasks flow through Kanban states correctly; dependency DAG correctly blocks/unblocks; no double-claiming; orphaned tasks reclaimed within 60 seconds.

---

## Phase 15: Consensus & Voting Mechanisms

**Objective:** Implement confidence-weighted consensus for decisions requiring agent agreement.

### 15.1 Voting Protocol Suite

| Protocol | Threshold | Use Case |
|----------|-----------|----------|
| **Simple majority** | >50% | Routine implementation choices |
| **Supermajority** | ≥66% | Architectural changes, API modifications |
| **Unanimity** | 100% | Security-critical, production deployments |
| **Weighted** | ≥66% weighted | Default; incorporates confidence + specialization |

### 15.2 Confidence-Weighted BFT

| Task | Description | Deliverable |
|------|-------------|-------------|
| 15.2.1 | Confidence probe: structured self-reflection prompt for each agent | Probe template |
| 15.2.2 | Confidence weighting: `vote_weight = base_weight × confidence_score` | Weight calculator |
| 15.2.3 | Specialization boost: +20% weight for domain expert on relevant decisions | Specialization index |
| 15.2.4 | Reputation tracking: historical accuracy on similar decisions | Reputation store |

### 15.3 Consensus Directory Structure

```
$MESH_DIR/consensus/
├── proposals/                  # Independent drafts (all-agents drafting)
│   └── decision-{id}/
│       └── agent-{uuid}.yaml  # Solution + confidence + rationale
├── votes/                      # Weighted votes
│   └── decision-{id}/
│       └── agent-{uuid}.yaml  # Vote + weight + justification
├── results/                    # Final decisions
│   └── decision-{id}.yaml     # Outcome + vote tally + dissents
└── active/
    └── decision-{id}.yaml     # Currently open decisions
```

### 15.4 Consensus Protocol Flow

```
1. PROPOSE: Initiator writes decision request to consensus/active/
2. DRAFT: All agents independently write solutions to proposals/{id}/
   - Wait for: all agents OR timeout (60s) OR quorum (66%)
3. SHARE: Solutions revealed simultaneously
   - Hash comparison: if all match → fast-path accept
4. VOTE: Agents review all proposals, write weighted votes
5. TALLY: Coordinator aggregates votes per threshold
6. DECIDE: Winner → results/{id}.yaml; notify all agents
```

### 15.5 Six Sigma Reliability Calculator

| Task | Description | Deliverable |
|------|-------------|-------------|
| 15.5.1 | Implement reliability formula: `system_error = O(p^{⌈n/2⌉})` | Calculator |
| 15.5.2 | Recommend agent count for target reliability (e.g., 5 agents → 0.11% error) | Sizing tool |
| 15.5.3 | Cost-reliability tradeoff visualization | Dashboard widget |

**Exit Criteria:** Consensus decisions complete within 2 minutes; weighted voting correctly incorporates confidence; heterogeneous agent teams achieve higher accuracy than homogeneous.

---

## Phase 16: Gated Multi-Agent Debate

**Objective:** Implement debate protocol that triggers only when beneficial, with adaptive stopping.

### 16.1 Debate Gating

| Task | Description | Deliverable |
|------|-------------|-------------|
| 16.1.1 | Self-critique extraction: 41 linguistic/semantic features | Feature extractor |
| 16.1.2 | Debate prediction model: should debate improve this answer? | Gate classifier |
| 16.1.3 | Automatic triggers: high-stakes decisions, low initial confidence, conflicting proposals | Trigger rules |
| 16.1.4 | Skip criteria: unanimous agreement, simple factual questions, reversible decisions | Skip rules |

### 16.2 Debate Protocol

```
$MESH_DIR/debate/
├── sessions/
│   └── debate-{id}/
│       ├── config.yaml         # Participants, topic, max rounds
│       ├── rounds/
│       │   ├── round-1/
│       │   │   └── agent-{uuid}.yaml  # Position + evidence + confidence
│       │   └── round-2/
│       │       └── agent-{uuid}.yaml  # Revised position
│       ├── stability.json      # Belief distribution tracking
│       └── result.yaml         # Final outcome
└── active/
    └── debate-{id}             # Currently running
```

### 16.3 Adaptive Stability Detection

| Task | Description | Deliverable |
|------|-------------|-------------|
| 16.3.1 | Track belief distributions across rounds | Distribution tracker |
| 16.3.2 | KS-test for distributional stability: stop when further rounds won't shift outcome | Stability detector |
| 16.3.3 | Maximum round cap: 3 rounds (research shows degradation beyond) | Round limiter |
| 16.3.4 | Early termination: if 2/3 agents converge with high confidence | Convergence detector |

### 16.4 Heterogeneous Team Composition

| Task | Description | Deliverable |
|------|-------------|-------------|
| 16.4.1 | Ensure debate teams include different model families | Team composer |
| 16.4.2 | Tool diversity: assign different evidence sources per agent (Tool-MAD pattern) | Tool assignment |
| 16.4.3 | Hide confidence scores during debate (prevent anchoring) | Confidence masking |
| 16.4.4 | Majority pressure mitigation: anonymous initial positions | Position anonymization |

**Exit Criteria:** Debate triggers only when predicted to help (≥70% precision); converges within 2 rounds on 80% of cases; heterogeneous teams outperform homogeneous by ≥5%.

---

## Phase 17: Multi-Tier Escalation Architecture

**Objective:** Exhaust all agent resolution before human escalation; human is absolute last resort.

### 17.1 Escalation Tiers

| Tier | Handler | Trigger | Max Duration |
|------|---------|---------|--------------|
| **0** | Self-resolution | Initial attempt | 10 min |
| **1** | Peer review | Self-resolution failed | 10 min |
| **2** | Lead agent | Peer disagreement | 15 min |
| **3** | Agent committee | Lead uncertain | 15 min |
| **4** | Human sponsor | Committee failed | Async (batched) |

### 17.2 Composite Escalation Scoring

```python
escalation_urgency = (
    0.30 × (1 - confidence_score) +      # Low confidence drives escalation
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

### 17.3 Lazy Escalation Prevention

| Task | Description | Deliverable |
|------|-------------|-------------|
| 17.3.1 | Mandatory self-resolution checklist: ≥3 approaches documented | Checklist validator |
| 17.3.2 | Escalation rejection: auto-reject if checklist incomplete | Gate logic |
| 17.3.3 | Causal influence tracking: flag agents that escalate without contributing | Contribution tracker |
| 17.3.4 | Escalation cooldown: same issue can't re-escalate for 5 minutes | Rate limiter |

### 17.4 Hard Gates (Always Escalate to Human)

| Category | Examples | Rationale |
|----------|----------|-----------|
| **Credentials** | API keys, passwords, tokens | Security-critical |
| **Production** | Deploy, database migration, DNS | Irreversible |
| **External APIs** | Payments, emails, SMS | Side effects |
| **Cost thresholds** | >$100 API spend, >1hr compute | Budget control |
| **Data deletion** | DROP TABLE, rm -rf, S3 delete | Irreversible |
| **New dependencies** | Adding npm/pip packages | Supply chain |
| **Audit-affecting** | Compliance logs, access control | Regulatory |

### 17.5 Never Escalate

| Category | Examples | Rationale |
|----------|----------|-----------|
| **Formatting** | Indentation, line length, naming | Style preference |
| **Implementation detail** | Which loop construct, variable names | Within spec |
| **Test fixes** | Iterating on failing tests | Self-correcting |
| **Patch dependencies** | Minor version bumps | Low risk |
| **Documentation** | README, comments, docstrings | Reversible |

### 17.6 Human Escalation Queue

```
$MESH_DIR/escalation/
├── queue/                      # Pending human review
│   └── esc-{id}.yaml          # Full context + approaches tried + confidence
├── batched/                    # Grouped for daily digest
│   └── batch-{date}.yaml
├── blocking/                   # Sync escalations (agent waits)
│   └── esc-{id}.yaml
└── resolved/
    └── esc-{id}.yaml          # Human decision + timestamp
```

### 17.7 Escalation Payload Schema

```yaml
id: esc-001
type: blocking | async
created_at: "2026-02-10T14:30:00Z"
agent: agent-{uuid}
task: task-{id}
tier_path: [0, 1, 2, 3]         # Tiers exhausted
urgency_score: 0.85
risk_category: production
summary: "Unable to resolve merge conflict in auth module"
context:
  attempts:
    - approach: "Auto-merge with ours strategy"
      result: "Semantic conflict in token validation"
    - approach: "Requested peer review from agent-xyz"
      result: "Peer also uncertain, recommended human review"
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

**Exit Criteria:** Human escalations occur only after all 4 agent tiers exhausted; escalation rate <5% of decisions; blocking escalations <1%.

---

## Phase 18: Observability, Monitoring & Integration

**Objective:** Complete observability stack and integration with existing agent-harness.

### 18.1 Event Logging

| Task | Description | Deliverable |
|------|-------------|-------------|
| 18.1.1 | Structured event log: JSONL append-only format | `$MESH_DIR/logs/events.jsonl` |
| 18.1.2 | Event types: agent_registered, task_claimed, consensus_started, escalation_triggered, etc. | Event schema |
| 18.1.3 | Correlation IDs: trace events across agents and decisions | Tracing integration |
| 18.1.4 | Log rotation: hourly rotation, 7-day retention | Rotation policy |

### 18.2 Metrics Collection

| Metric | Description | Alert Threshold |
|--------|-------------|-----------------|
| **agent_count** | Active agents in mesh | <2 (degraded) |
| **task_throughput** | Tasks completed per hour | <5 (blocked) |
| **escalation_rate** | % decisions escalated to human | >10% (review needed) |
| **consensus_time_p95** | 95th percentile consensus latency | >5 min (slow) |
| **debate_trigger_rate** | % decisions triggering debate | >30% (over-debating) |
| **heartbeat_failures** | Agent departures per hour | >3 (instability) |

### 18.3 Dashboard

| Task | Description | Deliverable |
|------|-------------|-------------|
| 18.3.1 | Real-time agent status: active, capabilities, current task | Status panel |
| 18.3.2 | Task Kanban: visual flow of tasks through states | Kanban view |
| 18.3.3 | Consensus history: decisions made, vote tallies, outcomes | Decision log |
| 18.3.4 | Escalation queue: pending human reviews with context | Escalation panel |
| 18.3.5 | Reliability metrics: Six Sigma calculator output | Reliability gauge |

### 18.4 Integration with Agent-Harness

| Integration Point | Harness Component | Mesh Component |
|-------------------|-------------------|----------------|
| File coordination | Phase 8 locks/claims | Task claiming |
| Cache sharing | Phase 9 request coalescing | Shared knowledge base |
| Tracing | Phase 10 observability | Correlation IDs |
| Conflict detection | Phase 8 intent broadcasting | Pre-commit hooks |

### 18.5 CLI Commands

```bash
# Agent management
mesh agents list                    # Show registered agents
mesh agents status {uuid}           # Detailed agent status
mesh agents capabilities            # Capability matrix

# Task management
mesh tasks list [--status=pending]  # List tasks by status
mesh tasks create --file=task.yaml  # Create new task
mesh tasks assign {task-id} {agent} # Manual assignment

# Consensus
mesh consensus start --file=decision.yaml  # Initiate decision
mesh consensus status {decision-id}         # Check status
mesh consensus history                      # Past decisions

# Escalation
mesh escalation queue               # Show pending escalations
mesh escalation resolve {esc-id}    # Human resolves escalation
mesh escalation batch               # Generate daily digest

# Monitoring
mesh status                         # Overall mesh health
mesh metrics                        # Current metrics
mesh logs [--follow]                # Stream event log
```

### 18.6 Notification System

| Channel | Use Case | Implementation |
|---------|----------|----------------|
| **File watcher** | Agent-to-agent notifications | inotifywait on inbox |
| **SIGUSR1** | Urgent interrupts | Signal + flag file |
| **Slack/Discord** | Human escalation digest | Webhook integration |
| **Desktop notification** | Blocking escalation | notify-send / osascript |

**Exit Criteria:** Complete visibility into mesh operations; humans notified only for Tier 4 escalations; CLI provides full operational control; integration with harness Phases 8-10 verified.

---

## Implementation Timeline

| Phase | Duration | Dependencies | Key Deliverables |
|-------|----------|--------------|------------------|
| **11** | 1 week | Harness Phases 1-10 | Agent registry, discovery, heartbeat |
| **12** | 1 week | Phase 11 | Git worktree isolation, branch coordination |
| **13** | 1 week | Phase 12 | Sandbox integration, autonomy tiers, test gates |
| **14** | 1 week | Phase 11 | Task Kanban, claiming, dependency DAG |
| **15** | 1 week | Phases 11, 14 | Voting protocols, confidence weighting, BFT |
| **16** | 1 week | Phase 15 | Gated debate, stability detection, team composition |
| **17** | 1 week | Phases 15, 16 | Escalation tiers, lazy prevention, hard gates |
| **18** | 1 week | All phases | Observability, CLI, dashboard, integration |

**Total:** 8 weeks (can parallelize 14 with 12-13)

---

## Risk Mitigation

| Risk | Impact | Mitigation |
|------|--------|------------|
| Agent heterogeneity gaps | Correlated failures | Enforce minimum 2 model families in any consensus |
| Escalation rate creep | Human overload | Weekly review of escalation logs, tune thresholds |
| Worktree proliferation | Disk exhaustion | Aggressive cleanup, worktree limits per agent |
| Debate over-triggering | Latency bloat | Gate classifier precision ≥70%, hard round cap |
| Consensus deadlock | Stuck decisions | Timeout + fallback to lead agent decision |
| Sandbox escape | Security breach | Regular audit, minimal privilege, network allowlist |

---

## Success Metrics

| Metric | Target | Measurement |
|--------|--------|-------------|
| Human escalation rate | <5% of decisions | Tier 4 / total decisions |
| Consensus accuracy | >90% | Correct outcomes (retrospective review) |
| Task throughput | ≥10/hour | Completed tasks per hour |
| P95 consensus latency | <2 min | Time from proposal to decision |
| Agent utilization | >70% | Time spent on tasks vs idle |
| Test pass rate | >95% | Post-agent-modification test runs |
| Permission prompts | ≤5/hour | Manual approvals required |

---

## Appendix A: File Structure Summary

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
├── tasks/                              # Kanban filesystem
│   ├── pending/
│   ├── claimed/
│   ├── in_progress/
│   ├── blocked/
│   ├── review/
│   ├── done/
│   └── index.yaml
├── claims/                             # mkdir-based atomic locks
│   └── task-{id}/
│       ├── owner
│       └── lease_expires
├── contracts/                          # Contract Net Protocol
│   ├── cfp/
│   ├── bids/
│   └── awards/
├── consensus/                          # Voting and decisions
│   ├── proposals/
│   ├── votes/
│   ├── results/
│   └── active/
├── debate/                             # Multi-agent debate
│   ├── sessions/
│   └── active/
├── escalation/                         # Human escalation queue
│   ├── queue/
│   ├── batched/
│   ├── blocking/
│   └── resolved/
├── inbox/                              # Per-agent message queues
│   └── agent-{uuid}/
│       ├── *.json
│       └── processed/
├── blackboard/                         # Shared knowledge
│   ├── codebase_map.md
│   ├── facts.json
│   └── decisions.jsonl
├── wal/                                # Write-ahead log
│   └── {seq}.log
├── logs/                               # Event logging
│   ├── events.jsonl
│   └── metrics.jsonl
└── workspaces/                         # Per-agent scratch
    └── agent-{uuid}/
```

---

## Appendix B: Protocol Quick Reference

### Task Claiming (atomic mkdir)
```bash
if mkdir "$MESH_DIR/claims/task-$ID" 2>/dev/null; then
    echo "$AGENT_UUID" > "$MESH_DIR/claims/task-$ID/owner"
    date -d "+30 seconds" +%s > "$MESH_DIR/claims/task-$ID/lease_expires"
    # Claimed successfully
else
    # Already claimed by another agent
fi
```

### Heartbeat Update
```bash
touch "$MESH_DIR/heartbeats/$AGENT_UUID"
```

### Consensus Vote
```bash
cat > "$MESH_DIR/consensus/votes/$DECISION_ID/$AGENT_UUID.yaml" << EOF
vote: approve
weight: 0.85
confidence: 0.9
justification: "Solution passes all tests and follows established patterns"
EOF
```

### Escalation Trigger
```bash
if (( $(echo "$URGENCY > 0.8" | bc -l) )); then
    cp "$ESCALATION_PAYLOAD" "$MESH_DIR/escalation/queue/"
    notify-send "Mesh Escalation" "Human review required: $TASK_ID"
fi
```

---

## Appendix C: Integration Checklist

- [ ] Phase 11 builds on harness `lib/core.sh` primitives
- [ ] Phase 12 integrates with harness file coordination (Phase 8)
- [ ] Phase 13 sandbox policies align with harness security model
- [ ] Phase 14 task DAG uses harness HLC timestamps
- [ ] Phase 15 consensus uses harness tracing correlation IDs
- [ ] Phase 18 metrics feed into harness observability stack
- [ ] All phases respect harness tmpfs management (Phase 10)
- [ ] CLI commands follow harness `bin/harness` patterns
