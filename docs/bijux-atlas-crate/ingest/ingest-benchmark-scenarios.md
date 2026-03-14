# Ingest Benchmark Scenarios

## Scenario Matrix

- `ingest_small_dataset`: tiny fixture ingest throughput baseline.
- `ingest_medium_dataset`: realistic fixture ingest throughput baseline.
- `ingest_large_dataset`: strict ingest profile baseline.
- `ingest_sharded_dataset`: contig sharding ingest baseline.
- `ingest_concurrent_ingestion`: parallel ingest throughput baseline.

## Pipeline Focus Scenarios

- `gff3_parsing_throughput`: GFF3 parser throughput.
- `fasta_loading_throughput`: FASTA loading throughput.
- `sqlite_write_performance`: SQLite persistence performance.
- `manifest_generation_latency`: manifest generation latency.
- `ingest_resource_tracking`: memory, CPU, I/O, read latency, and artifact latency sampling.

## Validation and Overhead Scenarios

- `fai_validation_overhead`: FAI validation overhead.
- `transcript_extraction`: transcript extraction throughput.
- `db_size_growth`: artifact size growth trend.
- `sqlite_query_latency`: ingest output query latency sanity.
