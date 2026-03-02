# Kubernetes Install Profile Invariants

- Owner: `bijux-atlas-operations`
- Type: `runbook`
- Audience: `operator`
- Stability: `stable`
- Last verified against: `main@c59da0bf`
- Reason to exist: define the combinations that keep every install profile renderable and installable by construction.

## Allowed combinations

- Cached-only profiles must set `server.readinessRequiresCatalog: false`.
- Cached-only smoke profiles should keep `hpa.enabled: false` unless the environment provides autoscaling inputs.
- Offline profiles may keep `cache.initPrewarm.enabled: true` only when `cache.pinnedDatasets` is non-empty.
- Every `cache.pinnedDatasets` entry must exist in `ops/datasets/manifest.json`.
- Offline profiles with `networkPolicy.allowDns: false` must not depend on DNS-only endpoints during startup.
- Any profile that sets `image.digest` must set `image.tag: ""`.
- Perf profiles that enable `metrics.customMetrics.enabled: true` must keep `hpa.enabled: true` and provide the required metrics annotations.

## Current profile intent

- `ci`: cached-only smoke, catalog-independent, no autoscaling dependency.
- `offline`: air-gapped cached-only without init prewarm, no autoscaling dependency, no DNS egress.
- `perf`: digest-pinned performance baseline with HPA and custom metrics enabled.

## Drift control

- Shared defaults stay in `ops/k8s/charts/bijux-atlas/values.yaml`; profile files override only the minimum deltas for their target environment.
- The canonical render matrix is `ops/k8s/install-matrix.json`.
- The canonical dataset ID manifest is `ops/datasets/manifest.json`.
- Reproduce locally with `bijux dev atlas ops profiles validate --allow-subprocess --format json`.
- Reproduce the pinned-dataset closure gate with `bijux dev atlas contracts ops --mode effect --allow-subprocess --filter-contract OPS-DATASET-001`.

## Failure example

```json
{
  "profile": "ci",
  "helm_template": {
    "status": "fail",
    "note": "helm guard failure",
    "errors": [
      "cached-only mode cannot require catalog readiness"
    ]
  }
}
```

## Safe schema changes

- Change `ops/k8s/charts/bijux-atlas/values.schema.json` and the chart defaults together.
- Re-run `bijux dev atlas ops profiles validate --allow-subprocess --format json` after every schema edit.
- If a profile now depends on inherited defaults, keep the defaults in `values.yaml` and only override the profile-specific delta.

## Why prewarm needs pinned datasets

- The init prewarm flow needs a concrete dataset list so it can warm the cache deterministically.
- An offline profile without pinned dataset IDs cannot prove what should be fetched or staged ahead of startup.
