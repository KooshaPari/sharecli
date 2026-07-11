# Bench / hyperfine scripts (C08)

| Script | Purpose |
|--------|---------|
| `hyperfine-healthz.sh` | Hyperfine latency for `GET /healthz` against a live serve |

## Criterion (in-repo)

```bash
cargo bench --locked --bench config_parse -- --sample-size 10 --warm-up-time 1 --measurement-time 2
cargo bench --locked --bench pool_list -- --sample-size 10 --warm-up-time 1 --measurement-time 2
cargo bench --locked --bench prometheus_render -- --sample-size 10 --warm-up-time 1 --measurement-time 2
python3 scripts/seed-bench-baseline.py
```

CI asserts means against `docs/eval/baselines/criterion-baseline.json` (see `.github/workflows/bench.yml`).

## Hyperfine

```bash
sharecli serve --bind 127.0.0.1:9000 &
./scripts/bench/hyperfine-healthz.sh
```

Pins and seed: `docs/eval/REPRO.md`.
