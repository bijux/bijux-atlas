# Crates Map

- Owner: `docs-governance`

## What

Generated map of workspace crates and primary purpose.

## Why

Provides a stable navigation index for crate responsibilities.

## Scope

Covers workspace crates from `Cargo.toml` members under `crates/`.

## Non-goals

Does not replace crate-level architecture and API docs.

## Contracts
- `bijux-atlas-api`: wire contracts and request/response schemas.
- `bijux-atlas-cli`: plugin CLI and operational commands.
- `bijux-atlas-core`: deterministic primitives, canonicalization, error types.
- `bijux-atlas-ingest`: deterministic ingest pipeline to artifacts.
- `bijux-atlas-model`: domain and artifact data types.
- `bijux-atlas-policies`: runtime policy schema and validation.
- `bijux-atlas-query`: query planning, limits, and pagination.
- `bijux-atlas-server`: runtime orchestration and effectful serving.
- `bijux-atlas-store`: artifact backends and integrity boundaries.

## Failure modes

Stale maps can hide ownership drift and boundary violations.

## How to verify

```bash
$ bijux dev atlas docs generate-crates-map
 docs crate-docs-contract-check
```

Expected output: crates map is regenerated and crate docs contract passes.

## See also

- [Crate Boundary Graph](crate-boundary-dependency-graph.md)
- [Terms Glossary](../_style/terms-glossary.md)
