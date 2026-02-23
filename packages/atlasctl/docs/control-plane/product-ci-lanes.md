# Product CI Lane Mapping

Canonical product lanes and their atlasctl entrypoints:

- `product-smoke`: `./bin/atlasctl suite run product --only fast --report json`
- `product-build`: `./bin/atlasctl product build`
- `product-verify`: `./bin/atlasctl product verify`
- `product-release-dry`: `./bin/atlasctl product release-candidate --internal`

Notes:

- `product-release-dry` runs release gates (bypass inventory, ops contracts drift, pins, docs inventory) and performs build/verify/checksum-sign without publishing.
- `product docker release` remains a CI-only lane and is separate from `product-release-dry`.
