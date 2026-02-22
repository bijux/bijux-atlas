# Run INDEX

## Purpose
Single executable script surface for ops commands.

## Public entrypoints
- `ops/run/prereqs.sh`
- `ops/run/doctor.sh`
- `ops/run/ops-check.sh`
- `ops/run/ops-smoke.sh`
- `atlasctl ops stack up`
- `atlasctl ops stack down`
- `ops/run/warm-entrypoint.sh`
- `atlasctl ops deploy`
- `atlasctl ops e2e run`

## Suites
- Entrypoints dispatch to suite manifests in area-specific directories.

## Contracts
- `ops/CONTRACT.md`
- `configs/ops/public-surface.json`
