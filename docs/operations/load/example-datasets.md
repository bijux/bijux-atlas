# Load Test Example Datasets

- Owner: `bijux-atlas-operations`
- Type: `reference`
- Audience: `operator`
- Stability: `stable`

## Purpose

Describe example datasets used for deterministic load harness exercises.

## Dataset Catalog

- `ops/load/data/query-sample.jsonl`: representative read-heavy request corpus.
- `ops/load/data/ingest-sample.jsonl`: representative ingest payload corpus.

## Usage

- Use the query sample for query-path baseline and burst scenarios.
- Use the ingest sample for sustained ingest and mixed workload scenarios.
- Keep sample datasets immutable to preserve reproducibility across CI runs.
