# Threat Model — sharecli

**Status:** ACTIVE  
**Owners:** maintainers (`@KooshaPari`)  
**Review cadence:** yearly or after any AuthN / serve / spawn surface change  
**Last reviewed:** 2026-07-12  
**Related:** [`SECURITY.md`](SECURITY.md), [`BOUNDARY.md`](BOUNDARY.md), [`docs/ops/AUTH.md`](docs/ops/AUTH.md)

Standalone STRIDE + attack-surface inventory for audit-v38 **L39** / **L20**.

## Trust boundary (summary)

| In scope | Out of scope |
|----------|--------------|
| Local `sharecli serve` HTTP/WS bind | Kernel / hypervisor escape |
| Process spawn / stop / limits | Supply-chain compromise of GitHub Actions runners |
| Config TOML under XDG / known folders | Physical access to unlocked workstation |
| Bearer token AuthN when configured | Federated IdP (future) |

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
| Serve HTTP | **High** if open bind without token | Medium — request smuggling N/A for simple routes | Medium — mitigate via `SHARECLI_AUDIT_LOG` | Medium — metrics / config routes | **High** — unbounded clients | Medium — spawn as user |
| Spawn | Medium — command injection via config | High — arbitrary argv from config | Low | Low | **High** — fork bomb / memory | Medium — no root required |
| Notifier webhook | Medium — URL from config | Low | Low | Medium — event payloads | Low | Low |
| Release assets | Medium until signed | Medium — unsigned archives | Low | Low | Low | Low |

## Attack surface inventory (serve)

| Route | Auth | Risk notes |
|-------|------|------------|
| `/healthz`, `/readyz` | Public | Keep probe-only; no secrets |
| `/metrics/prometheus` | Bearer if configured | May leak process names/paths |
| `/config` | Bearer if configured | Full config dump — treat as sensitive |
| `/ws` | Bearer if configured | Live thermal/events |
| `/debug/pprof/profile` | Bearer + `SHARECLI_PPROF=1` | CPU sample; loopback preferred |

## Mitigations (current)

- Optional Bearer (`SHARECLI_SERVE_TOKEN` / `config.serve.bearer_token`) — `docs/ops/AUTH.md`  
- Audit JSONL — `src/audit_log.rs`  
- Non-root Containerfile `USER` + HEALTHCHECK  
- cargo audit / deny / Dependabot on CI  
- Opt-in pprof only (`SHARECLI_PPROF`)  

## Residual / accepted risks

- Open loopback serve without token (localhost trust model) — document, do not bind `0.0.0.0` without token  
- Unsigned GitHub Release archives until signing/notarization lands (C11 L112)  
- No federated AuthN yet (C02 L21 gap)  

## Review checklist

- [ ] Re-run STRIDE after AuthN/serve route changes  
- [ ] Confirm SECURITY.md summary table still matches this file  
- [ ] Confirm Formula / release signing status in C11 lane  
