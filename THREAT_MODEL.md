# Threat Model — sharecli

**Status:** ACTIVE  
**Owners:** maintainers (`@KooshaPari`)  
**Review cadence:** yearly or after any AuthN / serve / spawn surface change  
**Last reviewed:** 2026-07-13 (W5.3 post-federation)  
**Related:** [`SECURITY.md`](SECURITY.md), [`BOUNDARY.md`](BOUNDARY.md), [`docs/ops/AUTH.md`](docs/ops/AUTH.md)

Standalone STRIDE + attack-surface inventory for audit-v38 **L39** / **L20**.

## Trust boundary (summary)

| In scope | Out of scope |
|----------|--------------|
| Local `sharecli serve` HTTP/WS bind | Kernel / hypervisor escape |
| Process spawn / stop / limits | Supply-chain compromise of GitHub Actions runners |
| Config TOML under XDG / known folders | Physical access to unlocked workstation |
| Bearer + JWT/JWKS AuthN when configured | OAuth Authorization Code / SAML SP / unix peer creds |
| Audit JSONL retention + AuthN burn metrics | Multi-tenant AuthZ / RBAC |

See `BOUNDARY.md` for ownership of process-orchestration vs host OS.

## Components

1. **CLI** (`sharecli` binary) — config, pool, project registry, limits  
2. **Serve** — Axum HTTP (`/healthz`, `/readyz`, `/metrics`, `/ws`, `/debug/pprof`)  
3. **Spawn core** — Zig/Rust process spawn + priority  
4. **Notifier** — desktop + webhook dispatch  
5. **Release / install** — tarballs, Homebrew Formula, Containerfile  

## STRIDE matrix

| Component | Spoofing | Tampering | Repudiation | Info disclosure | DoS | Elevation |
|-----------|----------|-----------|-------------|-----------------|-----|-----------|
| CLI config | Low — local FS trust | Medium — malicious TOML paths | Low — audit JSONL optional | Medium — secrets in config.toml | Low | Low |
| Serve HTTP | **High** if open bind without AuthN; **Medium** with bearer/JWT | Medium — request smuggling N/A for simple routes | Medium — mitigate via `SHARECLI_AUDIT_LOG` + rotation | Medium — metrics / config routes | **High** — unbounded clients; AuthN burn alerted | Medium — spawn as user |
| Serve JWT | Medium — stolen access token / wrong audience if misconfigured | Low — JWKS must be RSA/EC (HS* denied) | Low — `auth_ok`/`auth_fail` with `sub` | Low — token never logged | Medium — JWKS file/env misconfig → fail-closed | Low — no privilege beyond serve user |
| Spawn | Medium — command injection via config | High — arbitrary argv from config | Low | Low | **High** — fork bomb / memory | Medium — no root required |
| Notifier webhook | Medium — URL from config | Low | Low | Medium — event payloads | Low | Low |
| Release assets | Medium until signed | Medium — unsigned archives | Low | Low | Low | Low |

## Attack surface inventory (serve)

| Route | Auth | Risk notes |
|-------|------|------------|
| `/healthz`, `/readyz` | Public (all modes) | Keep probe-only; no secrets |
| `/metrics/prometheus` | Bearer / JWT if configured | May leak process names/paths; exposes `unauthorized_total` |
| `/config` | Bearer / JWT if configured | Full config dump — treat as sensitive |
| `/ws` | Bearer / JWT if configured | Live thermal/events |
| `/debug/pprof/profile` | Bearer / JWT + `SHARECLI_PPROF=1` | CPU sample; loopback preferred |

## Mitigations (current)

- Optional Bearer (`SHARECLI_SERVE_TOKEN` / `config.serve.bearer_token`) or JWT/JWKS (`auth_mode=jwt`, `[serve.jwt]`) — `docs/ops/AUTH.md`  
- Audit JSONL with size rotation — `src/audit_log.rs` (`SHARECLI_AUDIT_MAX_BYTES` / `RETAIN`)  
- AuthN burn: `sharecli_http_unauthorized_total` + `SharecliAuthFailBurn` — `docs/ops/SLO.md` SLO-4  
- Non-root Containerfile `USER` + HEALTHCHECK  
- cargo audit / deny / Dependabot on CI  
- Opt-in pprof only (`SHARECLI_PPROF`)  

## Residual / accepted risks

- Open loopback serve without token (localhost trust model) — document, do not bind `0.0.0.0` without AuthN  
- Unsigned GitHub Release archives until signing/notarization lands (C11 L112)  
- No OAuth login UI / SAML / unix-socket peer creds; AuthZ remains coarse (authenticated vs not)  
- JWT mode trusts configured JWKS document; operator must rotate keys via file update + restart (no hot JWKS fetch yet)  

## Review checklist (W5.3 — 2026-07-13)

- [x] Re-run STRIDE after AuthN/serve route changes (JWT + retention + burn)  
- [x] Confirm SECURITY.md summary table still matches this file  
- [x] Confirm Formula / release signing status in C11 lane (bottle sha filled; L112 still Blocked)  
- [x] Sign-off: post-federation review complete; next review on yearly cadence or next AuthN change  
