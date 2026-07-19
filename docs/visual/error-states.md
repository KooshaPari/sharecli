# Error and failure states (C10 L101)

Designed error states explain what failed, why it matters, and the recovery command — not a silent spinner or generic browser error.

## Dashboard WebSocket disconnect

| Element | Contract |
|---------|----------|
| Trigger | `WebSocket` `onclose` / `onerror` before reconnect timer fires |
| Panel | `.error-state` with `role="alert"` and `data-error-kind="disconnect"` |
| Title | `Dashboard disconnected` |
| Body | Mentions `sharecli serve` must be running |
| Primary CTA | **Retry now** button — clears timer and reconnects immediately |
| Secondary hint | `sharecli serve --bind 127.0.0.1:9000` |

On successful reconnect, the error panel is cleared and normal table / empty-state rendering resumes.

Implementation: `src/dashboard.html` (`renderDisconnectError`). Regression: `tests/c10_l101_error_states.rs`.

## CLI errors (existing)

Structured config validation errors (`src/config_validator.rs`) and TUI red-gate retry copy remain the CLI-side failure design. This lane covers the **dashboard disconnect** branch only.

## Acceptance

- Icon + explanation + recovery CTA on dashboard disconnect
- Retry button bypasses the 3s backoff
- See also [VISUAL_SPEC.md](VISUAL_SPEC.md) and [empty-states.md](empty-states.md)
