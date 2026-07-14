# OpenAPI — `sharecli serve`

Committed contract: [`serve.yaml`](./serve.yaml) (OpenAPI 3.0.3).

## Drift CI

`scripts/check-openapi-drift.py` asserts a **symmetric** match between:

- Axum `.route("…")` paths in `src/commands/serve.rs`
- Top-level `paths:` keys in `docs/openapi/serve.yaml`

Run locally:

```bash
python3 scripts/check-openapi-drift.py
```

Workflow: `.github/workflows/openapi-drift.yml` (C00 L2 / FR-004 HTTP surface).
