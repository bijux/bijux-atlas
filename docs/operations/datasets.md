# Dataset Operations

Dataset lifecycle is controlled through make targets only.

## Fetch and Verify

```bash
make ops-datasets-fetch
```

## Publish

```bash
make ops-publish DATASET=medium
make ops-publish DATASET=real1
```

## Validate and Warm

```bash
make ops-catalog-validate
make ops-dataset-qc
make ops-warm
make ops-cache-status
```

## Drills and Promotion

```bash
make ops-drill-corruption-dataset
make ops-dataset-promotion-sim
make ops-dataset-multi-release-test
make ops-dataset-federated-registry-test
```

## Notes

- Metadata snapshots are written under `artifacts/ops/<run-id>/datasets/` during publish.
- Quarantined corrupted datasets are tracked under `artifacts/e2e-datasets/quarantine/`.
