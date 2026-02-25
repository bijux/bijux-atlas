# E2E Composition Contract

E2E is composition-only and orchestrates existing domains.

Source contracts:
- `ops/e2e/suites/suites.json`
- `ops/e2e/scenarios/scenarios.json`
- `ops/e2e/expectations/expectations.json`
- `ops/e2e/fixtures/allowlist.json`
- `ops/e2e/reproducibility-policy.json`
- `ops/e2e/taxonomy.json`

Generated artifacts:
- `ops/e2e/generated/e2e-summary.json`
- `ops/e2e/generated/coverage-matrix.json`

Rules:
- suites compose stack/k8s/load/datasets/observe capabilities
- fixtures are restricted by allowlist
- scenario ordering and summary outputs are deterministic
