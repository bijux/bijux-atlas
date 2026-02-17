# Crates Map

- Owner: `docs-governance`

## What

Repository map of crates and their primary purpose.

## Why

Provides a stable entrypoint for navigating crate responsibilities.

## Scope

Covers workspace crates under `crates/`.

## Non-goals

Does not replace crate-level architecture docs.

## Contracts

- `bijux-atlas-core`: deterministic primitives, canonicalization, error types.
- `bijux-atlas-model`: domain and artifact data types.
- `bijux-atlas-policies`: runtime policy schema and validation.
- `bijux-atlas-store`: artifact backends and integrity boundaries.
- `bijux-atlas-ingest`: deterministic ingest pipeline to artifacts.
- `bijux-atlas-query`: query planning, limits, pagination.
- `bijux-atlas-api`: wire contracts and request/response schemas.
- `bijux-atlas-server`: runtime orchestration and effectful serving.
- `bijux-atlas-cli`: plugin CLI and operational commands.

## Failure modes

Unclear crate ownership causes boundary drift and coupling.

## How to verify

```bash
$ ./scripts/docs/check_crate_docs_contract.sh
```

Expected output: crate docs contract check passes.

## See also

- [Crate Layout Contract](../architecture/crate-layout-contract.md)
- [Crate Boundary Graph](../architecture/crate-boundary-dependency-graph.md)
- [Terms Glossary](../_style/terms-glossary.md)
