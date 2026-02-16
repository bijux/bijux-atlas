# Dataset Lifecycle

State machine:

1. Ingest: parse/validate raw inputs and compute derived artifacts.
2. Publish: atomically publish manifest + SQLite + checksums.
3. Serve: API reads only published immutable artifacts.
4. Deprecate: mark dataset as deprecated in catalog without mutating historical artifacts.
