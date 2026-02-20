## Summary
- 

## Validation
- [ ] `make dev-fmt`
- [ ] `make dev-lint`
- [ ] `make dev-test-all`
- [ ] `make dev-audit`

## Contract / SSOT Checklist
- [ ] Any API/CLI/metrics/error/chart/trace/config/artifact surface change updates `docs/contracts/*` first.
- [ ] Generated artifacts were refreshed (`scripts/areas/contracts/generate_contract_artifacts.py`).
- [ ] `make ssot-check` is green.
- [ ] OpenAPI drift reviewed (`make openapi-drift`).

## Risk
- [ ] Breaking change: explain in PR body and update compatibility docs.

## Ops/E2E Boundary Checklist (required if touching `ops/e2e/**`)
- [ ] Where should this fix live?
  - [ ] `ops/e2e` (scenario/assertion only)
  - [ ] `ops/run` (entrypoint/orchestration)
  - [ ] `ops/k8s` or `ops/stack` (deployment/infrastructure)
  - [ ] `ops/obs` or `ops/load` (telemetry/load-only)
- [ ] No direct infra patch/fixup was added under `ops/e2e/**`.
- [ ] `make policies/boundaries-check` is green.
- [ ] Stack/k8s smoke coverage ran in CI for this PR.
