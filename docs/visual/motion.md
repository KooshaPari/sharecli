# Motion — sharecli

Backbone-2 motion tokens live in `assets/tokens.css`:

| Token | Default | Reduced |
|-------|---------|---------|
| `--motion-duration-fast` | 150ms | 0ms |
| `--motion-duration` | 300ms | 0ms |
| `--motion-ease` | `cubic-bezier(0.4, 0, 0.2, 1)` | unchanged |

## `prefers-reduced-motion`

When the OS requests reduced motion:

- Token durations collapse to `0ms` in `:root`
- Dashboard (`src/dashboard.html`) disables non-essential transitions and the thermal critical pulse

Brand SMIL demos (`assets/brand/sharecli-icon-animated.svg`) remain optional decorative assets — operators embedding them should gate with the same media query.
