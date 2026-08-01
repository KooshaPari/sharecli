# Session recovery DAG and work breakdown

```
P0 Truth and safety
  +-- repair FUSE capability semantics -----------+
  +-- make existing recovery docs truthful -------+-- P1 ledger
                                                    |
P1 Durable session ledger -------------------------+
                                                    +-- P2 terminal adapters
P2 Ghostty capability/discovery adapter -----------+       |
P2 Managed PTY/zmx adapter ------------------------+       +-- P3 local RPC/live I/O
                                                            |
P3 Harness resolver + resume registry ---------------------+-- P4 executor/layout restore
                                                                    |
P4 CLI/IPC/dashboard operator UX ---------------------------+-- P5 crash dogfood
```

## Work packages

| ID | Deliverable | Depends on |
| -- | ----------- | ---------- |
| P0 | Typed KEXT -> FSKit -> non-FUSE capability state, restore WinFsp build hook | current dirty FUSE work |
| P1 | WAL schema, migrations, observation writer, retention/compaction | P0 |
| P2 | Ghostty native capability probe and surface discovery contract | P1 |
| P3 | Authenticated local RPC for observe/send/cancel with bounded queues | P2 |
| P4 | Harness adapters, confidence model, recovery planner/executor | P1, P2, P3 |
| P5 | Layout restore, crash/restart dogfood, tray/dashboard cockpit | P4 |

## Critical path

P0 -> P1 -> P2 -> P3 -> P4 -> P5. FUSE mounting is not on the critical path;
its capability ladder is validated in parallel and cannot block P1-P5.
