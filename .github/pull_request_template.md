## Summary
- 

## Validation
- [ ] `make ci-fast`
- [ ] `make ci-pr` or CI link attached when a full local run is impractical.
- [ ] Focused surface checks were rerun for the area touched by this PR.

## Contract / SSOT Checklist
- [ ] Any API/CLI/metrics/error/chart/trace/config/artifact surface change updates `docs/contracts/*` first.
- [ ] Generated artifacts were refreshed with the owning command (`make makes-target-list`, `make docs-reference-regenerate`, `make openapi-generate`, or another surface-specific generator).
- [ ] If makes target metadata changed, `make makes-target-list` is green.
- [ ] If docs reference pages changed, `make docs-reference-check` is green.
- [ ] If OpenAPI changed, `make openapi-generate` and the OpenAPI contract tests are green.
- [ ] No new bypass entries were introduced (attach `artifacts/atlas-dev/check/policies-bypass-report.json` or link CI artifact).
- [ ] No new ops scripts were added; use `bijux dev atlas` command families instead.
- [ ] Any new ops contract includes gate mapping, tests, and schema updates where applicable.
- [ ] Any new ops contract deletes or merges an existing ops contract, or links the approval that allows contract count growth.

## Risk
- [ ] Breaking change: explain in PR body and update compatibility docs.

## Ops/E2E Boundary Checklist (required if touching `ops/e2e/**`)
- [ ] Where should this fix live?
  - [ ] `ops/e2e` (scenario/assertion only)
  - [ ] `ops/run` (entrypoint/orchestration)
  - [ ] `ops/k8s` or `ops/stack` (deployment/infrastructure)
  - [ ] `ops/observe` or `ops/load` (telemetry/load-only)
- [ ] No direct infra patch/fixup was added under `ops/e2e/**`.
- [ ] Automation boundary checks are green in `make ci-fast` or the linked CI run.
- [ ] Stack/k8s smoke coverage ran in CI for this PR.

## Docs Checklist (required if touching `docs/**`, `mkdocs.yml`, or command docs output)
- [ ] Completed `.github/PULL_REQUEST_TEMPLATE/docs-governance.md` checklist items in PR description.
- [ ] Crate docs rule respected (`README.md` + `CONTRACT.md` in crate root; crate docs checks pass).
