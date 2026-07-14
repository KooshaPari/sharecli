# Typography — sharecli

Soft type-role scale for C10 L97. CLI/TUI remain monospace-first; dashboard HTML inherits the same roles.

| Role | CSS intent | Approx size | Use |
|------|------------|-------------|-----|
| display | `--type-display` | clamp(1.75rem, 4vw, 2.5rem) | Splash / brand lockup |
| heading | `--type-heading` | 1.25–1.5rem | Section titles |
| body | `--type-body` | 0.95–1rem | Prose / table cells |
| caption | `--type-caption` | 0.75–0.85rem | Hints, timestamps, muted meta |

## Stack

```css
font-family: "JetBrains Mono", "Fira Code", "Cascadia Code", ui-monospace, monospace;
```

## Rationale

Operator surfaces are log- and process-oriented. A mono stack preserves alignment for columns and status tables; display/heading roles add hierarchy without switching to a second brand face. Light theme / variable-font axes remain a follow-up.
