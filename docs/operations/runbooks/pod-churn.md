# Pod Churn Runbook

- Owner: `bijux-atlas-operations`
- Stability: `stable`

## Symptoms

- Elevated 5xx during reschedules/evictions.
- Latency spikes during deployment churn.
- Readiness flaps across replicas.

## Metrics

- `bijux_http_requests_total` by status/route
- `bijux_http_request_latency_p95_seconds`
- `bijux_overload_shedding_active`
- pod restart count and rollout status

## Drill

1. `make ops-drill-pod-churn`
2. `make ops-load-shedding`

## Expected Output

- Cheap requests remain serviceable.
- Non-cheap requests may shed under pressure.
- Service returns to steady latency after churn.

## Mitigations

1. Increase replica floor and verify PDB/HPA settings.
2. Reduce rollout concurrency during high traffic.
3. Validate node-local cache profile for faster recovery.

## Rollback

1. Roll back rollout settings to previous stable values.
2. Re-run `make ops-drill-pod-churn` to confirm stabilization.

