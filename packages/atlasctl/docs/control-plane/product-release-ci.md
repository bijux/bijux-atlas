# Verify A Release In CI

CI should use fast validation first, then release/publish lanes.

## Fast Validation (PR-safe)

1. `./bin/atlasctl product validate`
2. `./bin/atlasctl suite run product --only fast --report json`

## Release Validation (gated)

1. `CI=1 ./bin/atlasctl product docker release`
2. `./bin/atlasctl product build`
3. `./bin/atlasctl product validate`

## Artifacts

- Product lane evidence: `artifacts/evidence/product/<lane>/<run_id>/...`
- Product artifact manifest: `artifacts/evidence/product/build/<run_id>/artifact-manifest.json`

