## Summary
- Explain the user-visible, operator-visible, or repository-visible change.
- Link the owning issue, incident, or approval when one exists.

## Validation
- [ ] `make ci-fast`
- [ ] `make ci-pr` or CI link attached when a full local run is impractical.
- [ ] Focused commands for the touched surface were rerun and noted in this PR.

## Surface / SSOT Checklist
- [ ] Any API, CLI, metrics, error, chart, trace, config, or artifact change updated the owning SSOT first (`configs/schemas/contracts/**`, `configs/registry/**`, `ops/api/contracts/**`, `ops/observe/contracts/**`, `ops/inventory/**`, or another owning source).
- [ ] Reader docs in `docs/**` were updated only after the owning SSOT and generated artifacts were correct.
- [ ] Root repository docs (`README.md`, `SECURITY.md`, `CONTRIBUTING.md`) and any touched crate README stay aligned with the numbered docs spine and the real install surface.
- [ ] Generated artifacts were refreshed with the owning command (`make makes-target-list`, `make docs-reference-regenerate`, `make openapi-generate`, or another surface-specific generator).
- [ ] If makes target metadata changed, `make makes-target-list` is green.
- [ ] If docs reference pages changed, `make docs-reference-check` is green.
- [ ] If docs were moved or renamed, `bijux dev atlas docs redirects sync --allow-write` was run and the redirect registry stayed in sync.
- [ ] If OpenAPI changed, `make openapi-generate` and the OpenAPI contract tests are green.
- [ ] If governance exceptions or bypass files changed, the PR links the approval and the relevant governance artifact or CI run.
- [ ] No new ad hoc root or ops scripts were added; use `bijux atlas ...` or `bijux dev atlas ...` surfaces instead.
- [ ] Any new ops contract includes gate mapping, tests, and schema updates where applicable.
- [ ] Any contract-count growth is justified in the PR body and linked to the approval that allows it.

## Risk
- [ ] Breaking change: explain in PR body and update compatibility docs.

## Ops/E2E Boundary Checklist (required if touching `ops/e2e/**`)
- [ ] Where should this fix live?
  - [ ] `ops/e2e` (scenario/assertion only)
  - [ ] `ops/run` (entrypoint/orchestration)
  - [ ] `ops/k8s` or `ops/stack` (deployment/infrastructure)
  - [ ] `ops/observe` or `ops/load` (telemetry/load-only)
- [ ] No direct infra patch/fixup was added under `ops/e2e/**`.
- [ ] Any setup, repair, or deployment logic moved to the owning surface outside `ops/e2e/**`.
- [ ] Automation boundary checks are green in `make ci-fast` or the linked CI run.
- [ ] `make ops-validate` is green, or the linked CI run includes the equivalent lane.
- [ ] Stack, kind, or smoke coverage ran in CI for this PR when deployment behavior can be affected.

## Docs Checklist (required if touching `docs/**`, `mkdocs.yml`, `README.md`, `SECURITY.md`, `CONTRIBUTING.md`, crate README files, or command docs output)
- [ ] Completed `.github/PULL_REQUEST_TEMPLATE/docs-governance.md` checklist items in PR description.
- [ ] Published docs stay within the curated docs spine (`docs/01-introduction` through `docs/08-contracts`, `docs/assets`, and `docs/index.md`) and do not add Markdown pages under `docs/_internal`.
- [ ] Crate docs rule respected (`README.md` + `CONTRACT.md` in crate root; crate docs checks pass).
