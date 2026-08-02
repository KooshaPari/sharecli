# ShareCLI Ghostty control bridge

This Swift package is the tested JSON-RPC dispatcher, owner-only Unix listener,
and `SurfaceProvider` contract for ShareCLI's native Ghostty integration. It is
intentionally a small, app-side bridge: it validates bounded requests and
routes typed surface operations; it never evaluates shell text or owns a PTY by
itself.

## Ghostty fork integration

The Ghostty app/fork must provide a concrete `SurfaceProvider` backed by its
surface and PTY objects, then start `UnixControlServer` with that provider. The
provider must expose stable surface IDs, process evidence, live
read/write/resize operations, and capability reporting. Provider methods are
`async` so a `@MainActor` implementation can safely access Ghostty's live
surface tree without semaphore or synchronous cross-actor calls. Keep all app
and PTY references on that side of the boundary.

Requests are bounded: the NDJSON request and read response are capped at 1 MiB,
and one `surface.io.send` payload is capped at 64 KiB. The provider result is
also checked against the requested read size before it is serialized.

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

This package is a fork-ready binding contract and listener, not a patched
Ghostty application. The concrete `SurfaceProvider` and Ghostty app lifecycle
wiring remain an integration task in the Ghostty fork.
