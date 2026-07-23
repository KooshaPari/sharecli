# Error and failure states (C10 L101)

Designed error states explain what failed, why it matters, and the recovery command — not a silent spinner or generic browser error.

## Dashboard WebSocket disconnect

| Element | Contract |
|---------|----------|
| Trigger | `WebSocket` `onclose` / `onerror` before reconnect timer fires |
| Panel | `.error-state` with `role="alert"` and `data-error-kind="disconnect"` |
| Illustration | Tier-1 SVG `assets/dashboard/ui/error-states/disconnect.svg` — severed serve↔dashboard WebSocket scene (Backbone-2 `--bb2-error`), served at `/assets/dashboard/ui/error-states/disconnect.svg` |
| Title | `Dashboard disconnected` |
| Body | Mentions `sharecli serve` must be running |
| Primary CTA | **Retry now** button — clears timer and reconnects immediately |
| Secondary hint | `sharecli serve --bind 127.0.0.1:9000` |

On successful reconnect, the error panel is cleared and normal table / empty-state rendering resumes.

The Phenotype pack `empty-states/error.svg` remains available for generic UI-pack use; the **disconnect panel must not** use that abstract decoration — it uses the bespoke tier-1 disconnect scene above.

Implementation: `src/dashboard.html` (`renderDisconnectError`). Embedded via `src/dashboard_assets.rs`. Regression: `tests/c10_l101_error_states.rs`. Provenance: [PROVENANCE.md](PROVENANCE.md).

## CLI errors (existing)

Structured config validation errors (`src/config_validator.rs`) and TUI red-gate retry copy remain the CLI-side failure design. This lane covers the **dashboard disconnect** branch only.

## Acceptance

- Tier-1 disconnect illustration (scene of severed feed) + explanation + recovery CTA
- Retry button bypasses the 3s backoff
- See also [VISUAL_SPEC.md](VISUAL_SPEC.md) and [empty-states.md](empty-states.md)
