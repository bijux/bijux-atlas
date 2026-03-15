# Dataset Fixtures

## Purpose
Own the committed sample datasets used to verify dataset policies, release diffs, and operator-facing examples.

## What Lives Here
- Versioned fixture roots such as `medium/v1` and `release-diff/v1`
- `manifest.lock` files that pin downloadable fixture archives
- Query and response examples that prove dataset behavior against committed samples

## What Must Not Live Here
- Runtime code
- Ad hoc scratch files
- Unpinned binary blobs outside `assets/*.tar.gz`

## See Also
- `ops/datasets/CONTRACT.md`
- `ops/datasets/fixture-policy.json`
- `ops/datasets/generated/fixture-inventory.json`
