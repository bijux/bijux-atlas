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
- `111/homo_sapiens/GRCh38` is derived deterministically from 110 using `scripts/areas/fixtures/derive-release-111.sh` for stable diff regression.

## What

Reference definition for this topic.

## Why

Defines stable semantics and operational expectations.

## Scope

Applies to the documented subsystem behavior only.

## Non-goals

Does not define unrelated implementation details.

## Contracts

Normative behavior and limits are listed here.

## Failure modes

Known failure classes and rejection behavior.

## How to verify

```bash
$ make docs
```

Expected output: docs checks pass.

## See also

- [Reference Index](INDEX.md)
- [Contracts Index](../../contracts/contracts-index.md)
- [Terms Glossary](../../_style/terms-glossary.md)
