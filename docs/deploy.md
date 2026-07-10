# Deploy surface matrix

Status of every install/run surface sharecli claims. Update this table when a
surface gains proof (release asset, deploy URL, or CI log).

| Surface | Status | How | Proof / notes |
| ------- | ------ | --- | ------------- |
| crates.io (`cargo install sharecli`) | **shipped** | Publish via `.github/workflows/release.yml` `publish` job | Package version tracks `Cargo.toml` (`0.3.0`) |
| cargo-binstall | **configured** | `[package.metadata.dist]` in `Cargo.toml` | Targets: `aarch64-apple-darwin`, `x86_64-unknown-linux-gnu` |
| GitHub Releases (prebuilt binaries) | **partial** | Tag `v*` → release workflow artifact matrix | Releases `v0.1.0`–`v0.3.0` exist; **no assets attached yet** — matrix in `release.yml` builds linux+mac tarballs for future tags |
| Homebrew (`Formula/sharecli.rb`) | **stub** | Formula version `0.3.0` matches Cargo.toml | `sha256` is `PLACEHOLDER` until a darwin tarball is published (see formula header) |
| OCI container (`Containerfile`) | **ready** | Multi-stage build, non-root `USER sharecli`, `HEALTHCHECK` → `/healthz` | `podman build -f Containerfile -t sharecli .` then `podman run --rm -p 9000:9000 sharecli` |
| Self-hosted / reverse proxy | **documented** | Bind `sharecli serve --bind 0.0.0.0:9000` behind nginx/Caddy | Probe `GET /healthz` and `GET /readyz` (see `docs/ops/SLO.md`) |
| Cross-device fleet | **in progress** | `sharecli fleet register` / `status` over NATS | See `docs/CROSS_DEVICE_DEPLOY.md` |
| Desktop tray (Linux/macOS/Windows) | **partial** | Workspace crates + release header intent | Tray clients build in-tree; attach to release archives when matrix publishes |
| Mobile (iOS/Android/PWA) | **N/A** | Deliberate non-goal | ADR: [`docs/adr/0001-no-mobile-app.md`](adr/0001-no-mobile-app.md) |
| Edge / serverless (Workers, Vercel) | **N/A** | CLI + local supervisor, not an edge app | No `wrangler.toml` / `vercel.json` by design |

## Quick container smoke

```bash
podman build -f Containerfile -t sharecli:local .
podman run --rm -p 9000:9000 sharecli:local
curl -fsS http://127.0.0.1:9000/healthz
```

## Homebrew PLACEHOLDER removal

When the next release attaches `sharecli-aarch64-apple-darwin.tar.gz`:

```bash
gh release download vX.Y.Z -p 'sharecli-aarch64-apple-darwin.tar.gz'
shasum -a 256 sharecli-aarch64-apple-darwin.tar.gz
# paste into Formula/sharecli.rb sha256, bump version + url
```
