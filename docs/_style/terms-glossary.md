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

## Ops Terms

- `Stack`: dependency bring-up and fault injection foundation.
- `K8s`: chart install profiles, policies, and cluster validation.
- `Obs`: observability pack, signal contracts, dashboards, and drills.
- `Load`: k6 suites, scenarios, baselines, and load reports.
- `E2E`: composition-only workflows spanning stack + k8s + observe + datasets + load.
- `Fixture`: a committed or generated sample dataset used for deterministic validation.
- `Profile`: a named operational configuration selection such as `local`, `kind`, or `ci`.
- `Lane`: a named contract selection such as `local`, `pr`, `merge`, or `release`.

## Synonym Policy

- Use `store` for durable role naming; `minio` is implementation detail.
- Do not use `phase`, `step`, `task`, or `stage` in durable ops names.
