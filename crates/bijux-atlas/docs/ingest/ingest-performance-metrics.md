# Ingest Performance Metrics

## Throughput Metrics

- `ingest_rows_per_second`: extracted rows committed to SQLite per second.
- `ingest_bytes_per_second`: source bytes consumed per second.

## Latency Metrics

- `ingest_total_latency_ms`: full ingest pipeline duration.
- `ingest_stage_prepare_ms`: prepare stage duration.
- `ingest_stage_decode_ms`: decode stage duration.
- `ingest_stage_extract_ms`: extract stage duration.
- `ingest_stage_persist_ms`: persist stage duration.
- `ingest_stage_finalize_ms`: finalize stage duration.

## Resource Metrics

- `ingest_rss_mb`: peak resident memory during ingest.
- `ingest_cpu_percent`: sampled process CPU percent during ingest.
- `ingest_io_bytes_per_second`: estimated read throughput from source files.
- `ingest_file_read_latency_ms`: latency to read source fixture files.
- `ingest_artifact_generation_latency_ms`: latency to emit SQLite and manifest artifacts.

## Dataset Scale Categories

- `small`: tiny fixture dataset.
- `medium`: realistic fixture dataset.
- `large`: strict realistic fixture profile.
- `sharded`: realistic dataset with contig sharding enabled.
- `concurrent`: two ingest runs executed in parallel.
