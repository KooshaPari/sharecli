# Error budget policy (sharecli serve)

Formal error-budget and multi-window multi-burn-rate (MWMB) policy for audit-v38
**C05 L46**. SLO targets live in [`SLO.md`](SLO.md); alert rules in
[`alertmanager/sharecli.yml`](alertmanager/sharecli.yml); routing in
[`alerting.md`](alerting.md) and [`live-pd.md`](live-pd.md).

## Principles

1. **Error budget** — each SLO has a monthly (30-day) allowance for bad events.
   When the budget is exhausted, feature work pauses until reliability recovers.
2. **Burn rate** — how fast the budget is consumed relative to steady-state.
   A burn rate of `14.4` means the monthly budget would be gone in ~2 days at the
   current pace.
3. **MWMB alerting** — pair **fast** windows (page immediately) with **slow**
   windows (ticket / Slack) per the [Google SRE Workbook](https://sre.google/workbook/alerting-on-slos/).

## SLO budgets

| SLO | Objective | Monthly error budget | Steady burn threshold |
|-----|-----------|----------------------|------------------------|
| [SLO-1](SLO.md#slo-1--liveness-availability) | ≥ 99.5% `/healthz` success | 0.5% downtime (~3.6 h) | `up == 0` for 2m |
| [SLO-2](SLO.md#slo-2--controlled-restart--readiness-drain) | ≤ 2 unplanned restarts / day | 2 restarts / 24h | > 3 readiness flips / hour |
| [SLO-3](SLO.md#slo-3--metrics-scrape-freshness) / HTTP RED | 5xx rate ≤ 5% (5m) | 5% of requests may 5xx | 5% for 10m (slow burn) |
| [SLO-4](SLO.md#slo-4--authn-failure-burn) | 401 rate ≤ 10% (5m) | 10% of requests may 401 | 10% for 10m (slow burn) |

## MWMB multipliers (Google SRE)

Applied to the **monthly error budget** (not the SLO target itself):

| Window | Budget slice | Burn multiplier | Route |
|--------|--------------|-----------------|-------|
| 1 h (fast) | 2% of monthly | ×14.4 | `critical` → page |
| 6 h (slow) | 5% of monthly | ×6 | `warning` → Slack/ticket |

Prometheus rules encode these as `burn_window: fast|slow` labels on paired alerts in
`alertmanager/sharecli.yml`.

## Alert pairs

| Fast (page) | Slow (ticket) | SLO |
|-------------|---------------|-----|
| `SharecliHealthzDown` | `SharecliSlo1AvailabilityBurnSlow` | SLO-1 |
| `SharecliHttpErrorBudgetBurnFast` | `SharecliHttpErrorBudgetBurn` | SLO-3 / RED |
| `SharecliAuthFailBurnFast` | `SharecliAuthFailBurn` | SLO-4 |
| — | `SharecliReadyzDrainingStorm` | SLO-2 (info) |

### HTTP 5xx fast burn math

Steady-state budget allows **5%** 5xx (`0.05`). Fast window uses **14.4×** steady
burn → page when 5m error rate **> 72%** (`0.05 × 14.4`) for 2m.

### AuthN fast burn math

Steady-state budget allows **10%** 401 (`0.10`). Fast window pages when 5m
unauthorized rate **> 50%** for 2m (practical cap below 100% traffic).

## Escalation

| Budget remaining | Action |
|------------------|--------|
| > 50% | Normal development |
| 25–50% | Reliability review in weekly ops sync |
| < 25% | Freeze non-critical merges; prioritize burn fixes |
| Exhausted (0%) | Incident commander; postmortem required before new features |

Record burn events in the append-only table at the bottom of [`SLO.md`](SLO.md).

## On-call

Live PagerDuty routing keys and signed rosters are **not** committed to git.
Use env / secret store per [`live-pd.md`](live-pd.md). Severity routing in
[`alerting.md`](alerting.md) maps `critical` → page, `warning` → Slack.

## Verification

```bash
just slo-alerts-validate   # YAML + MWMB pair gate (C05 L46)
cargo test -p sharecli c05_l46 -- --nocapture
```
