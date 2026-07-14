# Responsive & adaptive layout (L81.11)

## Thermal TUI

| Width | Mode | Behavior |
|------:|------|----------|
| ≥ 80 cols | full | Margins + long pressure/gate copy |
| < 80 cols | compact | Zero margin, shortened labels |

Implementation:

- `is_compact(width)` / `COMPACT_WIDTH` in `crates/sharecli-thermal-tui`
- `render` reads `frame.area().width` (ratatui surface of `crossterm::terminal::size` / `COLUMNS`)
- `Event::Resize` accepted in the event loop so layout reflows on window resize

## Web dashboard

| Breakpoint | CSS |
|-----------:|-----|
| default / ≥1280 | base styles in `src/dashboard.html` |
| ≤768 | tighter padding; wrap header; allow cell wrap |
| ≤375 | phone padding + type scale |

CI smoke: `tests/a11y/dashboard_landmarks.rs` asserts viewport meta + both `@media` queries exist.
