# Values Schema

- Owner: `bijux-atlas-operations`
- Type: `runbook`
- Audience: `operator`
- Stability: `stable`
- Last verified against: `main@7dea4f4b9a65a61796b0f7ac8c2d185c0eaddb07`
- Reason to exist: explain how the chart values schema rejects invalid deployments before they reach the cluster.

Related ops contracts: `OPS-K8S-001`, `OPS-K8S-004`.

## Purpose

The chart schema is the first safety gate for Atlas configuration. It rejects unsupported keys,
invalid value shapes, and high-risk deployment mistakes before Helm renders templates.

## What It Prevents

- Unknown top-level keys that drift away from the published chart contract.
- Unsafe image selection such as `image.tag=latest`.
- Invalid cache combinations such as `cachedOnlyMode=true` with
  `readinessRequiresCatalog=true`.
- Invalid store backend shapes that would otherwise fail at runtime.
- Missing or malformed workload attachment objects such as service accounts, config mounts, and
  extra volumes.

## Common Failure Examples

The repository keeps intentionally invalid examples under `ops/k8s/values/` so operators can see
what the schema is expected to reject:

- `ops/k8s/values/schema-failure-image-latest.yaml`
- `ops/k8s/values/schema-failure-cached-only.yaml`

## Verify

```bash
helm lint ops/k8s/charts/bijux-atlas
```

Expected result: the chart lints cleanly with its default values, and invalid overlays fail fast
before template rendering.

## Rollback

If a new schema rule blocks an intended deployment, revert the values change first, then update the
schema only if the new input is part of the supported long-term contract.
