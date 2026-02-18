# Sharding Evolution: Monolith To Shards

## What

Sharding is an ingest-time layout choice with a router artifact:

- `none`: monolithic `gene_summary.sqlite`.
- `contig`: `catalog_shards.json` + `gene_summary.<shard>.sqlite` files.
- `region_grid`: reserved for future implementation.

Router metadata (`catalog_shards.json`) is the contract used by query/runtime:

- dataset identity
- shard mode
- `seqid -> shard` mapping
- per-shard sqlite checksum

## Why

Allows scaling large releases without changing API request/response semantics.

## Scope

Applies to ingest artifact layout and runtime shard routing only.

## Non-goals

Does not define a separate sharding API surface.

## Contracts

- Shard naming is deterministic and sorted:
  - contig mode uses canonical seqid keys.
  - partitioned mode uses `pNNN` fixed-width identifiers.
- `max_shards` policy is enforced to block pathological splits.
- `max_open_shards_per_pod` caps runtime fan-out pressure.
- Query semantics must be identical between `none` and `contig` modes.

## Failure modes

- `region_grid` plan currently fails fast as not implemented.
- Exceeding `max_shards` fails ingest deterministically.
- Missing/invalid router metadata disables shard fan-out for that dataset.

## How to verify

```bash
$ make ingest-sharded-medium
$ make ops-diff-smoke
```

Expected output: sharded ingest artifacts are produced and deterministic ops checks pass.

## See also

- [Reference Index](INDEX.md)
- [Contracts Index](../../contracts/contracts-index.md)
- [Sharding Schema](../../_generated/contracts/SHARDING_SCHEMA.md)
- [Terms Glossary](../../_style/terms-glossary.md)
