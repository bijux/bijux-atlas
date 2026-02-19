# Scripts Surface

Generated file. Do not edit manually.

Scripts are internal unless listed in `configs/ops/public-surface.json` or `scripts/ENTRYPOINTS.md` public section.

## Script Domains

- `scripts/check/`: validators and lint gates
- `scripts/gen/`: inventory/document generators
- `scripts/ci/`: CI glue only
- `scripts/dev/`: local helpers
- `scripts/lib/`: reusable shell/python libraries
- `scripts/bin/`: thin entrypoints

## scripts/bin

- `scripts/bin/bijux-atlas-dev`
- `scripts/bin/bijux-atlas-ops`
- `scripts/bin/isolate`
- `scripts/bin/require-isolate`

## checks

- `scripts/check/README.md`
- `scripts/check/check-bin-entrypoints.py`
- `scripts/check/check-docker-layout.py`
- `scripts/check/check-docker-policy.py`
- `scripts/check/check-no-latest-tags.py`
- `scripts/check/check-python-lock.py`
- `scripts/check/check-script-errors.py`
- `scripts/check/check-script-help.py`
- `scripts/check/check-script-ownership.py`
- `scripts/check/check-script-tool-guards.py`
- `scripts/check/check-script-write-roots.py`
- `scripts/check/docker-runtime-smoke.sh`
- `scripts/check/docker-scan.sh`
- `scripts/check/no-direct-path-usage.sh`
- `scripts/check/no-duplicate-script-names.sh`
