# Scripts Surface

Generated file. Do not edit manually.

Scripts are internal unless listed in `configs/ops/public-surface.json` or `scripts/areas/docs/ENTRYPOINTS.md` public section.

## Script Domains

- `scripts/areas/check/`: validators and lint gates
- `scripts/areas/gen/`: inventory/document generators
- `scripts/areas/ci/`: CI glue only
- `scripts/areas/dev/`: local helpers
- `scripts/lib/`: reusable shell/python libraries
- `scripts/bin/`: thin entrypoints

## scripts/bin

- `scripts/bin/bijux-atlas-dev`
- `scripts/bin/bijux-atlas-ops`
- `scripts/bin/bijux-atlas-scripts`
- `scripts/bin/isolate`
- `scripts/bin/make_explain`
- `scripts/bin/make_graph`
- `scripts/bin/render_public_help`
- `scripts/bin/require-isolate`
- `scripts/bin/run_drill.sh`

## checks

- `scripts/areas/check/INDEX.md`
- `scripts/areas/check/README.md`
- `scripts/areas/check/check-atlas-scripts-cli-contract.py`
- `scripts/areas/check/check-bijux-atlas-scripts-boundaries.py`
- `scripts/areas/check/check-bin-entrypoints.py`
- `scripts/areas/check/check-docker-image-size.py`
- `scripts/areas/check/check-docker-layout.py`
- `scripts/areas/check/check-docker-policy.py`
- `scripts/areas/check/check-no-adhoc-python.py`
- `scripts/areas/check/check-no-direct-python-invocations.py`
- `scripts/areas/check/check-no-latest-tags.py`
- `scripts/areas/check/check-no-make-scripts-references.py`
- `scripts/areas/check/check-no-python-executable-outside-tools.py`
- `scripts/areas/check/check-python-lock.py`
- `scripts/areas/check/check-python-migration-exceptions-expiry.py`
- `scripts/areas/check/check-repo-script-boundaries.py`
- `scripts/areas/check/check-script-errors.py`
- `scripts/areas/check/check-script-help.py`
- `scripts/areas/check/check-script-ownership.py`
- `scripts/areas/check/check-script-shim-expiry.py`
- `scripts/areas/check/check-script-shims-minimal.py`
- `scripts/areas/check/check-script-tool-guards.py`
- `scripts/areas/check/check-script-write-roots.py`
- `scripts/areas/check/check-scripts-lock-sync.py`
- `scripts/areas/check/check_duplicate_script_names.py`
- `scripts/areas/check/docker-runtime-smoke.sh`
- `scripts/areas/check/docker-scan.sh`
- `scripts/areas/check/generate-scripts-sbom.py`
- `scripts/areas/check/no-direct-path-usage.sh`
- `scripts/areas/check/no-duplicate-script-names.sh`
- `scripts/areas/check/python_migration_exceptions.py`

## root bin shims

- `bin/make_explain`
- `bin/make_graph`
- `bin/render_public_help`
