# Cryptography & key management

Audit-v38 **C02 L22**. sharecli is a local process supervisor — **not** a KMS.

## Threat surface

The product surface that ever touches keys or secrets is intentionally small:

| Surface | Where | What kind of secret | How it's compared |
|---------|-------|---------------------|-------------------|
| `serve` Bearer token | `SHARECLI_SERVE_TOKEN` env | Opaque shared secret (string) | SHA-256 digest + length-checked equality (`src/serve_auth.rs:326-327`) |
| `serve` JWT validation | `SHARECLI_SERVE_JWKS_URL` env + cached JWKS document | RS256 public key | Verified via `jsonwebtoken` (audited crate) |
| Audit log path | `SHARECLI_AUDIT_PATH` | None | Plain JSONL append-only file |
| `sharecli history` JSONL | `$XDG_STATE_HOME/sharecli/history.jsonl` | None | Plain JSONL append-only file |
| `sharecli upgrade` probe | none | none | no network egress (Plan 783 / T-880) |

Everything else uses no secrets.

## Key lifecycle (operator contract)

1. **Provisioning**: Bearer tokens come from a secret store (Vault, AWS Secrets Manager, OS keyring). Never commit them.
2. **Storage at rest**: sharecli does **not** persist Bearer tokens or JWT keys. It only persists:
   - Audit JSONL (no secrets)
   - History JSONL (no secrets)
   - Configuration (no secrets; see `docs/ops/secrets.md` redaction rule)
3. **Rotation**: Restart `serve` with a new `SHARECLI_SERVE_TOKEN` (or rotate the JWT issuer's signing key and update `SHARECLI_SERVE_JWKS_URL`). No persistence path means no stale token risk.
4. **Disposal**: Delete `SHARECLI_AUDIT_PATH` and `sharecli/history.jsonl` files via standard filesystem unlink (no key shredding required; no secrets were stored).

## Algorithm inventory

### Product crypto (audited)

- **SHA-256** for Bearer token digest via `sha2` crate (length-checked equality)
- **RS256** for JWT validation via `jsonwebtoken` (audited crate, depends on `ring`/`aws-lc-rs`)
- **TLS** at the reverse-proxy layer (out-of-scope for sharecli; `nginx`/`caddy` recommended)

### Utility crypto (non-product)

The following modules under `sharecli::util` are **non-product** helpers — they ship for parity but must not be used for new secrets without an ADR:

| Module | Path | Status |
|--------|------|--------|
| `xxtea` | `src/xxtea.rs` via `src/util/mod.rs` | Toy cipher; not used in product paths. |
| `hkdf` | `src/hkdf.rs` via `src/util/mod.rs` | HKDF-SHA256 helper; only used in tests. |
| `chacha20` | `src/chacha20.rs` via `src/util/mod.rs` | Pure-Rust ChaCha20 stream cipher; not used in product paths. |
| `x509_chain` | `src/x509_chain.rs` via `src/util/mod.rs` | ASN.1 DER parser; parses but does not verify trust. |
| `pem_decode` | `src/pem_decode.rs` via `src/util/mod.rs` | PEM armoring parser. |

These remain under `sharecli::util` (Tier C facade, behind `pub mod util` re-export only). New consumers of these modules require an ADR that names an audited alternative and a migration path.

## KMS / sealed secrets / hardware keys — explicitly out of scope

Per the C02 L22 rubric gap ("prefer audited crates for any real crypto; document key lifecycle or remove toy crypto from public CLI"):

- **AWS KMS / GCP KMS / Azure Key Vault**: Not used. sharecli is local.
- **HashiCorp Vault transit/secret engines**: Not used. sharecli is local.
- **Sealed Secrets / age / sops**: Not used. sharecli is local.
- **Hardware keys (YubiKey HSM, TPM, Apple Secure Enclave)**: Not used. sharecli is local.

If sharecli ever moves to a networked/multi-tenant deployment, this section becomes the highest-priority rework item.

## Cross-references

- [`docs/ops/secrets.md`](secrets.md) — Bearer/JWT runtime contract (no static secrets in repo).
- [`docs/ops/privacy-tenant.md`](privacy-tenant.md) — single-operator local tenancy policy (C02 L24).
- [`docs/ops/AUTH.md`](AUTH.md) — federated JWT resource-server mode.
- [`THREAT_MODEL.md`](../../THREAT_MODEL.md) — STRIDE-per-component including Crypto tampered-with / repudiation paths.
- [`SECURITY.md`](../../SECURITY.md) — vulnerability disclosure + signed-release contact.

## Soft follow-up (deferred — same as Plan 802 predecessor)

| Item | Status |
|------|--------|
| Document key lifecycle (this file) | **DONE** (Wave17 Plan 802) |
| Document algorithm inventory (this file) | **DONE** (Wave17 Plan 802) |
| Cross-reference THREAT_MODEL.md / AUTH.md / secrets.md | **DONE** (Wave17 Plan 802) |
| Replace toy crypto with audited alternatives if any product path is ever identified | Deferred (no product path uses these utils today) |
| Integrate OS keyring for serve token | Deferred (no platform-agnostic library) |