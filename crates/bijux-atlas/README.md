# bijux-atlas

`bijux-atlas` is the published Atlas runtime crate for genomics dataset delivery.
It ingests GFF3 and FASTA inputs into immutable dataset artifacts, serves gene-query workflows
through CLI and HTTP surfaces, and exports the OpenAPI contract for those APIs.

This crate is the right starting point if you are looking for:

- GFF3 and FASTA ingest in Rust
- immutable genome annotation dataset artifacts
- gene and transcript query APIs
- a Rust HTTP server plus OpenAPI export for genomic datasets

## What Ships

- `bijux-atlas`: end-user CLI for dataset, catalog, ingest, diff, garbage-collection, config, and
  OpenAPI workflows
- `bijux-atlas-server`: runtime HTTP server for Atlas APIs
- `bijux-atlas-openapi`: OpenAPI export utility
- Rust library modules rooted in `adapters`, `app`, `contracts`, `domain`, and `runtime`

## How It Fits With `bijux-cli`

Atlas owns the genomic dataset runtime itself.
The sibling `bijux-cli` repository owns the umbrella command runtime that can route Atlas under
`bijux atlas ...` and `bijux dev atlas ...`.

Use this crate when you want the Atlas runtime and libraries directly.
Use `bijux-cli` when you want a shared command root that can host Atlas alongside other Bijux tools.

## Install and Verify

Choose one install route at a time.

Install the published crate directly when you want the Atlas binaries or crate APIs without the
umbrella runtime:

```bash
cargo install --locked bijux-atlas
```

Verify the installed runtime surfaces:

```bash
bijux-atlas --help
bijux-atlas version
bijux-atlas-server --help
bijux-atlas-openapi --help
```

Run the current checkout directly:

```bash
cargo run -p bijux-atlas --bin bijux-atlas -- --help
cargo run -p bijux-atlas --bin bijux-atlas-server -- --help
cargo run -p bijux-atlas --bin bijux-atlas-openapi -- --out ./openapi.json
```

## Documentation

- Product documentation: <https://bijux.github.io/bijux-atlas/>
- Rust API documentation: <https://docs.rs/bijux-atlas/latest/bijux_atlas/>
- Source repository: <https://github.com/bijux/bijux-atlas>
- Maintainer control plane: <https://github.com/bijux/bijux-atlas/tree/main/crates/bijux-dev-atlas>

The GitHub Pages site is the human-facing documentation surface. `docs.rs` is the API reference
for the Rust crate itself.

## Scope

Use this crate when you need to:

- build immutable genomic dataset artifacts from GFF3 and FASTA inputs
- run Atlas dataset and catalog workflows locally or in CI
- serve Atlas through the HTTP runtime
- generate the published OpenAPI description
- integrate against the crate-owned domain, contract, and runtime modules

This crate does not own repository governance, release automation, or documentation publishing.
Those maintainer workflows live in `bijux-dev-atlas` and the repository-level docs and ops
contracts.

## Main Workflows

- `config`: inspect and validate runtime configuration inputs
- `catalog`: validate, publish, roll back, and promote catalog artifacts
- `dataset`: verify dataset roots and dataset-level contracts
- `ingest`: build governed ingest artifacts from source datasets
- `diff`: compare dataset and catalog artifacts
- `gc`: plan and apply garbage collection for managed artifacts
- `policy`: validate and inspect policy-governed behavior
- `openapi`: export the API contract from the runtime-owned surface

## Feature Flags

- `backend-local`: enable the local filesystem-backed store integration
- `backend-s3`: enable the S3-like store integration on top of the local backend support
- `jemalloc`: enable the optional allocator override
- `bench-ingest-throughput`: enable the heavier ingest benchmark targets

## Stability and Contract Policy

- Top-level command names and documented noun-first command families are treated as release
  surfaces.
- `--json` output is deterministic and intended for CI snapshots and automation.
- API errors, status mappings, and OpenAPI output are governed by contract tests.
- Runtime configuration is owned by contracts and validators, not ad hoc scripts.
- Compatibility tests, contract tests, and golden outputs are part of the supported maintenance
  model.

The following are not stable API promises:

- undocumented helper functions
- convenience imports outside the canonical module owners
- benchmark-only or internal testing helpers

## Source Layout

- `src/adapters`: inbound and outbound integrations such as CLI, HTTP, store, sqlite, redis,
  telemetry, and filesystem code
- `src/app`: use-case orchestration, ports, cache services, and server application state
- `src/contracts`: external schemas, runtime config contracts, and stable error definitions
- `src/domain`: business rules for dataset, query, ingest, policy, cluster, and security behavior
- `src/runtime`: runtime configuration and process-level setup

If a change affects transport or persistence details, it usually belongs in `adapters`. If it
changes business behavior, it usually belongs in `domain`. If it changes an external schema or
stable error surface, it belongs in `contracts`.
