# Ops Surface

Generated from ops manifests.

## Stable Entrypoints

- `make ops-help`
- `make ops-surface`
- `make ops-layout-lint`
- `make ops-prereqs`
- `make ops-doctor`
- `make ops-lint`
- `make ops-fmt`
- `make ops-gen`
- `make ops-gen-check`
- `make ops-contracts-check`
- `make ops-e2e-validate`
- `make ops-stack-up`
- `make ops-stack-down`
- `make ops-obs-up`
- `make ops-obs-verify`
- `make ops-datasets-verify`
- `make ops-e2e-smoke`
- `make ops-k8s-suite`
- `make ops-load-suite`
- `make ops-deploy`
- `make ops-undeploy`
- `make ops-k8s-tests`
- `make ops-observability-validate`
- `make ops-load-smoke`
- `make ops-dataset-qc`
- `make ops-realdata`
- `make ops-ci-fast`
- `make ops-ci-nightly`
- `make ops-ref-grade-local`
- `make ops-full`

## E2E Scenarios

- `smoke`: `make ops-e2e-smoke` (stack=True, obs=True, datasets=True, load=True)
- `k8s-suite`: `make ops-k8s-suite` (stack=True, obs=True, datasets=False, load=False)
- `realdata`: `make ops-realdata` (stack=True, obs=True, datasets=True, load=False)
- `perf-e2e`: `make ops-perf-e2e` (stack=True, obs=True, datasets=True, load=True)
