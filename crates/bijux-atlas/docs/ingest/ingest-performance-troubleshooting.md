# Ingest Performance Troubleshooting

## Throughput regression

- Compare `ingest_scenarios` results against baseline fixture.
- Check parser and SQLite write benches independently.

## Memory regression

- Inspect `ingest_resource_tracking` RSS samples.
- Verify fixture size and shard settings are unchanged.

## Sharding regression

- Run shard generation, catalog generation, and shard distribution benches together.
- Validate shard count and per-contig spread did not collapse.
