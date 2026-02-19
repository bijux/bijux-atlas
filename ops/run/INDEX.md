# Run INDEX

## Purpose
Single executable script surface for ops commands.

## Public entrypoints
- `ops/run/prereqs.sh`
- `ops/run/doctor.sh`
- `ops/run/ops-check.sh`
- `ops/run/ops-smoke.sh`
- `ops/run/stack-up.sh`
- `ops/run/stack-down.sh`
- `ops/run/deploy-atlas.sh`
- `ops/run/obs-up.sh`
- `ops/run/obs-verify.sh`
- `ops/run/datasets-verify.sh`
- `ops/run/e2e-smoke.sh`
- `ops/run/k8s-suite.sh`
- `ops/run/load-suite.sh`

## Suites
- Entrypoints dispatch to suite manifests in area-specific directories.

## Contracts
- `ops/CONTRACT.md`
- `configs/ops/public-surface.json`
