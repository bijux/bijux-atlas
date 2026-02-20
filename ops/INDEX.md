# Ops Canonical Entrypoints

Public ops automation entrypoints are `ops/run/*` wrappers only.

## Canonical Wrappers

- `ops/run/stack-up.sh`
- `ops/run/stack-down.sh`
- `ops/run/down.sh`
- `ops/run/deploy-atlas.sh`
- `ops/run/k8s-restart.sh`
- `ops/run/k8s-apply-config.sh`
- `ops/run/e2e.sh`
- `ops/run/e2e-smoke.sh`
- `ops/run/k8s-tests.sh`
- `ops/run/k8s-suite.sh`
- `ops/run/load-suite.sh`
- `ops/run/load-smoke.sh`
- `ops/run/obs-up.sh`
- `ops/run/obs-verify.sh`
- `ops/run/obs-validate.sh`
- `ops/run/ops-check.sh`
- `ops/run/ops-smoke.sh`
- `ops/run/prereqs.sh`
- `ops/run/doctor.sh`
- `ops/run/report.sh`
- `ops/run/root-lanes.sh`
- `ops/run/root-local.sh`
- `ops/run/warm.sh`
- `ops/run/warm-entrypoint.sh`
- `ops/run/warm-dx.sh`
- `ops/run/cache-status.sh`
- `ops/run/cache-prune.sh`
- `ops/run/evidence-open.sh`
- `ops/run/evidence-bundle.sh`
- `ops/run/artifacts-open.sh`
- `ops/run/configmap-drift-report.sh`
- `ops/run/contract-check.sh`
- `ops/run/contract-report.py`
- `ops/run/clean.sh`

## Policy

- Do not call `ops/*/scripts/*.sh` directly from docs, CI workflows, or root/ci makefiles.
- Legacy `legacy/*` or `*-legacy` entrypoint names are forbidden.
