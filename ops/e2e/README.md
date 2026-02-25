# Ops E2E

Composition-only end-to-end scenarios across stack, k8s, observe, load, and datasets.

## Start Here
- `ops/e2e/CONTRACT.md`
- `ops/e2e/suites/suites.json`
- `ops/e2e/scenarios/scenarios.json`
- `ops/e2e/expectations/expectations.json`
- `ops/e2e/reproducibility-policy.json`
- `ops/e2e/fixtures/allowlist.json`
- `ops/e2e/taxonomy.json`

## Generated
- `ops/e2e/generated/e2e-summary.json`
- `ops/e2e/generated/coverage-matrix.json`

## Entrypoints
- `make ops-e2e SUITE=smoke|k8s-suite|realdata`
- `make ops-e2e-validate`

Placeholder extension directories tracked with `.gitkeep`: `ops/e2e/datasets`, `ops/e2e/expectations`, `ops/e2e/manifests`.
