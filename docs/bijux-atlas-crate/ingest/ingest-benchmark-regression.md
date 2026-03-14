# Ingest Benchmark Regression Guard

Regression checks are defined in `tests/ingest_benchmark_regression.rs`.

## Guard Rules

- Baseline fixture must include small, medium, and large scenarios.
- Golden fixture must explicitly enable required overhead and scaling checks.
- CI must fail when benchmark fixture contracts regress.
