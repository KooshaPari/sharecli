# Kilo Gastown Methodology Spec — thegent-sharecli

## Overview

`thegent-sharecli` is a rig in the Kilo Gastown multi-rig coordination system (town ID: `78a8d430-a206-4a25-96c0-5cd9f5caf984`, rig ID: `03e7d736-587f-4e1a-aa4d-ea735ad5df45`). This document explains how Kilo Gastown mechanics apply to this repository.

## Town Architecture

```
78a8d430 (Town)
└── 03e7d736 (this rig: thegent-sharecli)
    ├── other rigs in town...
```

- **Town:** A collection of rigs coordinated by a Mayor agent. The Mayor oversees cross-rig dependencies, convoy scheduling, and town-wide progress tracking.
- **Rig:** An individual repository with polecat agents working beads. Each rig has a unique ID and hosts one or more agents.
- **Polecat:** Worker agents that pick up and execute beads within a rig.

## Beads

Beads are the atomic work units in Kilo Gastown.

| Type | Description |
|------|-------------|
| `issue` | A task or feature request to be implemented |
| `convoy` | A tracked group of related beads with dependency chains |
| `merge_request` | A code change awaiting review and merge |

### Bead Lifecycle

1. **Open** → **In Progress**: Agent picks up the bead
2. **In Progress** → **In Review**: Agent completes work and calls `gt_done`
3. **In Review** → **Merged**: Refinery merges the branch
4. **In Review** → **Rework**: Reviewer requests changes; bead returns to **In Progress**

## Convoys

Convoys track groups of beads that span multiple rigs or require coordinated changes across repositories.

### When to Use Convoys

Use a convoy when:
- A feature or fix touches multiple rigs
- There is a dependency chain where one rig's changes depend on another rig's changes
- You need cross-rig progress tracking via `gt_list_convoys`

### Staged vs Active Convoys

| State | Description |
|-------|-------------|
| **Staged** | Convoy is created but not yet active. Beads may be assigned but no agent is working. |
| **Active** | At least one bead in the convoy is in progress. Agents are actively working beads. |

### Convoy Naming Convention

```
convoy/{short-description}/{convoy_id}/head
```

Example: `convoy/agileplus-kilo-specs-thegent-sharecli/6893f09f/head`

### Feature Branch Naming

Each rig's worktree within a convoy follows:

```
convoy/{short-description}/{convoy_id}/gt/{agent-name}/{bead_id}
```

Example: `convoy/agileplus-kilo-specs-thegent-sharecli/6893f09f/gt/polecat-34/d5322733`

## Delegation Tools

### gt_sling

Delegates a single bead to another agent.

```
gt_sling --bead-id <id> --to-agent <agent-id>
```

### gt_sling_batch

Delegates multiple beads at once.

```
gt_sling_batch --bead-ids <id1,id2,...> --to-agent <agent-id>
```

## Progress Tracking

### gt_list_convoys

Lists all convoys in the town with their status.

### gt_convoy_status

Returns detailed status of a specific convoy, including all constituent beads and their states.

## Merge Modes

### Review-Then-Land (Coupled)

The default mode. When an agent calls `gt_done`:
1. Branch is pushed to review queue
2. Bead transitions to `in_review`
3. Refinery reviews and merges
4. Changes land on the default branch

### Review-and-Merge (Independent)

Changes are reviewed and merged independently without waiting for convoy-wide coordination. Used when beads have no cross-rig dependencies.

## How This Rig Fits Into the Town

`thegent-sharecli` participates in the town as a **shared infrastructure component**:

- **Dependency Provider:** Other rigs may depend on sharecli for process management or CLI utilities.
- **Cross-Rig Convoys:** When sharecli behavior changes affect other rigs, those changes travel via convoy with explicit dependency declarations.
- **Bead Assignment:** Beads targeting this rig are dispatched by the Mayor and picked up by available polecat agents.

## Kilo Gastown Commands Reference

| Command | Purpose |
|---------|---------|
| `gt_prime` | Get full context: identity, hooked bead, mail, open beads |
| `gt_done` | Complete current bead, push branch, transition to `in_review` |
| `gt_sling` | Delegate a bead to another agent |
| `gt_sling_batch` | Delegate multiple beads |
| `gt_list_convoys` | List all convoys in town |
| `gt_convoy_status` | Get detailed convoy status |
| `gt_bead_status` | Inspect a bead's current state |
| `gt_bead_close` | Close a completed bead |
| `gt_mail_send` | Send coordination message to another agent |
| `gt_mail_check` | Check for undelivered mail |
| `gt_checkpoint` | Write crash-recovery data |
| `gt_status` | Emit dashboard status update |
| `gt_escalate` | Escalate an issue requiring human intervention |

## Agent Identity

Each agent has a unique identity:

```
Polecat-{number}-polecat-{rig_id}@{town_id}
```

Example: `Polecat-34-polecat-03e7d736@78a8d430`

## Conventions

### Commit Messages

Format: `{type}: {short description}`

Types: `feat`, `fix`, `docs`, `chore`, `refactor`, `test`

Example: `docs: add Kilo Gastown methodology spec`

### Branch Naming

Branches follow the convoy naming convention described above. Do not switch branches within a worktree.

### Pre-Submission Gates

Before calling `gt_done`, run quality gates:
1. `task quality`

If any gate fails, fix and re-run until all pass.

### Commit Hygiene

- Commit after every meaningful unit of work
- Push after every commit (container disk is ephemeral)
- Use descriptive commit messages referencing the bead

(End of file - total 127 lines)
