# GHCR publish

Audit-v38 **C06 L56 / L58** — container registry + cosign hard path (T-660).

## Runbook

1. Build: `docker build -f Containerfile -t ghcr.io/kooshapari/sharecli/sharecli:TAG .`
2. Auth: `echo $GITHUB_TOKEN | docker login ghcr.io -u USER --password-stdin`
3. Push: `docker push ghcr.io/kooshapari/sharecli/sharecli:TAG`
4. CI hard path: [`.github/workflows/container-cosign.yml`](../../.github/workflows/container-cosign.yml)
   — keyless `cosign sign` + `cosign attest` + verify (OIDC; no Apple secrets).
5. Consumer verify: `bash scripts/container-cosign-verify.sh <image@digest|tag>`

See [`docs/slsa.md`](../slsa.md) L56 for soft sign-blob vs hard GHCR publish.

## CI posture

| Item | Status |
|------|--------|
| Containerfile USER non-root | Done |
| GHCR push on release / `v*` tags | **Hard** — `container-cosign.yml` |
| Keyless cosign sign + attest + verify | **Hard** — OIDC (`id-token` + `packages:write`) |
| Soft sign-blob on main | `container-cosign-soft.yml` (no registry) |
| SBOM artifact | `sbom.yml` on main |

## Registry permission blocker

If the hard job fails on `docker push` / package create with `GITHUB_TOKEN`:

1. Confirm Actions has `packages: write` (workflow already sets this).
2. Org/package: allow GITHUB_TOKEN to create packages, or pre-create
   `ghcr.io/kooshapari/sharecli/sharecli` with workflow write access.
3. Re-dispatch `Container cosign (hard)` with `skip_push=false` after grant.
4. Temporary evidence-only: `skip_push=true` exercises build + predicate wiring.

Soft goal (L58): release registry default documented; L56 hard cosign is the
sign/attest/verify gate.
