# Bench / hyperfine scripts (C08)

| Script | Purpose |
|--------|---------|
| `hyperfine-healthz.sh` | Hyperfine latency for `GET /healthz` against a live serve |

## Criterion (in-repo)

```bash
cargo bench --locked --bench config_parse -- --sample-size 10 --warm-up-time 1 --measurement-time 2
cargo bench --locked --bench pool_list -- --sample-size 10 --warm-up-time 1 --measurement-time 2
cargo bench --locked --bench prometheus_render -- --sample-size 10 --warm-up-time 1 --measurement-time 2
cargo bench --locked --bench jwt_auth_validate -- --sample-size 10 --warm-up-time 1 --measurement-time 2
python3 scripts/seed-bench-baseline.py
```

CI asserts means against `docs/eval/baselines/criterion-baseline.json` (see `.github/workflows/bench.yml`).

## Hyperfine

```bash
sharecli serve --bind 127.0.0.1:9000 &
./scripts/bench/hyperfine-healthz.sh
```

Pins and seed: `docs/eval/REPRO.md`.

### Hyperfine JSON artifact

`hyperfine-healthz.sh` exports JSON via `--export-json` (default:
`docs/eval/baselines/hyperfine-healthz.json`, override with
`SHARECLI_HYPERFINE_OUT`). Commit refreshed JSON after local runs when
updating LOAD-2 latency trends.

CI uploads hyperfine JSON as an Actions artifact:
- Soft job `hyperfine healthz (soft)` on PR/push (`continue-on-error`)
- Nightly / `workflow_dispatch` job `cargo bench (nightly trends)` also
  builds `sharecli`, probes `/healthz`, and uploads `hyperfine-healthz-<sha>.json`
  alongside Criterion trend JSON (`docs/eval/TRENDS.md`).
