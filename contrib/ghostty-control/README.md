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

The bridge also supports `surface.io.subscribe` / `surface.io.unsubscribe`.
Each subscription has a bounded queue (maximum 256 entries), output chunks are
limited to 64 KiB, and server-originated `surface.io.event` notifications carry
one monotonically increasing sequence plus an explicit dropped/resync marker.
The listener serializes all socket writes, so a slow watcher cannot block the
Ghostty actor. On the ShareCLI side, consume this stream with:

```sh
sharecli surface watch --surface-id <surface-id>
```

Requests without a JSON-RPC `id` are notifications and receive no response.
The native provider still has to publish PTY events into `LiveIOEventHub`; the
bridge does not infer terminal output from AppleScript or shell processes.

### Fork lifecycle checklist

The provider belongs in the Ghostty app target, beside the app/surface registry;
this package must not retain `ghostty_app_t`, `ghostty_surface_t`, or `SurfaceView`
references. Bind the listener after `Ghostty.App` reaches its ready state and
stop it before app/surface teardown. Resolve each request through a weak
`SurfaceView` record keyed by its UUID, then hop to `@MainActor` for the short
operation. Never use a raw C pointer as a durable surface ID: upstream frees the
underlying `ghostty_surface_t` with the owning `SurfaceView`.

The upstream C API provides foreground PID/TTY and bounded screen-text reads,
but no public raw-PTY subscription callback. A fork must publish output into
`LiveIOEventHub` from its own termio/app callback instrumentation and should
publish title, cwd, resize, and child-exit changes from the corresponding
`SurfaceView`/Ghostty callbacks. If that instrumentation is unavailable, report
`read: false`/live events unavailable rather than scraping AppleScript or
executing a shell command.

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
