# Keyboard navigation

## Thermal TUI (`sharecli thermal` / governor view)

| Key | Action | Implementation |
|-----|--------|----------------|
| `q` | Quit TUI, restore terminal | `sharecli-thermal-tui` `is_quit_key` |
| `Ctrl-C` | Quit TUI (same as `q`) | `sharecli-thermal-tui` `is_quit_key` |

The footer documents both bindings. No mouse input is required.

Unit tests: `crates/sharecli-thermal-tui/src/lib.rs` — `test_is_quit_key_*`.

## Web dashboard (`sharecli serve` → `/`)

The dashboard is read-only (no form controls). Keyboard users can:

1. **Tab** through the skip link → status region → process table.
2. **Skip link** (`Skip to process table`) jumps focus to `#main-content`.

Landmarks: `<nav aria-label="Dashboard status">` wraps the header; `<main id="main-content">` wraps the data table.

Automated check: `tests/a11y/dashboard_landmarks.rs`.

## CLI (non-interactive)

All subcommands are invocable without a pointing device. Shell completions (`sharecli completions bash`) support keyboard-driven workflows in bash/zsh/fish/powershell.
