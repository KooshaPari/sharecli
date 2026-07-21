# Keyboard navigation

## Thermal TUI (`sharecli thermal` / governor view)

| Key | Action | Implementation |
|-----|--------|----------------|
| `q` | Quit TUI, restore terminal | `sharecli-thermal-tui` `is_quit_key` |
| `Ctrl-C` | Quit TUI (same as `q`) | `sharecli-thermal-tui` `is_quit_key` |
| `Tab` | Cycle focus forward: gate → pool → status → host watch → agents | `handle_key` → `KeyAction::FocusNext` |
| `Shift-Tab` | Cycle focus backward | `handle_key` → `KeyAction::FocusPrev` |
| `1`–`5` | Jump focus to gate / pool / status / host watch / agents panel | `handle_key` → `KeyAction::FocusPanel` |
| `r` | Force immediate governor + operator poll | `handle_key` → `KeyAction::ForcePoll` |
| `?` | Toggle footer help overlay | `handle_key` → `KeyAction::ToggleHelp` |

The footer documents quit bindings and `? help`. When help is open, the overlay lists the full matrix. Focused panels show a cyan bold border. No mouse input is required.

Unit tests: `crates/sharecli-thermal-tui/src/lib.rs` — `test_handle_key_*`, `test_panel_focus_*`; FR gate: `tests/c09_l81_keyboard_design_system.rs` (`fr004_l81_3_thermal_tui_keyboard_matrix`).

## Web dashboard (`sharecli serve` → `/`)

The dashboard is read-only (no form controls). Keyboard users can:

1. **Tab** through the skip link → status region → process table.
2. **Skip link** (`Skip to process table`) jumps focus to `#main-content`.

Landmarks: `<nav aria-label="Dashboard status">` wraps the header; `<main id="main-content" tabindex="-1">` wraps the data table (skip-link target).

Automated checks:

- **Playwright Tab-cycle (CI via `.github/workflows/a11y.yml`):** `scripts/a11y/playwright_keyboard.mjs` — first Tab focuses skip link; Enter moves focus to `#main-content`; visible focus outline.
- **Rust gate:** `tests/c09_l81_keyboard_design_system.rs` (FR-004 · T-651).

Run locally:

```bash
cargo build --release -p sharecli
./target/release/sharecli serve --bind 127.0.0.1:9000 &
SHARECLI_VISUAL_FIXTURE=1 npm run a11y:keyboard
```

## CLI (non-interactive)

All subcommands are invocable without a pointing device. Shell completions (`sharecli completions bash`) support keyboard-driven workflows in bash/zsh/fish/powershell.
