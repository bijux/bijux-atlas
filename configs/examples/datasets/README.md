# Dataset example configs

These directories hold tutorial and documentation datasets that exercise Atlas flows without claiming production realism.

Use them to:
- verify ingest and query flows locally
- drive tutorial screenshots and examples
- exercise pagination, filtering, and schema handling with deterministic sample data

Do not use them to infer:
- production dataset scale
- source-data coverage
- runtime performance guarantees

The dataset shape contract lives in `specification.md`. Authoritative runtime dataset policies live under `configs/sources/runtime/datasets/`.
