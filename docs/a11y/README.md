# Accessibility — sharecli

**Compliance posture:** WCAG 2.2 **Level A** for the embedded web dashboard; **dev-AX Level A** for CLI/TUI surfaces (structured output, non-color degradation, keyboard exits).

sharecli is primarily a developer-facing CLI with three human-visible surfaces:

| Surface | Path | Level A posture |
|---------|------|-----------------|
| Web dashboard | `src/dashboard.html` (served at `/` by `sharecli serve`) | Semantic landmarks (`nav`, `main`), `lang="en"`, live status regions; see `tests/a11y/` |
| Thermal TUI | `crates/sharecli-thermal-tui` | Keyboard-only operation (`q`, `Ctrl-C`); text labels alongside color (GREEN/YELLOW/RED) |
| CLI stdout/stderr | `src/main.rs` | `NO_COLOR` honored; `--json` / `--format json` machine-readable modes; plain-language errors |

## WCAG 2.2 Level A — CLI & TUI

The following Level A expectations apply to terminal surfaces (adapted from WCAG 1.4.1, 1.3.1, 2.1.1):

1. **Use of color (1.4.1):** Thermal state and process status expose text labels, not color alone (`level_label`, table columns). Set `NO_COLOR=1` or `TERM=dumb` to disable ANSI styling.
2. **Info and relationships (1.3.1):** `sharecli ps` prints fixed column headers; `sharecli list --json` returns structured fields.
3. **Keyboard (2.1.1):** TUI is fully operable without a mouse; see [`keyboard.md`](./keyboard.md).
4. **Timing adjustable (2.2.1):** TUI polls on a fixed interval; user can exit immediately with `q` or `Ctrl-C`.

Automated enforcement for the dashboard:

- **Rust (CI via `cargo test`):** `tests/a11y/dashboard_landmarks.rs` — landmark + `lang` assertions.
- **axe-core (CI via `.github/workflows/a11y.yml`):** `scripts/a11y/axe-dashboard.mjs` scans `src/dashboard.html` with jsdom (no browser). Tags **`wcag2a`**, **`wcag21a`**, **`wcag22a`** (WCAG 2.x Level A). The job **hard-fails** on **serious/critical** violations; moderate/minor are logged only.

Run locally: `npm ci && npm run a11y:dashboard`.

## Degraded-mode operation

| Condition | Behavior |
|-----------|----------|
| `NO_COLOR` set | Theme ANSI suppressed; output remains readable plain text |
| `TERM=dumb` | No escape sequences assumed; use `--json` where available |
| WebSocket disconnect | Dashboard shows `disconnected — reconnecting in 3s` and auto-retries |
| Thermal governor unavailable | TUI falls back to `ThermalLevel::Green` and continues polling |

See [`status-and-recovery.md`](./status-and-recovery.md) for FR-004 health surfaces and error-recovery patterns.

## Related docs

- [CLI/TUI checklist](./cli-tui-checklist.md) — pre-merge operator checklist (C01 L17)
- [Color contrast ratios](./contrast.md) — Backbone-2 tokens (`assets/tokens.css`, `src/theme.rs`)
- [High contrast / forced-colors](./high-contrast.md) — `prefers-contrast` posture, token pairs, dashboard hex audit (C10 L104/L105)
- [Keyboard bindings](./keyboard.md) — TUI quit keys and dashboard focus order
- [Design system](./design-system.md) — tokens, dashboard/tray/TUI components, terminology (C09 L81.8)
- [Responsive layout](./responsive.md) — TUI compact mode + dashboard 375/768 breakpoints
- [Status & recovery](./status-and-recovery.md) — health endpoints, validation hints, degraded mode

## Screen-reader / assistive-tech checklist

Procedure + automated evidence: [`sr-checklist.md`](./sr-checklist.md) · [`sr-pass-evidence.md`](./sr-pass-evidence.md) (C09 L81.4 / FR-004 NFR).

1. **Dashboard (automated):** axe Level A CI + landmark/SR structure tests (`cargo test --test a11y`); skip link + `aria-live` status/thermal.
2. **Dashboard (manual / soft):** VoiceOver/NVDA — verify `nav` announces connection status; `main` table headers are read in order (still READY for per-release AT).
3. **CLI:** Run `sharecli list --json` and pipe to your tooling; avoid parsing colorized `ps` output.
4. **Tray (macOS):** `accessibilityDescription: "ShareCLI"` on menu-bar icon (`desktop/ShareCLITray`).
