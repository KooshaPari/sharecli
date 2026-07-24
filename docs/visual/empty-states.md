# Empty and zero-data states (C10 L100)

Designed empty states explain what is missing and the next command — not a blank table or silent CLI exit.

## CLI (`sharecli ps`)

| Situation | Copy pattern |
|-----------|----------------|
| Idle pool (first run) | `No managed processes yet. Get started: sharecli start <project> <harness>` + `sharecli serve` |
| Filtered by `--project` | `No processes match project '<name>'. Try: sharecli start <name> <harness>` |
| Filtered by `--harness` | `No processes match harness '<name>'. Try: sharecli start <project> <name>` |

Implementation: `print_ps_empty_hint` in `src/commands/mod.rs`. Regression: `tests/c10_l100_empty_states.rs`.

## Dashboard (`src/dashboard.html`)

| Kind | When | Title | CTA |
|------|------|-------|-----|
| `first-run` | Connected, snapshot has zero processes, never had rows | No processes yet | `sharecli start <project> <harness>` |
| `cleared` | Snapshot empty after at least one process was shown | All processes stopped | `sharecli ps` |

Visual fixture (`SHARECLI_VISUAL_FIXTURE=1`) waits for `data-empty-kind="first-run"` after WebSocket connect (300ms idle timeout) so golden PNGs capture the first-run panel.

## Acceptance

- Icon + one-line explanation + primary CTA on dashboard empty panel
- First-run vs filtered/cleared copy differs between CLI and dashboard branches
- See also [VISUAL_SPEC.md](VISUAL_SPEC.md) §3
