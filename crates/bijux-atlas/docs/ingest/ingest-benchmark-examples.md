# Ingest Benchmark Examples

## Build benchmark binaries

`cargo bench -p bijux-atlas --features bench-ingest-throughput --no-run`

## Run one benchmark

`cargo bench -p bijux-atlas --features bench-ingest-throughput --bench ingest_scenarios`

## Run regression fixtures

`cargo test -p bijux-atlas --test ingest -- ingest_benchmark_regression --nocapture`
