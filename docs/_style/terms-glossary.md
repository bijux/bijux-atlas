# Terms Glossary

- `Release`: upstream annotation/version identifier (example: `110`).
- `Dataset`: unique tuple of `release + species + assembly`.
- `Artifact`: immutable output set for one dataset.
- `Catalog`: index of published datasets and artifact pointers.
- `Registry`: one or more catalog sources merged deterministically.
- `Store`: backend that serves catalog/manifests/artifacts.
- `Shard`: optional partition of dataset SQLite for scale.
- `Budget`: configured hard limit for work, bytes, latency, or retries.
