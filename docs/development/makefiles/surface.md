# Makefiles Public Surface

- Owner: `docs-governance`

## What

Defines stable make target interfaces exported by the repository root `Makefile`.

## Why

Make targets are operational interfaces used by CI and local workflows.

## Scope

Public targets printed by `make help`.

## Non-goals

Does not document internal helper targets prefixed with `_`.

## Contracts

Stable targets:

- `fmt`
- `lint`
- `check`
- `test`
- `test-all`
- `coverage`
- `audit`
- `openapi-drift`
- `ci`
- `fetch-fixtures`
- `fetch-real-datasets`
- `load-test`
- `load-test-1000qps`
- `cold-start-bench`
- `memory-profile-load`
- `run-medium-ingest`
- `run-medium-serve`
- `crate-structure`
- `crate-docs-contract`
- `cli-command-surface`
- `culprits-all`
- `culprits-max_loc`
- `culprits-max_depth`
- `culprits-file-max_rs_files_per_dir`
- `culprits-file-max_modules_per_dir`
- `e2e-local`
- `e2e-k8s-install-gate`
- `e2e-k8s-suite`
- `e2e-perf`
- `e2e-realdata`
- `ops-up`
- `ops-stack-up`
- `ops-down`
- `ops-stack-down`
- `ops-stack-validate`
- `ops-stack-smoke`
- `ops-stack-health-report`
- `ops-stack-version`
- `ops-stack-uninstall`
- `ops-stack-slow-store`
- `ops-reset`
- `ops-env-print`
- `ops-cluster-sanity`
- `ops-publish-medium`
- `ops-publish`
- `ops-deploy`
- `ops-offline`
- `ops-perf`
- `ops-multi-registry`
- `ops-ingress`
- `ops-warm`
- `ops-soak`
- `ops-smoke`
- `ops-metrics-check`
- `ops-traces-check`
- `ops-k8s-tests`
- `ops-k8s-template-tests`
- `ops-load-prereqs`
- `ops-load-smoke`
- `ops-load-full`
- `ops-drill-store-outage`
- `ops-drill-minio-outage`
- `ops-drill-prom-outage`
- `ops-drill-otel-outage`
- `ops-drill-toxiproxy-latency`
- `ops-drill-overload`
- `ops-drill-memory-growth`
- `ops-drill-corruption`
- `ops-drill-pod-churn`
- `ops-drill-upgrade`
- `ops-drill-rollback`
- `ops-upgrade-drill`
- `ops-rollback-drill`
- `ops-realdata`
- `ops-report`
- `ops-script-coverage`
- `ops-shellcheck`
- `ops-kind-version-check`
- `ops-k6-version-check`
- `ops-helm-version-check`
- `ops-kubectl-version-check`
- `ops-kubeconform-version-check`
- `ops-tool-check`
- `ops-tools-check`
- `ops-values-validate`
- `ops-openapi-validate`
- `ops-dashboards-validate`
- `ops-alerts-validate`
- `ops-release-matrix`
- `ops-ci`
- `ops-ci-nightly`
- `ops-clean`
- `ops-perf-prepare-store`
- `ops-perf-e2e`
- `ops-perf-nightly`
- `ops-perf-cold-start`
- `ops-perf-cold-start-prefetch-5pods`
- `ops-perf-compare-redis`
- `ops-perf-suite`
- `ops-baseline-policy-check`
- `ops-observability-validate`
- `ops-observability-smoke`
- `ops-observability-pack-tests`
- `ops-observability-pack-lint`
- `ops-obs-up`
- `ops-obs-down`
- `ops-obs-mode`
- `ops-obs-mode-minimal`
- `ops-obs-mode-full`
- `ops-drill-alerts`
- `ops-drill-overload`
- `ops-drill-memory-growth`
- `ops-slo-burn`
- `ssot-check`
- `observability-check`
- `docs`
- `docs-serve`
- `docs-freeze`
- `docs-hardening`
- `layout-check`
- `layout-migrate`
- `bootstrap`
- `bootstrap-tools`
- `scripts-index`
- `docker-build`
- `docker-smoke`
- `chart-package`
- `chart-verify`
- `no-direct-scripts`
- `doctor`
- `help`

Perf targets:

- `perf-nightly`

Dev targets:

- `dev-fmt`
- `dev-lint`
- `dev-check`
- `dev-test`
- `dev-test-all`
- `dev-coverage`
- `dev-audit`
- `dev-ci`
- `dev-clean`

## Failure modes

Undocumented target changes break CI, scripts, or developer workflows.

## How to verify

```bash
$ make help
$ python3 scripts/docs/check_make_targets_documented.py
```

Expected output: make target documentation check passes.

## See also

- [Repo Surface](../repo-surface.md)
- [Scripts Index](../scripts/INDEX.md)
- [Make Targets Inventory](../make-targets-inventory.md)
- [Crate Layout Contract](../../architecture/crate-layout-contract.md)
