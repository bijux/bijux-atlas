# Profile Matrix Contracts

- Owner: `docs-governance`
- Type: `reference`
- Audience: `user`
- Stability: `stable`
- Reason to exist: document the executable checks that keep the Kubernetes install profile matrix renderable and schema-valid.

## Check IDs

- `OPS-PROFILES-001`: every install profile must render with `helm template`.
- `OPS-PROFILES-002`: every install profile must satisfy the merged `values.schema.json`.
- `OPS-PROFILES-003`: every install profile must pass `kubeconform` when `kubeconform` is available.
- `OPS-PROFILES-004`: the rollout-safety profile set must exist and validate as a named subset.
- `OPS-DATASET-001`: every install profile `cache.pinnedDatasets` entry must stay inside `ops/datasets/manifest.json`.

## Reproduce locally

```bash
bijux dev atlas ops profiles validate --allow-subprocess --format json
bijux dev atlas ops profiles validate --allow-subprocess --profile ci --format json
bijux dev atlas ops profiles validate --allow-subprocess --profile-set rollout-safety --format json
bijux dev atlas contracts ops --mode effect --allow-subprocess --filter-contract OPS-DATASET-001
bijux dev atlas contracts ops --mode effect --allow-subprocess --filter-contract OPS-PROFILES-001
bijux dev atlas contracts ops --mode effect --allow-subprocess --filter-contract OPS-PROFILES-004
```

## Failure example

```json
{
  "profile": "perf",
  "values_schema": {
    "status": "fail",
    "note": "values schema failure",
    "errors": [
      "$.image: value matches forbidden schema"
    ]
  }
}
```

## Safe updates

- Keep shared defaults in `ops/k8s/charts/bijux-atlas/values.yaml`.
- Keep profile files focused on environment-specific overrides.
- Keep `cache.pinnedDatasets` entries inside `ops/datasets/manifest.json` before updating any profile.
- If `rollout-safety-contract.json` adds a profile, add the same profile to `ops/k8s/install-matrix.json` before merging.
