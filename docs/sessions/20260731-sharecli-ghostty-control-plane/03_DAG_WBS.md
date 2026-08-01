# DAG / WBS

```text
P0 FUSE truth + fail-open
  |
P1 durable observation WAL ----+
  |                             |
P2 adapter/resolver -----------+--> P4 CLI + IPC recovery
  |                             |
P3 Ghostty control client -----+--> P5 live I/O contract tests
                                |
                                +--> P6 native Ghostty server/fork
                                      |
                                      +--> P7 pane discovery/layout restore
                                            |
                                            +--> P8 macOS dogfood + chaos gate
```

Completed here: P0-P5's ShareCLI-side contracts, CLI/IPC integrations, and
tests. P6-P8 remain open because they require a Ghostty-side implementation
and real macOS permission, mount, crash, and restart validation.
