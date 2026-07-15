# Structured logging (soft)

Audit-v38 **C05 L41**.

## Formats

| Mode | How | Notes |
|------|-----|-------|
| Pretty (default) | unset / `SHARECLI_LOG_FORMAT=pretty` | Human local debug; respects `NO_COLOR` |
| JSON | `SHARECLI_LOG_FORMAT=json` | One JSON object per event for agents/log shippers |

`SHARECLI_LOG_FORMAT` applies when logging is enabled (not `--quiet`). Verbose:
`--verbose` / `-v` → DEBUG.

```bash
SHARECLI_LOG_FORMAT=json sharecli serve --bind 127.0.0.1:9000
```

OTel spans remain optional via `OTEL_EXPORTER_OTLP_ENDPOINT` (`docs/ops/otel.md`).
