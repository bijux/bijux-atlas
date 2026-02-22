# Run INDEX

## Purpose
Single executable script surface for ops commands.

## Public entrypoints
- `./bin/atlasctl ops prereqs --report text`
- `./bin/atlasctl ops doctor --report text`
- `ops/run/ops-check.sh`
- `ops/run/ops-smoke.sh`
- `atlasctl ops stack up`
- `atlasctl ops stack down`
- `atlasctl ops warm`
- `atlasctl ops deploy`
- `atlasctl ops e2e run`

## Suites
- Entrypoints dispatch to suite manifests in area-specific directories.

## Contracts
- `ops/CONTRACT.md`
- `configs/ops/public-surface.json`
