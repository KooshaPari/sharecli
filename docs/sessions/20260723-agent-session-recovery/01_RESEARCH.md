# Research

- Ghostty 1.3.1 exposes AppleScript/App Intents but no macOS control socket.
- Ghostty main has capability-gated PID/TTY scripting support; stable binaries
  must report degraded identity resolution when unavailable.
- `zmx` supplies Unix-socket PTY attachment, VT state/history, send, and tail.
- NATS is unnecessary for a single-host control plane; use local RPC with a
  Unix socket and the existing ShareCLI HTTP/WebSocket bridge.
- Rio is Rust/WebGPU and actively changing; its public documentation exposes
  keybindings/configuration but no stable external session-control API. It is
  an adapter candidate, not a migration dependency.
