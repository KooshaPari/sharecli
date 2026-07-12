# Deploy surface matrix

Status of every install/run surface sharecli claims. Update this table when a
surface gains proof (release asset, deploy URL, or CI log).

| Surface | Status | How | Proof / notes |
| ------- | ------ | --- | ------------- |
| crates.io (`cargo install sharecli`) | **shipped** | Publish via `.github/workflows/release.yml` `publish` job | Package version tracks `Cargo.toml` (`0.3.0`) |
| cargo-binstall | **configured** | `[package.metadata.dist]` in `Cargo.toml` | Targets: `aarch64-apple-darwin`, `x86_64-unknown-linux-gnu` |
| GitHub Releases (prebuilt binaries) | **ready** | Tag `v*` → `release.yml` `github-release` job | Attaches **UNSIGNED** `sharecli-*.tar.gz` + `.sha256` (+ SBOM in-archive). Not notarized (L112 open). |
| Homebrew (`Formula/sharecli.rb`) | **partial** | Bottle URL still PLACEHOLDER; `brew install --HEAD` builds from git | Fill sha after first tagged attach of darwin tarball |
| OpenAPI (`docs/openapi/serve.yaml`) | **stub** | Committed paths for `/healthz` `/readyz` `/metrics/prometheus` `/config` `/debug/pprof/profile` | Mirrors `sharecli serve` HTTP surface |
| SBOM (CycloneDX) | **shipped** | `sbom.yml` on main + embedded in release tarballs | `sharecli.cdx.json` in-archive + CI artifact |
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

Until release assets exist, build from git:

```bash
brew install --HEAD Formula/sharecli.rb
```

When the next release attaches `sharecli-aarch64-apple-darwin.tar.gz`:

```bash
gh release download vX.Y.Z -p 'sharecli-aarch64-apple-darwin.tar.gz'
shasum -a 256 sharecli-aarch64-apple-darwin.tar.gz
# paste into Formula/sharecli.rb sha256, bump version + url
```

## SBOM + OpenAPI stubs

- CycloneDX SBOM: `.github/workflows/sbom.yml` (push `main` / `workflow_dispatch`) uploads `sharecli-sbom`.
- Serve HTTP contract stub: [`docs/openapi/serve.yaml`](openapi/serve.yaml).
