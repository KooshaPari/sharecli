# ShareCLI Ghostty Control Plane

Date: 2026-07-31

This session implements the first production slice of ShareCLI's terminal
control plane: durable session observations, evidence-gated resume recipes,
bounded crash recovery, a Ghostty Unix-socket client, and explicit FUSE
capability selection. The implementation remains scoped to ShareCLI.

Success criteria for this slice:

- session observations survive process and database reopen;
- ambiguous process evidence never becomes an unattended resume recipe;
- recovery uses argv (never a shell) and bounded non-blocking launches;
- IPC and CLI expose observation/recovery operations;
- Ghostty I/O is capability-gated and authenticated when a control socket exists;
- FUSE selection is KEXT first, approved FSKit second, then fail-open.

Not completed in this slice: a Ghostty fork/server, native pane enumeration,
continuous subscriptions, layout restore, and macOS mount dogfood.
