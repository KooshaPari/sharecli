# Alerting + on-call routing (sharecli serve)

Severity-routed alerts for audit-v38 **L48**. Rules live in
`docs/ops/alertmanager/sharecli.yml` (Prometheus `rule_files`). Deliver via
Alertmanager → Slack / PagerDuty / webhook, or use sharecli's built-in notifier.

## Severity → route

| Severity | Destination | Ack / page |
|----------|-------------|------------|
| `critical` | PagerDuty / OpsGenie / on-call phone | Immediate page |
| `warning` | Slack `#sharecli-alerts` (or ticket) | Business hours |
| `info` | Dashboard / weekly digest | No page |

Example Alertmanager route snippet:

```yaml
route:
  receiver: slack-warnings
  group_by: [alertname, service]
  routes:
    - matchers: [severity = critical]
      receiver: pagerduty-oncall
      continue: false
    - matchers: [severity = warning]
      receiver: slack-warnings
    - matchers: [severity = info]
      receiver: null
receivers:
  - name: pagerduty-oncall
    pagerduty_configs:
      - routing_key: "${PAGERDUTY_ROUTING_KEY}"
  - name: slack-warnings
    slack_configs:
      - api_url: "${SLACK_WEBHOOK_URL}"
        channel: "#sharecli-alerts"
  - name: "null"
```

## Built-in notifier (no Prometheus)

`src/notifier.rs` already dispatches desktop + HTTP webhooks. Point a webhook
receiver at Slack/PagerDuty Events API v2 for process/thermal events when you
are not running a full Prom stack:

```bash
# example: SHARECLI_NOTIFY_WEBHOOK=https://hooks.slack.com/services/...
```

See `docs/ops/SLO.md` for error-budget policy that these alerts enforce.

## Runbooks

### SharecliHealthzDown

1. `curl -sf http://127.0.0.1:9000/healthz`
2. Check serve lock / PID (`sharecli serve` still holding bind).
3. Inspect `SHARECLI_AUDIT_LOG` for `serve_stop` / crash.

### SharecliHttpErrorBudgetBurn

1. Scrape `/metrics/prometheus` — inspect `sharecli_http_errors_total`.
2. Check auth misconfig (`SHARECLI_SERVE_TOKEN`) causing 401 storms (not 5xx).
3. Review recent deploy SHA vs `docs/eval/baselines/criterion-baseline.json`.

### SharecliHttpLatencyP99High

1. Confirm load (scripts/load).
2. Optional: `SHARECLI_PPROF=1` + `GET /debug/pprof/profile?seconds=15`.
3. Compare Criterion gate on the PR that last touched serve hot paths.

### SharecliAuthFailBurn

1. Scrape `/metrics/prometheus` — `sharecli_http_unauthorized_total`.
2. Tail audit log for `auth_fail` (`reason`, `mode`, `path`).
3. Verify bearer token / JWT issuer-audience-JWKS (`docs/ops/AUTH.md`).
4. Confirm probes (`/healthz`, `/readyz`) remain public and are not counted as auth failures.

### SLO-2

Unplanned restart storm: correlate thermal critical shutdowns and notifier
webhook volume; budget is ≤2 restarts / day (`docs/ops/SLO.md`).

## Related

- Rules: `docs/ops/alertmanager/sharecli.yml`
- SLOs: `docs/ops/SLO.md`
- Grafana: `docs/ops/grafana/sharecli-serve.json`
- RED metrics: `docs/ops/otel.md`
