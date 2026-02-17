# Realistic Public Sample Datasets

SSOT manifest: `datasets/real-datasets.json`.

This manifest pins:
- dataset IDs (`release/species/assembly`)
- source archive
- SHA256 checksum
- deterministic derivation flow for secondary releases used in diff regression

## Fetch
- `make fetch-real-datasets`

Outputs are placed under `artifacts/real-datasets/`.

## Notes
- `110/homo_sapiens/GRCh38` is the pinned downloadable sample.
- `111/homo_sapiens/GRCh38` is derived deterministically from 110 using `scripts/fixtures/derive-release-111.sh` for stable diff regression.
