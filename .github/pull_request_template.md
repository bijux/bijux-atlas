## Summary
- 

## Validation
- [ ] `make dev-fmt`
- [ ] `make dev-lint`
- [ ] `make dev-test-all`
- [ ] `make dev-audit`

## Contract / SSOT Checklist
- [ ] Any API/CLI/metrics/error/chart/trace/config/artifact surface change updates `docs/contracts/*` first.
- [ ] Generated artifacts were refreshed (`scripts/contracts/generate_contract_artifacts.py`).
- [ ] `make ssot-check` is green.
- [ ] OpenAPI drift reviewed (`make openapi-drift`).

## Risk
- [ ] Breaking change: explain in PR body and update compatibility docs.
