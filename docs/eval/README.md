# Eval index (sharecli)

Process-supervisor evaluation surfaces for audit-v38 **C08**.

| Doc / path | Role |
|------------|------|
| [`GOVERNANCE.md`](GOVERNANCE.md) | In-scope family, N/A ADR map, bench add process, CI gate ownership |
| [`adr/0002-eval-surface-out-of-scope.md`](../adr/0002-eval-surface-out-of-scope.md) | Governance: Harbor/SWE-bench N/A |
| [`REPRO.md`](REPRO.md) | Seed / lockfile / SHA reproducibility |
| [`TRENDS.md`](TRENDS.md) | Nightly Criterion trend artifact contract |
| [`baselines/criterion-baseline.json`](baselines/criterion-baseline.json) | Committed Criterion means for the perf gate |
| `benches/` | Criterion microbenches (`config_parse`, `pool_list`, `prometheus_render`, `jwt_auth_validate`) |
| `scripts/load/` | HTTP load burst against `/healthz` |
| `scripts/check-bench-baseline.py` | Fail on >50% mean regression vs baseline |
| `scripts/seed-bench-baseline.py` | Refresh baseline JSON from Criterion output |
| `scripts/bench/export-trend.py` | Export Criterion means to nightly JSON |
| `.github/workflows/bench.yml` | Soft `criterion` + `bench-gate` + cron `bench-nightly` |
| [`ops/SLO.md`](../ops/SLO.md) | SLO + append-only bench measurement rows |
| [`ops/alerting.md`](../ops/alerting.md) | Alertmanager severity routing (C05 L48) |
