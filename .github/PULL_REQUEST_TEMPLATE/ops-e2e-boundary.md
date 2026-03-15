## Ops/E2E Boundary Checklist

Use this template for pull requests touching `ops/e2e/**`.

- [ ] Where should this fix live?
  - [ ] `ops/e2e` (scenario/assertion only)
  - [ ] `ops/run` (entrypoint/orchestration)
  - [ ] `ops/k8s` or `ops/stack` (deployment/infrastructure)
  - [ ] `ops/observe` or `ops/load` (telemetry/load-only)
- [ ] No direct infra patch/fixup was added under `ops/e2e/**`.
- [ ] Automation boundary checks are green in `make ci-fast` or the linked CI run.
- [ ] Stack/k8s smoke coverage ran in CI for this PR.
