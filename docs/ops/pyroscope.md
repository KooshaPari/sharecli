# Pyroscope push (soft) — sharecli

Continuous profile ingest for audit-v38 **C05** (Pyroscope gap). Local
`SHARECLI_PPROF=1` already exposes `/debug/pprof/profile`. This doc covers
**pushing** those profiles to a Pyroscope-compatible backend without requiring
live Grafana Cloud credentials in CI.

## Env

| Variable | Purpose |
|----------|---------|
| `SHARECLI_PPROF=1` | Enable CPU profile HTTP route |
| `SHARECLI_PYROSCOPE_URL` | Base URL, e.g. `http://127.0.0.1:4040` |
| `SHARECLI_PYROSCOPE_APP` | Application name (default `sharecli`) |
| `SHARECLI_SERVE_TOKEN` | Optional bearer for authenticated serve |

## Local Pyroscope (Docker)

```bash
docker run --rm -p 4040:4040 grafana/pyroscope:latest
```

## One-shot push

```bash
# from repo root (unix)
just pyro-push-sample
# or:
curl -o /tmp/sharecli.pb "http://127.0.0.1:9000/debug/pprof/profile?seconds=5"
curl -X POST \
  "${SHARECLI_PYROSCOPE_URL:-http://127.0.0.1:4040}/ingest?name=${SHARECLI_PYROSCOPE_APP:-sharecli}&from=$(date +%s)000&format=pprof" \
  --data-binary @/tmp/sharecli.pb
```

Windows: capture with `samply` / `cargo flamegraph` (see `profiling.md`); Pyroscope
ingest of pprof still works if you convert/export a compatible profile.

## CI stance

No live Pyroscope secret is required for merge gates. Optional soft job may
smoke `curl` against a disposable Pyroscope container later.

## Related

- [`profiling.md`](./profiling.md) — enable + capture
- Grafana Pyroscope ingest API — pprof format
