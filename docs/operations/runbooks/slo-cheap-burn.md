# SLO Cheap Burn

## Symptoms

- `BijuxAtlasCheapSloBurnFast|Medium|Slow` firing.
- Elevated 5xx rate on `class="cheap"` traffic.

## Metrics

- `http_requests_total{class="cheap",status=~"5.."}`
- `http_requests_total{class="cheap"}`
- `atlas_overload_active`

## Commands

```bash
make ops-slo-alert-proof
kubectl -n atlas-observability get prometheusrules
```

## Expected outputs

- Fast/medium/slow burn alerts are evaluated and present.
- Cheap endpoint 5xx ratio returns below budget after mitigation.

## Mitigations

- Verify cheap endpoint dependency health and restart failed dependency pods.
- Reduce expensive background load that can starve cheap handlers.
- If needed, temporarily tighten admission on standard/heavy classes.

## Rollback

- Revert latest config/chart changes impacting request routing or concurrency.

## Postmortem checklist

- Capture burn windows and top failing routes.
- Record trigger, mitigation timeline, and residual error budget.
