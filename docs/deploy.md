# Deploy surface matrix

Status of every install/run surface sharecli claims. Update this table when a
surface gains proof (release asset, deploy URL, or CI log).

**Finality + OS parity (macOS / Windows / Linux / WSL):** [`deploy/FINALITY.md`](deploy/FINALITY.md).

| Surface | Status | How | Proof / notes |
| ------- | ------ | --- | ------------- |
| crates.io (`cargo install sharecli`) | **shipped** | Publish via `.github/workflows/release.yml` `publish` job | Package version tracks `Cargo.toml` (`0.3.0`) |
| cargo-binstall | **configured** | `[package.metadata.dist]` in `Cargo.toml` | Targets: `aarch64-apple-darwin`, `x86_64-unknown-linux-gnu` |
| GitHub Releases (prebuilt binaries) | **ready** | Tag `v*` → `release.yml` `github-release` job | Attaches **UNSIGNED** CLI + tray archives + `.sha256` (+ SBOM). Not notarized (L112 open). |
| Homebrew (`Formula/sharecli.rb`) | **partial** | Bottle URL still PLACEHOLDER; `brew install --HEAD` builds from git | Fill sha after first tagged attach of darwin tarball |
| OpenAPI (`docs/openapi/serve.yaml`) | **gated** | All `serve` Axum routes; drift CI via `scripts/check-openapi-drift.py` | Mirrors `sharecli serve` HTTP surface |
| SBOM (CycloneDX) | **shipped** | `sbom.yml` on main + embedded in release tarballs | `sharecli.cdx.json` in-archive + CI artifact |
| OCI container (`Containerfile`) | **ready** | Multi-stage build, non-root `USER sharecli`, `HEALTHCHECK` → `/healthz` | `podman build -f Containerfile -t sharecli .` then `podman run --rm -p 9000:9000 sharecli` |
| Self-hosted / reverse proxy | **documented** | Bind `sharecli serve --bind 0.0.0.0:9000` behind nginx/Caddy | Probe `GET /healthz` and `GET /readyz` (see `docs/ops/SLO.md`) |
| Cross-device fleet | **in progress** | `sharecli fleet register` / `status` over NATS | See `docs/CROSS_DEVICE_DEPLOY.md` |
| CLI — Linux / macOS / Windows | **GA** | Release matrix in `release.yml` (incl. `x86_64-pc-windows-msvc`) | Parity floor lane A — see [`FINALITY.md`](deploy/FINALITY.md) |
| Tray — Linux (SNI) / macOS (Swift) / Windows (WinUI) | **beta** | Native per OS; release + `desktop-builds.yml` | Artifacts `sharecli-tray-*` / `sharecli-desktop-macos-*` (Win tray soft until green) |
| Dashboard UI (all hosts) | **beta** | Web cockpit `http://127.0.0.1:9000/`; macOS also native `DashboardView` | Equal parity; macOS is optimal native peak |
| WSL | **beta (bridged)** | CLI in WSL + Windows tray or WSLg Linux tray | [`FINALITY.md`](deploy/FINALITY.md#wsl-bridge-parity) — equal capability list via bridge |
| Mobile (iOS/Android/PWA) | **N/A** | Deliberate non-goal | ADR: [`docs/adr/0001-no-mobile-app.md`](adr/0001-no-mobile-app.md) |
| Edge / serverless (Workers, Vercel) | **N/A** | CLI + local supervisor, not an edge app | No `wrangler.toml` / `vercel.json` by design |
| Traditional packaged unit (systemd/Caddy sample) | **soft sample** | [`systemd/sharecli.service`](systemd/sharecli.service) + [`caddy/Caddyfile.sample`](caddy/Caddyfile.sample) | Optional self-host; not a packaged distro unit |

## Signing

Soft runbook: [`ops/codesign-notarize.md`](ops/codesign-notarize.md) (L112). Release assets remain unsigned until org secrets land.

## Auto-update (soft)

Operator channels: [`ops/auto-update.md`](ops/auto-update.md) (C11 L111). Prefer crates.io / binstall / brew / checksummed Releases — no in-binary `self-update` yet.

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
