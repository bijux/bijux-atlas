# SLO Overload Survival

## Symptoms

- `BijuxAtlasOverloadSurvivalViolated` firing.
- `atlas_overload_active` is high while cheap endpoint success drops.

## Metrics

- `atlas_overload_active`
- `atlas_shed_total{class="cheap"}`
- `http_requests_total{class="cheap",status=~"2.."}`

## Commands

```bash
make ops-drill-overload
make ops-slo-alert-proof
```

## Expected outputs

- Overload control remains active, but cheap class success recovers above 99.99%.

## Mitigations

- Verify shed policy prioritizes cheap class.
- Increase cheap-class concurrency budget.
- Reduce standard/heavy queue depth pressure.

## Rollback

- Revert overload threshold tuning and controller changes.

## Postmortem checklist

- Capture overload window, cheap success dip duration, and corrected thresholds.
