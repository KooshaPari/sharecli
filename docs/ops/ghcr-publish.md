# GHCR publish (soft)

Audit-v38 **C06 L58** — container registry default.

## Runbook

1. Build: `docker build -f Containerfile -t ghcr.io/KooshaPari/sharecli:TAG .`
2. Auth: `echo $GITHUB_TOKEN | docker login ghcr.io -u USER --password-stdin`
3. Push: `docker push ghcr.io/KooshaPari/sharecli:TAG`
4. Release workflow attaches SBOM (`sbom.yml`) + cosign soft evidence.

## CI posture

| Item | Status |
|------|--------|
| Containerfile USER non-root | Done |
| GHCR push on release tag | Soft — manual until L58 hard gate |
| SBOM artifact | `sbom.yml` on main |

Soft goal: L58 **1→2** when release docs + sample `ghcr.io` tag documented (this file).
