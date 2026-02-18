# Runbook: Rollback Playbook

- Owner: `bijux-atlas-operations`

## Symptoms

- New deployment causes sustained error or latency regression.

## Metrics

- `bijux_http_requests_total`
- `bijux_http_request_latency_p95_seconds`
- `bijux_errors_total`

## Commands

```bash
$ kubectl rollout undo deploy/bijux-atlas -n default
$ curl -s http://127.0.0.1:8080/readyz
```

## Expected outputs

- Rollout undo returns success.
- Readiness and request metrics return to baseline window.

## Mitigations

- Halt rollout progression.
- Keep degraded mode controls enabled until stable.

## Rollback

- Revert API image and catalog pointer to last known good state.

## Postmortem checklist

- Trigger commit/config identified.
- Compatibility impact documented.
- Rollback drill evidence attached.

## See also

- `ops-ci`

## Dashboards

- [Observability Dashboard](../observability/dashboard.md)

## Drills

- make ops-drill-store-outage
- make ops-drill-pod-churn
