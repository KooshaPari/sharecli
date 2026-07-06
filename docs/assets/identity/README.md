# sharecli — Identity Demo Media (L105)

Animated SVG + MP4 showcasing the [Backbone-2 pulse-green + warm-amber palette](../../assets/tokens.css) in motion.

## Files

| File | Purpose |
|---|---|
| `demo.svg` | 480×270 animated SVG — process heartbeat + ECG trace + cooldown flash (looped CSS animation, ~5s) |
| `demo.mp4` | H.264/MP4 rendered from `demo.svg` via playwright + ffmpeg (24fps, 5s loop) |

## Palette (Backbone-2 — pulse-green + warm-amber)

- Outer background `#0a0d12`
- Inset panel `#161b22`
- Pulse-green `#3fb950` (dominant — process heartbeat)
- Warm-amber `#d29922` (cooldown flash — single hot pixel)

## Animation

- Core pulse: 1s cubic-bezier heartbeat (faster than AgilePlus — process tempo)
- ECG trace: static SVG path under the pulse for monitor vibe
- Cooldown flash: 4s ease-in-out amber pixel + label, brief at 80–95%

## Render command

```sh
python /tmp/svg2mp4.py demo.svg demo.mp4 480 270 24 5
```

## Source of truth

- Tokens: [`../../assets/tokens.css`](../../assets/tokens.css)
- Source icon: [`../../assets/icons/sharecli-512x512.png`](../../assets/icons/sharecli-512x512.png)
- Scorecard: `.claude/audit/.vision/L96-L107.md`