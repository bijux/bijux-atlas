# Terms Glossary

- `Release`: upstream annotation/version identifier (example: `110`).
- `Release-indexed`: all dataset/materialization identity is anchored by release.
- `Dataset`: unique tuple of `release + species + assembly`.
- `SSOT`: single source of truth contract registry under `docs/contracts/*.json`.
- `Artifact`: immutable output set for one dataset.
- `Catalog`: index of published datasets and artifact pointers.
- `Registry`: one or more catalog sources merged deterministically.
- `Store`: backend that serves catalog/manifests/artifacts.
- `Shard`: optional partition of dataset SQLite for scale.
- `Budget`: configured hard limit for work, bytes, latency, or retries.
