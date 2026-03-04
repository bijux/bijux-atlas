# Helm Env Allowlist Subset

- Owner: `bijux-atlas-operations`
- Type: `reference`
- Audience: `reviewers`
- Stability: `stable`
- Last verified against: `main@7dea4f4b9a65a61796b0f7ac8c2d185c0eaddb07`
- Reason to exist: define the executable contract that keeps Helm-emitted ConfigMap env keys inside the runtime allowlist.

## Contract

- Check ID: `OPS-HELM-ENV-001`
- Domain: `ops`
- Mode: `effect`
- Blocking lanes: `pr`, `merge`, `release`

## What it checks

- Renders the canonical chart from `ops/k8s/charts/bijux-atlas`.
- Extracts only `ATLAS_` and `BIJUX_` keys from matching ConfigMap `.data` sections.
- Loads `configs/contracts/env.schema.json`.
- Fails if any rendered key is outside `allowed_env`.
- Reports allowlist keys missing from Helm as informational drift data.

## Reproduce locally

```bash
bijux dev atlas contracts ops --mode effect --allow-subprocess --filter-contract OPS-HELM-ENV-001
```

To inspect the raw emitted surface directly:

```bash
bijux dev atlas ops helm-env \
  --chart ops/k8s/charts/bijux-atlas \
  --values ops/k8s/charts/bijux-atlas/values.yaml \
  --allow-subprocess \
  --format json
```

## Reports

- `artifacts/contracts/ops/helm/helm-env-report.json`
- `artifacts/contracts/ops/helm/helm-env-subset.json`

`extra` is blocking.
`missing` is informational and helps catch stale allowlist entries.
