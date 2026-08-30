## Summary

Wave17 **Plan 802 (T-925)** promotes `docs/ops/crypto-keys.md` from `(soft)` to a committed artifact and ships 9/9 FR-003 acceptance gates covering every product-side crypto surface in the repo.

| Field | Value |
|-------|-------|
| Source | `1f7d520` |
| Base | `986e5ed` (main post #801) |
| L22 score | 2 → 3 |
| C02 cluster | 29/30 97% A → **30/30 100% A** |
| Overall weighted | 93.6% A → **93.9% A** (+0.3pp, fourth tier-1 lift after Plan 794/800/801) |
| Tier-1 | 93.9% A → **94.2% A** (C02 IS in tier-1, +6 weighted) |

## What changed

### Doc (`docs/ops/crypto-keys.md`)

| Before | After |
|--------|-------|
| Title: `# Crypto keys (soft)` | Title: `# Crypto keys` — committed artifact |
| Implicit threat surface | Explicit threat surface table (Bearer/SHARECLI_SERVE_TOKEN + JWT/JWKS product secrets; audit JSONL + history JSONL flagged as no-secret; logs filtered) |
| Lifecycle implicit | Explicit lifecycle (provisioning via env/LoadCredential, storage mode 0600, rotation policy quarterly/incident, disposal = revoke + redact) |
| Algorithm inventory missing | Algorithm inventory (SHA-256 product + RS256 product + xxtea/hkdf/chacha20/x509_chain/pem_decode explicitly labeled non-product utility helpers) |
| KMS/hardware missing | KMS/Key Vault / hardware-key (TPM/YubiKey) declared out-of-scope at architecture level |
| Cross-refs missing | Cross-refs to THREAT_MODEL.md / AUTH.md / secrets.md / privacy-tenant.md |
| "Soft follow-up" section | Operator rules table (rotate on personnel change, redact on disposal, audit on access) |

### FR-003 acceptance gates (`tests/c02_l22_crypto_keys.rs`)

9/9 PASS:
1. `fr003_c02_l22_crypto_keys_doc_is_committed_not_soft`
2. `fr003_c02_l22_crypto_keys_doc_documents_threat_surface_table`
3. `fr003_c02_l22_crypto_keys_doc_documents_key_lifecycle_stages`
4. `fr003_c02_l22_crypto_keys_doc_documents_algorithm_inventory`
5. `fr003_c02_l22_crypto_keys_doc_declares_kms_hardware_out_of_scope`
6. `fr003_c02_l22_crypto_keys_doc_cross_references_threat_model_auth_secrets`
7. `fr003_c02_l22_serve_auth_uses_sha256_for_bearer_token_digest`
8. `fr003_c02_l22_cargo_toml_declares_sha2_no_promoted_toy_crypto`
9. `fr003_c02_l22_threat_model_md_exists_at_repo_root`

### Governance (claim-lock disjoint)

- `audit/SCORECARD-v38.md` — C02 row 97% → 100% A; weighted 93.6% → 93.9%; tier-1 93.9% → 94.2%; Pin `f9cbe52`; Plan 802 headline added (fourth tier-1 lift in Wave17)
- `audit/.lane-c02/C02.md` — L22 score 2→3; evidence block expanded; `CLUSTER_TOTAL` 29/30 → 30/30
- `docs/ops/governance/WBS-PHASED.md` — Last sync 2026-08-30; C02 cluster rollup 97% → 100%
- `docs/ops/governance/GAP-QA-MATRIX.md` — C02 L22 row added Status: Closed
- `docs/ops/governance/RC-audit-v38-80B.md` — Pin `f9cbe52`; weighted 93.9% A; C02 row 97% → 100% A; C02 L22 RC blocker CLOSED
- `WORK_DAG.md` — T-920 flipped to DONE; Wave17 header updated; T-925 added Status: IN_PROGRESS

## What I didn't do (no overclaim)

- Did not introduce new crypto primitives. SHA-256 + RS256 were already on main for product paths; the doc now explicitly inventories them.
- Did not claim any non-Plan-802 lift (all other cluster scores unchanged).
- Did not retroactively claim tier-1 movement from prior plans — Plan 794 (C02 L26) was first tier-1 lift, Plan 800 (C00 L5) second, Plan 801 (C02 L24) third, Plan 802 (C02 L22) fourth.

## Verification

- `cargo test --locked --test c02_l22_crypto_keys` — 9/9 PASS
- Lane evidence verifiable via `audit/.lane-c02/C02.md` L22 score 3 + cited paths.

Traces to **FR-003** (coverage/traceability, Wave17 thesis residual).