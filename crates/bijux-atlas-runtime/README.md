# bijux-atlas-runtime

`bijux-atlas-runtime` is the canonical Atlas runtime composition crate for
genomics dataset delivery. It composes ingest, query, store, API, and runtime
wiring so the direct CLI and server owner crates can expose stable Atlas
product surfaces, while the shorter `bijux-atlas` package remains a
compatibility alias for historical Rust imports.

This crate is the right starting point if you are looking for:

- GFF3 and FASTA ingest in Rust
- immutable genome annotation dataset artifacts
- gene and transcript query APIs
- a Rust HTTP server plus OpenAPI export for genomic datasets

## What Ships

- `bijux-atlas-cli`: end-user CLI owner for dataset, catalog, ingest, diff, garbage-collection,
  config, and OpenAPI workflows
- `bijux-atlas-server`: runtime HTTP server owner for Atlas APIs
- `bijux-atlas-api`: OpenAPI export owner for `bijux-atlas-openapi`
- `bijux-atlas`: compatibility alias crate for the historical `bijux_atlas`
  import path
- Rust library modules rooted in `adapters`, `app`, `contracts`, `domain`, and
  `runtime`

## How It Fits With `bijux-cli`

Atlas owns the genomic dataset runtime itself. The sibling `bijux-cli`
repository owns the umbrella command runtime that can route Atlas under
`bijux atlas ...` and `bijux dev atlas ...` when that command surface is
already installed in an environment.

Use this crate when you want the canonical Atlas runtime and libraries
directly.
Use `bijux-cli` when you want a shared command root that can host Atlas alongside other Bijux tools.

## Install and Verify

Choose one install route at a time.

Install the published binaries directly when you want Atlas without the
umbrella runtime:

```bash
cargo install --locked bijux-atlas-cli --bin bijux-atlas
cargo install --locked bijux-atlas-server --bin bijux-atlas-server
cargo install --locked bijux-atlas-api --bin bijux-atlas-openapi
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
cargo run -p bijux-atlas-cli --bin bijux-atlas -- --help
cargo run -p bijux-atlas-server --bin bijux-atlas-server -- --help
cargo run -p bijux-atlas-api --bin bijux-atlas-openapi -- --out ./openapi.json
```

## Documentation

- Product documentation: <https://bijux.github.io/bijux-atlas/>
- Rust API documentation:
  <https://docs.rs/bijux-atlas-runtime/latest/bijux_atlas_runtime/>
- Source repository: <https://github.com/bijux/bijux-atlas>
- Maintainer control plane: <https://github.com/bijux/bijux-atlas/tree/main/crates/bijux-atlas-dev>

The GitHub Pages site is the human-facing documentation surface. `docs.rs` is the API reference
for the Rust crate itself.

## Scope

Use this crate when you need to:

- build immutable genomic dataset artifacts from GFF3 and FASTA inputs
- run Atlas dataset and catalog workflows locally or in CI
- serve Atlas through the HTTP runtime via `bijux-atlas-server`
- generate the published OpenAPI description via `bijux-atlas-api`
- integrate against the crate-owned domain, contract, and runtime modules

This crate does not own repository governance, release automation, or
documentation publishing. Those maintainer workflows live in
`bijux-atlas-dev` and the repository-level docs and ops contracts.

## Main Workflows

- `config`: inspect and validate runtime configuration inputs
- `catalog`: validate, publish, roll back, and promote catalog artifacts
- `dataset`: verify dataset roots and dataset-level contracts
- `ingest`: build governed ingest artifacts from source datasets
- `diff`: compare dataset and catalog artifacts
- `gc`: plan and apply garbage collection for managed artifacts
- `policy`: validate and inspect policy-governed behavior
- `openapi`: export the API contract through the API-owned binary surface

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
- API-facing HTTP contract, response-shape, and observability suites are owned in `bijux-atlas-api`; startup, cache, backend, and server-wiring suites are owned in `bijux-atlas-server`; runtime keeps only shared library, compatibility, and boundary-contract tests.
- Runtime configuration is owned by contracts and validators, not ad hoc scripts.
- Compatibility tests, contract tests, and golden outputs are part of the supported maintenance
  model.

The following are not stable API promises:

- undocumented helper functions
- convenience imports outside the canonical module owners
- benchmark-only or internal testing helpers

## Scientific Annotation Handling

- Atlas uses 1-based closed genomic coordinates across ingest, query, and export contracts.
- Partial or missing annotation structures are retained and classified explicitly in canonical
  completeness fields instead of being silently normalized away.
- Biotype derivation records attribute-key provenance in ingest evidence so downstream users can
  distinguish source-provided annotations from fallback-derived values.
- Ambiguous scientific signals such as unresolved biotypes or conflicting normalized contig sources
  are emitted as first-class evidence and block publication under strict publish gates.

## Source Layout

- `src/adapters`: runtime-owned outbound integrations such as filesystem and
  store-facing adapters
- `src/app`: orchestration, ports, cache coordination, and process-facing
  runtime services
- `src/contracts`: runtime config contracts and stable error definitions
- `src/domain`: cluster, policy, security, and other runtime-owned domain
  behavior
- `src/runtime`: runtime configuration and process-level setup

CLI entrypoints live in `bijux-atlas-cli`, HTTP entrypoints live in
`bijux-atlas-server`, OpenAPI export lives in `bijux-atlas-api`, and the
historical Rust import path lives in the `bijux-atlas` compatibility crate.

If a change affects transport or persistence details, it usually belongs in `adapters`. If it
changes business behavior, it usually belongs in `domain`. If it changes an external schema or
stable error surface, it belongs in `contracts`.
