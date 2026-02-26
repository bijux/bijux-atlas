# SLO Registry Refresh Stale

- Owner: `bijux-atlas-operations`
- Tier: `tier2`
- Audience: `operators`
- Source-of-truth: `ops/CONTRACT.md`, `ops/inventory/**`, `ops/schema/**`

## Symptoms

- `BijuxAtlasRegistryRefreshStale` firing.
- `atlas_registry_refresh_age_seconds` remains above threshold.

## Metrics

- `atlas_registry_refresh_age_seconds`
- `atlas_registry_refresh_failures_total`

## Commands

```bash
make ops-proof-cached-only
kubectl -n atlas-e2e logs deploy/atlas-e2e-atlas --tail=200
```

## Expected outputs

- Refresh age drops below threshold and remains stable.
- No sustained refresh failure increments.

## Mitigations

- Validate registry backend reachability and credentials.
- Restart refresh worker if wedged.
- Temporarily increase cache TTL only with incident approval.

## Alerts

- Primary alert: `BijuxAtlasRegistryRefreshStale`.
- Dashboard: `docs/operations/observability/dashboard.md`.
- Drill reference: `make ops-drill-store-outage`.

## Rollback

- Roll back registry integration/config changes from the latest deploy.

## Postmortem checklist

- Record stale duration, root cause, and freshness guard improvements.
