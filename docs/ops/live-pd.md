# Live PagerDuty / on-call wiring (soft)

Audit-v38 **C05 L48** — live incident routing for `sharecli serve`.

Rule pack and runbooks already ship in `docs/ops/alertmanager/sharecli.yml` and
`docs/ops/alerting.md`. This file documents the **live PD/on-call wiring** gap:
Alertmanager receivers, SLO error-budget links, and escalation placeholders.
Secrets and roster rows stay out of git until org policy allows.

## Soft posture

| Item | Status |
|------|--------|
| Prometheus rules (`sharecli.yml`) | Done — severity labels on all alerts |
| Severity → route table | Done — `docs/ops/alerting.md` |
| Alertmanager `route` + receivers | **Soft** — env placeholders below |
| Signed on-call roster | **Placeholder** — fill before production page |
| Live PagerDuty routing key | **Placeholder** — secret store only |

Soft goal: L48 stays **2** until a non-placeholder receiver is exercised in staging.

## SLO → alert mapping

Error budgets and probe policy live in [`SLO.md`](./SLO.md). Wire pages to the SLO
that each alert protects.

| Alert (`sharecli.yml`) | SLO | Severity | Error-budget note |
|------------------------|-----|----------|-------------------|
| `SharecliHealthzDown` | [SLO-1](./SLO.md#slo-1--liveness-availability) | `critical` | Liveness probe failure; page immediately |
| `SharecliReadyzDrainingStorm` | [SLO-2](./SLO.md#slo-2--controlled-restart--readiness-drain) | `info` | Restart storm; investigate if >2/day budget burns |
| `SharecliHttpErrorBudgetBurn` | [SLO-3](./SLO.md#slo-3--metrics-scrape-freshness) / RED | `warning` | 5xx burn >5% for 10m |
| `SharecliAuthFailBurn` | [SLO-4](./SLO.md#slo-4--authn-failure-burn) | `warning` | 401 rate >10% for 10m |
| `SharecliHttpLatencyP99High` | [BENCH targets](./SLO.md#bench-linked-targets-c08) | `warning` | p99 >500ms for 15m |

Full probe table: [`SLO.md` — Mapping to probes](./SLO.md#mapping-to-probes).

## Alertmanager routes (placeholder)

Load rules from `docs/ops/alertmanager/sharecli.yml`. Example **staging** route
tree — substitute receivers from your secret store before enabling pages.

```yaml
# alertmanager.yml (snippet — do not commit routing keys)
global:
  resolve_timeout: 5m

route:
  receiver: slack-warnings
  group_by: [alertname, service, slo]
  group_wait: 30s
  group_interval: 5m
  repeat_interval: 4h
  routes:
    # SLO-1 liveness — page on-call
    - matchers:
        - severity = critical
        - service = sharecli
      receiver: pagerduty-primary
      continue: false

    # SLO-3/4 + latency — ticket / Slack during business hours
    - matchers:
        - severity = warning
        - service = sharecli
      receiver: slack-warnings
      continue: false

    # SLO-2 drain / info — no page
    - matchers:
        - severity = info
      receiver: "null"

receivers:
  - name: pagerduty-primary
    pagerduty_configs:
      - routing_key: "${PAGERDUTY_ROUTING_KEY}"  # Events API v2 — secret store
        severity: critical
        description: "{{ .CommonAnnotations.summary }}"

  - name: slack-warnings
    slack_configs:
      - api_url: "${SLACK_WEBHOOK_URL}"
        channel: "#sharecli-alerts"
        title: "{{ .CommonLabels.alertname }}"
        text: "{{ .CommonAnnotations.description }}"

  - name: "null"
```

### Built-in notifier fallback

When Prometheus/Alertmanager is not deployed, `src/notifier.rs` can POST to the
same webhook URLs. See [`alerting.md` — Built-in notifier](./alerting.md#built-in-notifier-no-prometheus).

## Escalation placeholders

Fill before declaring live PD. Do not commit phone numbers or routing keys.

| Tier | Role | Contact (placeholder) | When |
|------|------|----------------------|------|
| L1 | Primary on-call | `oncall-primary@example.com` / PD schedule **sharecli-primary** | `critical` pages; ack within **15m** |
| L2 | Secondary on-call | `oncall-secondary@example.com` / PD schedule **sharecli-secondary** | No L1 ack in 15m; auto-escalate |
| L3 | Service owner | `OWNER_PLACEHOLDER` | SLO budget exhausted or repeat pages in 24h |
| L4 | Platform / SRE | `SRE_PLACEHOLDER` | Multi-service outage or scrape infra down |

### Escalation policy (soft)

1. **Ack** — L1 acknowledges in PagerDuty within 15 minutes of `critical` page.
2. **Mitigate** — follow runbook in [`alerting.md`](./alerting.md) for the firing alert.
3. **Escalate** — if no ack at +15m, PD policy escalates to L2; at +30m unmitigated, notify L3.
4. **Post-incident** — log burn against the SLO in [`SLO.md`](./SLO.md); append a row to the measurement log if bench/load regressed.

## On-call roster (placeholder)

| Week (UTC) | Primary | Secondary | Notes |
|------------|---------|-----------|-------|
| `YYYY-Www` | `PRIMARY_NAME` | `SECONDARY_NAME` | Replace before first live page |
| `YYYY-Www+1` | `PRIMARY_NAME` | `SECONDARY_NAME` | Rotate weekly |

Roster source of truth: **PagerDuty schedule** or org calendar — not this table in git.

## Staging checklist

1. Deploy Prometheus scrape job for `job="sharecli-serve"` (see [`otel.md`](./otel.md)).
2. Mount `sharecli.yml` under Prometheus `rule_files`.
3. Configure Alertmanager receivers with **staging** routing keys (not production).
4. Fire synthetic alert: stop `sharecli serve` → confirm `SharecliHealthzDown` routes to staging PD.
5. Verify warning alerts land in `#sharecli-alerts` (or equivalent) without paging.
6. Document first successful staging page in release checklist; then bump L48 evidence.

## Related

- [Alerting + runbooks](./alerting.md) — severity table and per-alert steps
- [SLOs](./SLO.md) — error budgets and probe mapping
- [Alert rules](./alertmanager/sharecli.yml) — Prometheus rule pack
- [C05 lane evidence](../../audit/.lane-c05/C05.md) — L48 rubric score
