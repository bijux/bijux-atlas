# Runbook: Registry Federation

- Owner: `bijux-atlas-store`

## Symptoms

- Inconsistent dataset visibility across pods.
- Catalog churn or conflict-shadow anomalies.

## Metrics

- `bijux_errors_total`
- `bijux_dataset_hits`
- `bijux_dataset_misses`

## Commands

```bash
$ curl -s http://127.0.0.1:8080/debug/registry-health
$ make ssot-check
```

## Expected outputs

- Registry health endpoint reports deterministic source order.
- SSOT checks confirm registry contract consistency.

## Mitigations

- Freeze registry refresh during incident.
- Reorder source priority to trusted registry.

## Rollback

- Restore previous registry source ordering and TTL settings.

## Postmortem checklist

- Conflict root cause identified.
- Merge semantics tests updated.
- Runbook and policy docs updated.
