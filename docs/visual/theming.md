# Theming — sharecli

| Name | CLI aliases | Surfaces |
|------|-------------|----------|
| Backbone-2 (dark) | `backbone-2`, `bb2`, `dark` | Default CLI / dashboard graphite |
| Backbone-2 Light | `backbone-2-light`, `bb2-light`, `light` | Paper ground + darker accents |

CSS: `assets/tokens.css` (`:root` dark; `[data-theme=light]` + `prefers-color-scheme: light`).
Rust: `src/theme.rs` (`Tokens::BACKBONE2` / `Tokens::BACKBONE2_LIGHT`).

```bash
sharecli --theme light version
```

High-contrast variant remains a follow-up.
