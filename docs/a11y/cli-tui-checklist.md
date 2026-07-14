# CLI / TUI accessibility checklist

Operator checklist for audit-v38 **C01 L17** (terminals + thermal TUI).
Dashboard WCAG Detail: [`README.md`](README.md), [`contrast.md`](contrast.md).

## Always

| Check | How |
|-------|-----|
| Color not sole signal | Thermal labels GREEN/YELLOW/RED; status columns are text |
| `NO_COLOR` / `TERM=dumb` | Output remains readable; use `--json` where available |
| Keyboard-only TUI | `q` / `Ctrl-C` exit; no mouse required ([`keyboard.md`](keyboard.md)) |
| Structured list output | `sharecli ps` headers; `sharecli list --json` |
| Health / recovery copy | Plain-language errors; [`status-and-recovery.md`](status-and-recovery.md) |

## Dashboard (when editing `src/dashboard.html`)

| Check | How |
|-------|-----|
| Landmarks + `lang` | `cargo test --test dashboard_landmarks` (or a11y suite) |
| axe Level A | `npm run a11y:dashboard` / `.github/workflows/a11y.yml` |
| Reduced motion | `prefers-reduced-motion` in dashboard CSS |

## Sign-off

Before merging TUI/CLI UX changes: tick the Always table; before dashboard HTML merges: tick Dashboard table.
