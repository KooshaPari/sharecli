# Implementation strategy

Use narrow Rust traits at the terminal boundary: `SurfaceAdapter`,
`OutputObserver`, `InputDispatcher`, and `LayoutRestorer`. Keep Ghostty-specific
transport behind one adapter crate/module. Existing `sharecli-session` remains
the ledger domain; existing ShareCLI IPC remains the local operator transport.

Start with capability probing and read-only discovery. Enable input dispatch
only when an adapter returns a stable pane identifier and explicit send
capability. The recovery executor invokes structured argv recipes, never a
shell string. It limits concurrent launches, records every decision, and makes
manual intervention a first-class result.

FUSE must expose capabilities rather than deciding recovery correctness. The
macOS implementation must retain Windows WinFsp build support, distinguish a
real FSKit request from the kernel/default fuser path, and return NonFuse when
neither optional backend is usable.
