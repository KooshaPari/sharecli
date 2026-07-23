# Visual asset provenance — sharecli

Tier legend (audit-v38 C10):

| Tier | Meaning |
|------|---------|
| 1 | Hand-authored / AI-coded in-repo; license = repo |
| 2 | Derived export from tier-1 SoT |
| 3 | Third-party or generative; must declare source/license |

| Asset | Tier | Source / notes |
|-------|:----:|----------------|
| `assets/tokens.css` | 1 | Backbone-2 token SoT |
| `src/theme.rs` | 1 | Rust mirror of tokens |
| `assets/brand/sharecli-icon.svg` | 1 | Hand-coded hexagonal chevron |
| `assets/brand/sharecli-icon-animated.svg` | 1 | SMIL motion variant of mark |
| `assets/brand/README.md` | 1 | Brand narrative |
| `assets/icons/sharecli.iconset/*` | 2 | Raster ladder from brand SVG |
| `assets/icons/sharecli.icns` | 2 | macOS pack from iconset |
| `docs/assets/identity/demo.svg` | 1 | Heartbeat / ECG demo |
| `docs/assets/identity/demo.mp4` | 2 | Render of demo.svg |
| `assets/dashboard/ui/error-states/disconnect.svg` | 1 | Hand-authored L101 disconnect scene (serve↔dashboard severed feed); Backbone-2 tokens |
| `assets/dashboard/ui/empty-states/*` | 3 | Phenotype UI pack (Keycap palette); not used for disconnect panel |
| Dashboard mono fonts (CDN/system) | 3 | JetBrains/Fira/Cascadia — system/CDN fallbacks only |

No tier-3 generative image packs are shipped as primary brand marks. The disconnect error panel uses the tier-1 `disconnect.svg` only.
