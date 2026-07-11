# Eval index (sharecli)

Process-supervisor evaluation surfaces for audit-v38 **C08**.

| Doc / path | Role |
|------------|------|
| [`adr/0001-eval-surface-out-of-scope.md`](../adr/0001-eval-surface-out-of-scope.md) | Governance: Harbor/SWE-bench N/A |
| [`REPRO.md`](REPRO.md) | Seed / lockfile / SHA reproducibility |
| [`baselines/criterion-baseline.json`](baselines/criterion-baseline.json) | Committed Criterion means for the perf gate |
| `benches/` | Criterion microbenches |
| `scripts/load/` | HTTP load burst against `/healthz` |
| `scripts/check-bench-baseline.py` | Fail on >50% mean regression vs baseline |
| `scripts/seed-bench-baseline.py` | Refresh baseline JSON from Criterion output |
| `.github/workflows/bench.yml` | Soft `criterion` job + hard-ish `bench-gate` job |
| [`ops/SLO.md`](../ops/SLO.md) | SLO + append-only bench measurement rows |
