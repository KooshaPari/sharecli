# Cryptography & key management (soft)

Audit-v38 **C02 L22**. sharecli is a local supervisor — **not** a KMS.

## Product stance

| Surface | Policy |
|---------|--------|
| Serve Bearer token | Opaque shared secret; compared via SHA-256 digest (`serve_auth`) |
| Release attestations | SLSA / cosign path (see `docs/slsa.md`) — not app-level crypto |
| Utility modules under `src/` (`xxtea`, `hkdf`, …) | **Non-product** helpers; do not use for new secrets without an ADR |
| KMS / sealed secrets / hardware keys | Out of scope until a networked multi-tenant mode exists |

## Operator guidance

1. Prefer `SHARECLI_SERVE_TOKEN` from a secret store / OS keychain, not a committed file.
2. Rotate the token by restarting `serve` with a new env value.
3. Do not expose `/` or auth-gated routes beyond loopback without TLS at a reverse proxy.

## Soft follow-up

| Item | Status |
|------|--------|
| Document non-product crypto utils | Done (this file) |
| Remove or feature-gate toy crypto modules from default builds | Deferred |
| Integrate OS keyring for serve token | Deferred |

See also: [`privacy-tenant.md`](privacy-tenant.md) (L24).
