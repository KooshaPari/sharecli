# Loading and skeleton states (C10 L99)

Designed loading states reserve the final layout and explain progress — not a blank table or generic browser spinner.

## Dashboard (`src/dashboard.html`)

| Phase | Status copy | Body |
|-------|-------------|------|
| WebSocket connecting | `connecting…` | Operator panel value skeletons (`data-loading-kind="panel-value"`) + three table skeleton rows (`data-loading-kind="table-row"`) |
| Connected, awaiting snapshot | `connected — loading processes…` | Same skeletons; `#operator-panels[aria-busy="true"]` and `#proc-body[aria-busy="true"]` |
| Data arrived | `connected` | Real operator panels, process rows, or empty/error panel (skeleton cleared) |

Skeleton bars mirror operator panel `dd` geometry (gate/host_watch/agents) and the five table columns (Name, PID, Memory MB, Project, Status) with Backbone-2 sync-violet shimmer (`skeleton-shimmer`). The connection dot uses `#status-dot.connecting` (violet pulse) until the socket opens.

Reduced motion: shimmer and connecting pulse collapse to static panel-colored bars (see `@media (prefers-reduced-motion: reduce)`).

Implementation: `renderOperatorPanelSkeletons` + `renderSkeletonRows` in `src/dashboard.html`. Regression: `tests/c10_l99_skeleton_states.rs`.

## CLI / TUI

Thermal TUI slot fill uses the ratatui `Gauge` widget with token-aligned colors — action progress, not a data-table skeleton. Dashboard WebSocket loading is the primary L99 surface for this lane.

## Acceptance

- Content-shaped placeholders match operator panel and table column geometry (no layout shift on first snapshot)
- Branded sync-violet motion on skeleton bars and connecting dot
- `aria-busy` on `#operator-panels` and `#proc-body` while skeleton placeholders are shown
- See also [VISUAL_SPEC.md](VISUAL_SPEC.md) §4 and [motion.md](motion.md)
