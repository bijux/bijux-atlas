## Ops/E2E Boundary Checklist

Use this template for pull requests touching `ops/e2e/**`. Keep `ops/e2e` limited to scenario
selection, assertions, and evidence wiring.

- [ ] Change belongs in the selected owner:
  - [ ] `ops/e2e` (scenario/assertion only)
  - [ ] `ops/run` (entrypoint/orchestration)
  - [ ] `ops/k8s` or `ops/stack` (deployment/infrastructure)
  - [ ] `ops/observe` or `ops/load` (telemetry/load-only)
- [ ] No direct infra patch/fixup was added under `ops/e2e/**`.
- [ ] Any setup, repair, or deployment logic moved to the owning surface outside `ops/e2e/**`.
- [ ] Automation boundary checks are green in `make ci-fast` or the linked CI run.
- [ ] `make ops-validate` is green, or the linked CI run includes the equivalent lane.
- [ ] Stack, kind, or smoke coverage ran in CI for this PR when deployment behavior can be affected.
