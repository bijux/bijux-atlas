# bijux-atlas

`bijux-atlas` is the product-facing Atlas crate. It owns the end-user CLI, the runtime HTTP server, the OpenAPI generator, and the contract-governed domain logic behind dataset, catalog, ingest, policy, and query workflows.

Use this crate when you need to:

- run Atlas workflows locally or in CI
- serve Atlas over HTTP
- generate or validate Atlas API contracts
- work on Atlas domain behavior, runtime configuration, or adapters

This crate is not the repository control plane. Repository governance, docs automation, registry maintenance, and workspace audits live in [`bijux-dev-atlas`](../bijux-dev-atlas/README.md).

## What Ships Here

- `bijux-atlas`: main CLI for dataset, catalog, ingest, policy, config, diff, garbage-collection, and OpenAPI workflows
- `bijux-atlas-server`: runtime server binary for Atlas HTTP traffic
- `bijux-atlas-openapi`: OpenAPI generation utility
- Rust modules for domain rules, application orchestration, adapters, contracts, and runtime configuration

## Supported Entry Points

- CLI users should start with `bijux-atlas`
- server operators should start with `bijux-atlas-server`
- API and schema consumers should start with the generated OpenAPI output and the documented API contracts
- Rust contributors should start from the canonical module roots: `adapters`, `app`, `contracts`, `domain`, and `runtime`

The supported surface is the documented CLI, server behavior, API contracts, and canonical module owners. Undocumented helper paths are implementation detail.

## Primary Workflows

- `config`: inspect and validate runtime configuration inputs
- `catalog`: validate, publish, roll back, and promote catalog artifacts
- `dataset`: verify dataset roots and dataset-level contracts
- `ingest`: run governed ingest flows and validation
- `diff`: compute governed dataset and catalog differences
- `gc`: plan and apply garbage collection for managed artifacts
- `policy`: validate and inspect policy-driven behavior
- `openapi`: generate API descriptions from the contract-owned surface

## Quick Start

Show the main CLI surface:

```bash
cargo run -p bijux-atlas --bin bijux-atlas -- --help
```

Validate a catalog artifact:

```bash
cargo run -p bijux-atlas --bin bijux-atlas -- catalog validate <path>
```

Verify a dataset root:

```bash
cargo run -p bijux-atlas --bin bijux-atlas -- dataset verify \
  --root <dir> \
  --release 110 \
  --species homo_sapiens \
  --assembly GRCh38
```

Inspect the server runtime surface:

```bash
cargo run -p bijux-atlas --bin bijux-atlas-server -- --help
```

Inspect the OpenAPI surface:

```bash
cargo run -p bijux-atlas --bin bijux-atlas -- openapi --help
```

## Runtime and Feature Flags

Important environment variables:

- `BIJUX_LOG_LEVEL`: overrides log verbosity
- `BIJUX_CACHE_DIR`: sets the shared cache directory used by Atlas workflows

Important Cargo features:

- `backend-local`: enables the local filesystem-backed store integration
- `backend-s3`: enables the S3-like store integration on top of the local backend support
- `jemalloc`: enables the optional allocator override
- `bench-ingest-throughput`: enables the heavier ingest benchmark targets

## Stability and Contract Policy

- top-level command names and noun-first command families are treated as contract surfaces
- `--json` output is deterministic and intended for CI snapshots and automation
- API errors, status mappings, and OpenAPI output are governed by contract tests
- runtime configuration is owned by config contracts and validators, not ad hoc scripts
- compatibility tests and contract tests are part of the supported maintenance model

The following are not stable API promises:

- undocumented helper functions
- convenience imports outside the canonical module owners
- benchmark-only or internal testing helpers

## Source Layout

The source tree is organized so ownership stays boring and obvious:

- `src/adapters`: inbound and outbound integrations such as CLI, HTTP, store, sqlite, redis, telemetry, and filesystem code
- `src/app`: use-case orchestration, ports, cache services, and server application state
- `src/contracts`: external schemas, runtime config contracts, and stable error definitions
- `src/domain`: business rules for dataset, query, ingest, policy, cluster, and security behavior
- `src/runtime`: runtime configuration and process-level setup

If a change affects transport or persistence details, it should usually land in `adapters`. If it changes business behavior, it should usually land in `domain`. If it changes an external schema or stable error surface, it belongs in `contracts`.

## Documentation Map

Repository docs in this worktree:

- crate documentation index: [../../docs/--archive/bijux-atlas-crate/index.md](../../docs/--archive/bijux-atlas-crate/index.md)
- CLI command reference: [../../docs/--archive/bijux-atlas-crate/cli-command-list.md](../../docs/--archive/bijux-atlas-crate/cli-command-list.md)
- CLI UX and output contract: [../../docs/--archive/bijux-atlas-crate/cli-ux-contract.md](../../docs/--archive/bijux-atlas-crate/cli-ux-contract.md)
- exit codes: [../../docs/--archive/bijux-atlas-crate/exit-codes.md](../../docs/--archive/bijux-atlas-crate/exit-codes.md)

API and integration surfaces:

- API docs index: [../../docs/--archive/api/index.md](../../docs/--archive/api/index.md)
- plugin contract: [../../docs/--archive/bijux-atlas-crate/plugin-contract.md](../../docs/--archive/bijux-atlas-crate/plugin-contract.md)
- crate API stability reference: [../../docs/--archive/reference/crate-api-stability.md](../../docs/--archive/reference/crate-api-stability.md)

## Using the Library

The primary supported interface is the CLI plus the documented server and API contracts. The Rust library is available for workspace use, tests, and integration code, but imports should stay anchored in the canonical owners:

- `bijux_atlas::adapters`
- `bijux_atlas::app`
- `bijux_atlas::contracts`
- `bijux_atlas::domain`
- `bijux_atlas::runtime`

Avoid depending on undocumented internal helpers or convenience paths outside those owners.

## Development Notes

- the crate forbids unsafe Rust
- deterministic behavior is an explicit design goal
- contract tests, compatibility tests, golden outputs, and surface guardrails are part of the maintenance model
- cache and artifact output are expected to live under the repository artifacts root, not crate-local scratch directories
- contributor changes should preserve the canonical ownership model of `adapters`, `app`, `contracts`, `domain`, and `runtime`
