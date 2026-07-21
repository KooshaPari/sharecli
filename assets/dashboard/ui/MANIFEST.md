# Phenotype UI Pack — 2026-07-21

Generated with **ImageMagick 7.1 + FFmpeg 8.1** (no Blender).

## Brand tokens (Keycap Palette)

| Token | Hex | Usage |
|-------|-----|-------|
| Teal (primary) | `#7ebab5` | Headings, accents, CTAs |
| Midnight (secondary) | `#090a0c` | Backgrounds, base geometry |
| Frost (accent) | `#e8f4f2` | Body text on dark surfaces |

Source: `orchestrator/tokens.json`

## Generated assets

### Icons & favicons
- `icons/phenotype_icon.png` — 512×512 base icon (ImageMagick gradient tile)
- `favicons/phenotype_16.png` — 16×16
- `favicons/phenotype_32.png` — 32×32
- `favicons/phenotype_64.png` — 64×64
- `favicons/phenotype_128.png` — 128×128
- `favicons/phenotype.ico` — multi-res ICO bundle

### Banners (3 sizes)
- `banners/og_1200x630.png` — Open Graph / social card
- `banners/hero_1920x600.png` — landing hero
- `banners/dashboard_1280x320.png` — sharecli operator dashboard header

### Empty states (SVG + PNG)
- `empty-states/no-data.{svg,png}` — "No data yet"
- `empty-states/no-results.{svg,png}` — "No results"
- `empty-states/error.{svg,png}` — "Something went wrong"

### Video (FFmpeg)
- `video/brand_intro.mp4` — 3s 1920×1080 brand intro
- `video/brand_intro.gif` — 10fps preview GIF

## Entrypoints used

| Script | Role |
|--------|------|
| `imagemagick/batch_ui_pack.sh` | Master batch (this pack) |
| `imagemagick/generate_base_icon.sh` | Base icon without Blender |
| `imagemagick/favicon_multi.sh` | Multi-res favicon + ICO |
| `imagemagick/social_card.sh` | 1200×630 OG card |
| `imagemagick/feature_banner.sh` | 1920×600 hero |
| `imagemagick/dashboard_banner.sh` | 1280×320 dashboard header |
| `imagemagick/empty_state.sh` | Empty-state SVG + PNG |
| `ffmpeg/brand_intro.sh` | Brand intro animation |
| `orchestrator/driver.py` | Manifest dispatcher (optional) |
| `orchestrator/manifest_ui.json` | UI-only manifest |

## Mirror copy (sharecli dashboard)

Synced to:
`repos/worktrees/sharecli/feat-dashboard-ws-operator-envelope/assets/ui-pack-2026-07-21/`

## Still requires Blender / other legs

| Asset type | Tool | Script |
|------------|------|--------|
| Glass 3D app icons | Blender | `blender/glass_icon.py` |
| Volumetric hero renders | Blender | `blender/hero.py` |
| Batch icon/hero renders | Blender | `blender/render_all.sh` |
| UE5 cinematics | Unreal | `unreal/render_cinematic.sh` |
| Photoshop/Illustrator exports | Adobe CC | `adobe/` (stub) |

## Re-run

```bash
cd repos/asset-engine
chmod +x imagemagick/*.sh ffmpeg/*.sh
./imagemagick/batch_ui_pack.sh out/ui-pack-2026-07-21
```
