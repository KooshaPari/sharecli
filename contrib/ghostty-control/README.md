# ShareCLI Ghostty control bridge

This Swift package is the tested JSON-RPC dispatcher and `SurfaceProvider`
contract for ShareCLI's native Ghostty integration. It is intentionally a
small, app-side bridge: the dispatcher validates bounded requests and routes
typed surface operations; it never evaluates shell text or owns a PTY by
itself.

## Ghostty fork integration

The Ghostty app/fork must provide a concrete `SurfaceProvider` backed by its
surface and PTY objects, then own the local Unix-domain listener that feeds
newline-delimited requests to `ControlDispatcher`. The provider must expose
stable surface IDs, process evidence, live read/write/resize operations, and
capability reporting. Keep all app and PTY references on that side of the
boundary.

The listener is expected to be local-only and owner-readable/writable
(filesystem mode `0600`). When a control token is configured, pass it to
`ControlDispatcher(provider:expectedToken:)`; every request must then include
the matching top-level `token`. ShareCLI should treat missing sockets,
unsupported capabilities, and provider errors as explicit degraded states,
never as permission to execute an untrusted command.

## Build and test

```sh
swift test --filter ShareCLIGhosttyControlTests
```

This package is a fork-ready binding contract and dispatcher, not a patched
Ghostty application. The concrete `SurfaceProvider`, Unix listener, and
Ghostty app lifecycle wiring remain an integration task in the Ghostty fork.
