# Eval index (sharecli)

Process-supervisor evaluation surfaces for audit-v38 **C08**.

| Doc / path | Role |
|------------|------|
| [`adr/0001-eval-surface-out-of-scope.md`](../adr/0001-eval-surface-out-of-scope.md) | Governance: Harbor/SWE-bench N/A |
| [`REPRO.md`](REPRO.md) | Seed / lockfile / SHA reproducibility |
| `benches/` | Criterion microbenches |
| `scripts/load/` | HTTP load burst against `/healthz` |
| `.github/workflows/bench.yml` | Soft (advisory) PR/main bench job |
| [`ops/SLO.md`](../ops/SLO.md) | SLO + append-only bench measurement rows |
