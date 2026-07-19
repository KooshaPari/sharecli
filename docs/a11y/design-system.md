# Design system — sharecli surfaces

**C09 L81.8** — single source for UI vocabulary, tokens, and reusable components across the web dashboard, macOS tray, and thermal TUI.

## Token source of truth

| Layer | Path | Role |
|-------|------|------|
| CSS custom properties | `assets/tokens.css` | Web + docs; Backbone-2 palette, type roles, motion |
| Rust mirror | `src/theme.rs` | CLI/TUI ANSI theming; must stay aligned with tokens.css |
| Typography / motion | `docs/visual/typography.md`, `docs/visual/motion.md` | Human-readable token contracts |

Do not introduce one-off hex values in new UI. Extend `tokens.css` and mirror in `theme.rs` when adding a semantic color.

## Web dashboard components (`src/dashboard.html`)

Read-only status cockpit served by `sharecli serve`. No ad-hoc controls — reuse these patterns:

| Component | Markup / id | Purpose |
|-----------|-------------|---------|
| Skip link | `.skip-link` → `#main-content` | First Tab stop; keyboard entry to data table |
| Status nav | `<nav aria-label="Dashboard status">` | Connection dot, title, live status, thermal badge |
| Connection dot | `#status-dot` | Visual only (`aria-hidden`); text in `#status-label` |
| Live status | `#status-label` | `aria-live="polite"` connection string |
| Thermal badge | `#thermal-status` | `aria-live="polite"` governor level (text + emoji) |
| Process table | `table[aria-label="Managed processes"]` | `scope="col"` headers; no row actions |
| Last update | `#last-update` | `aria-live="polite"` refresh timestamp |

Focus management: skip link uses `:focus` outline (`2px solid`); `#main-content` has `tabindex="-1"` for skip-target focus.

## macOS tray (`desktop/ShareCLITray`)

| Element | Contract |
|---------|----------|
| Menu bar icon | `accessibilityDescription: "ShareCLI"` |
| Actions | Mirror CLI verbs (`list`, `stop`, `serve`) — same terminology as `sharecli --help` |

Tray does not embed dashboard HTML; it delegates to CLI subprocesses.

## Thermal TUI (`crates/sharecli-thermal-tui`)

| Concept | Vocabulary | Keys |
|---------|------------|------|
| Thermal level | `GREEN` / `YELLOW` / `RED` text labels (not color alone) | — |
| Quit | Documented in footer | `q`, `Ctrl-C` via `is_quit_key` |
| Compact layout | `COMPACT_WIDTH = 80` columns | Resize reflow |

Keybinding matrix: [`keyboard.md`](./keyboard.md).

## Terminology (CLI-wide)

Use these terms consistently in help text, errors, and UI copy:

| Concept | Preferred term | Avoid for same action |
|---------|----------------|----------------------|
| End a managed process | **stop** (`sharecli stop`) | kill (except `--force` SIGKILL path) |
| Remove idle processes | **prune** | delete, clean |
| Remove CLI from system | **uninstall** | remove (package managers may say remove) |
| Process group | **project** (`sharecli project …`) | group, fleet (fleet is supervisor scope) |
| Screen share target | **cast** | stream, mirror |

Destructive paths (`stop --force`, `prune --force`) require `--yes` or show a dry-run preview first (L81.6).

## Automated consistency checks

| Check | Command / path |
|-------|----------------|
| Landmarks + skip link | `cargo test -p sharecli --test a11y` |
| axe WCAG 2.x Level A | `npm run a11y:dashboard` |
| Keyboard Tab-cycle | `npm run a11y:keyboard` (requires `sharecli serve`) |
| Contrast pairs | `docs/a11y/contrast.md` |

## Related

- [`README.md`](./README.md) — accessibility hub
- [`keyboard.md`](./keyboard.md) — focus order and TUI bindings
- [`contrast.md`](./contrast.md) — token contrast ratios
