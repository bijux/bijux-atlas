# End-to-end Tests

- Owner: `bijux-atlas-operations`
- Type: `runbook`
- Audience: `operator`
- Stability: `stable`
- Last verified against: `main@7dea4f4b9a65a61796b0f7ac8c2d185c0eaddb07`
- Reason to exist: define the exact end-to-end test entrypoint and expected outputs.

## Commands

```bash
make k8s-validate
make ops-k8s-tests
bijux-dev-atlas ops scenario list --format json
bijux-dev-atlas ops scenario run --scenario minimal-single-node --plan --format json
bijux-dev-atlas ops scenario run --scenario minimal-single-node --evidence --allow-write --format json
```

## Expected outputs

- Validation and test summaries in command output.
- Evidence artifacts under `artifacts/evidence/k8s/`.
- Scenario evidence artifacts under `artifacts/ops/scenarios/<scenario-id>/<run-id>/`.
- No failing checks in release gate reports.

## Scenario spec reference

- Source of truth: `ops/e2e/scenarios/scenarios.json`.
- Compatibility table: `ops/e2e/scenarios/version-compatibility.json`.
- Required tools registry: `ops/e2e/scenarios/required-tools.json`.
- Scenario contract schema: `ops/schema/e2e-scenarios.schema.json`.

## Scenario artifact layout

- `artifacts/ops/scenarios/<scenario-id>/<run-id>/result.json`: machine-readable scenario report.
- `artifacts/ops/scenarios/<scenario-id>/<run-id>/summary.md`: human report summary.
- `run-id` is deterministic (`sha256(scenario-id + mode)` truncated to 12 hex chars).

## Scenario matrix

- CI fast coverage: `smoke`, `query-pagination`, `query-filter-projection`, `artifact-integrity`.
- CI slow coverage: `k8s-suite`, `realdata`, `perf-e2e`, `high-concurrency`.
- Manual operator drills: `low-resource`, `offline-mode`, `restart-resume`, `mixed-load`.

## Core guides

- [Kubernetes tests](k8s-tests.md)
- [Fixture taxonomy](fixtures.md)
- [Scenarios as evidence](scenarios-evidence.md)
- [Scenario spec reference](scenario-spec-reference.md)
- [Scenario artifact layout](scenario-artifact-layout.md)
- [Scenario matrix](scenario-matrix.md)

## Next

- [Kubernetes Operations](../k8s/index.md)
- [Load Testing](../load/index.md)
