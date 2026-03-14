# Ingest Benchmark CI

Workflow: `.github/workflows/ingest-benchmark-ci.yml`

## Pipeline Steps

1. Build all ingest benchmark binaries.
2. Run ingest benchmark regression fixture tests.

This lane prevents benchmark surface drift and guarantees reproducible benchmark executables are kept buildable.
