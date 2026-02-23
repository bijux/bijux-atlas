# Verify A Release In CI

CI should use fast validation first, then release/publish lanes.

## Fast Validation (PR-safe)

1. `./bin/atlasctl product validate`
2. `./bin/atlasctl suite run product --only fast --report json`

## Release Validation (gated)

1. `./bin/atlasctl product build --plan`
2. `./bin/atlasctl product release-candidate --internal`
3. `CI=1 ./bin/atlasctl product docker release`

## Artifacts

- Product lane evidence: `artifacts/evidence/product/<lane>/<run_id>/...`
- Product artifact manifest: `artifacts/evidence/product/build/<run_id>/artifact-manifest.json`

## CI Lane Mapping

- `product-smoke`: `atlasctl suite run product --only fast`
- `product-build`: `atlasctl product build`
- `product-verify`: `atlasctl product verify`
- `product-release-dry`: `atlasctl product release-candidate --internal`
