# Ingest Benchmark Examples

## Build benchmark binaries

`cargo bench -p bijux-atlas-ingest --features bench-ingest-throughput --no-run`

## Run one benchmark

`cargo bench -p bijux-atlas-ingest --features bench-ingest-throughput ingest_scenarios`

## Run regression fixtures

`cargo test -p bijux-atlas-ingest ingest_benchmark_regression -- --nocapture`
