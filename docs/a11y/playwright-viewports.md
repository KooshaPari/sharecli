# Playwright viewport soft gate

Audit-v38 **C09 L81.11**. Soft — `continue-on-error`.

## Matrix

| Viewport | Width | Surface |
|----------|------:|---------|
| mobile | 375 | `src/dashboard.html` |
| tablet | 768 | same |
| desktop | 1280 | same |

Breakpoints already in CSS (`@media` 768 / 375). TUI compact width = 80 cols.

## Soft CI

`.github/workflows/playwright-soft.yml` boots `sharecli serve`, installs Playwright
Chromium, captures PNGs for the three widths, uploads artifacts. Failures do not
block merge.

## Local

```bash
cargo build --release -p sharecli
npx --yes playwright@1.49.0 install chromium
# then run the workflow script steps, or:
just playwright-soft   # when recipe present
```
