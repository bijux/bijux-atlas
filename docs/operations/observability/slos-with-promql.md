# SLOs with PromQL

- Owner: `bijux-atlas-operations`
- Type: `runbook`
- Audience: `operator`
- Stability: `stable`
- Last verified against: `main@d09a3c7f`
- Reason to exist: define practical PromQL examples used to evaluate SLO burn and operator response thresholds.

## Scope

This page complements the policy in [SLO policy](slo-policy.md) with concrete query shapes used in incident triage.

## Example query patterns

- Error-rate burn (short window):
  - `sum(rate(http_requests_total{code=~"5.."}[5m])) / sum(rate(http_requests_total[5m]))`
- Error-rate burn (long window):
  - `sum(rate(http_requests_total{code=~"5.."}[1h])) / sum(rate(http_requests_total[1h]))`
- High-latency budget pressure:
  - `histogram_quantile(0.99, sum(rate(http_request_duration_seconds_bucket[5m])) by (le))`
- Availability proxy:
  - `sum(up{job="bijux-atlas"}) / count(up{job="bijux-atlas"})`

## Interpretation guidance

- Evaluate short and long burn windows together to reduce false positives.
- Treat sustained p99 increases plus error-rate growth as overload risk.
- Confirm signal with dashboards and traces before rollback or traffic shedding.

## Verify success

```bash
make ops-observability-verify
```

Expected result: configured SLO-related alert rules and dashboards remain consistent with the query model.

## Next

- [SLO policy](slo-policy.md)
- [Alerts](alerts.md)
- [Dashboards](dashboards.md)
