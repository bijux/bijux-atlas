# bijux-atlas

`bijux-atlas` is the Atlas user-facing crate. It owns the CLI, the HTTP server binary, the OpenAPI generator, and the contract-driven domain logic behind dataset, catalog, ingest, policy, and query workflows.

This crate is for:

- operators and developers running Atlas commands locally or in CI
- consumers of the Atlas HTTP server and OpenAPI contract
- contributors working on Atlas domain rules, runtime configuration, and adapter behavior

This crate is not the repository control plane. Repository governance, docs automation, policy audits, and maintenance workflows live in [`bijux-dev-atlas`](../bijux-dev-atlas/README.md).

## What This Crate Owns

- user-facing CLI commands under `bijux-atlas`
- server runtime and HTTP adapters under `bijux-atlas-server`
- OpenAPI generation under `bijux-atlas-openapi`
- domain rules for datasets, queries, ingest, policy, cluster, and security
- contract-owned API errors, config schemas, and external payload shapes

## Binaries

- `bijux-atlas`: main Atlas CLI for dataset, catalog, ingest, policy, and OpenAPI workflows
- `bijux-atlas-server`: Atlas HTTP server
- `bijux-atlas-openapi`: OpenAPI generation utility

## Source Layout

The source tree is organized so ownership is obvious:

- `src/adapters`: inbound and outbound integrations such as CLI, HTTP, store, sqlite, redis, telemetry, and filesystem code
- `src/app`: use-case orchestration, ports, cache services, and server application state
- `src/contracts`: external schemas, runtime config contracts, and stable error definitions
- `src/domain`: business rules for dataset, query, ingest, policy, cluster, and security behavior
- `src/runtime`: runtime configuration and process-level setup

If a change affects transport or persistence details, it should usually land in `adapters`. If it changes business behavior, it should usually land in `domain`. If it changes an external schema or stable error surface, it belongs in `contracts`.

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

Inspect the server or OpenAPI utilities:

```bash
cargo run -p bijux-atlas --bin bijux-atlas-server -- --help
cargo run -p bijux-atlas --bin bijux-atlas-openapi -- --help
```

## Stable User-Facing Guarantees

- top-level command names and noun-first command families are treated as contract surfaces
- `--json` output is deterministic and designed for CI snapshots and automation
- error payloads and exit-code classes are documented and stable by contract
- runtime configuration is owned by contract documents and validation logic, not ad hoc flags or scripts
- API behavior is anchored in generated OpenAPI and contract tests

## Environment

- `BIJUX_LOG_LEVEL`: overrides log verbosity
- `BIJUX_CACHE_DIR`: sets the shared cache directory used by Atlas workflows

## Documentation Map

- crate documentation index: [../../docs/bijux-atlas-crate/index.md](../../docs/bijux-atlas-crate/index.md)
- CLI command reference: [../../docs/bijux-atlas-crate/cli-command-list.md](../../docs/bijux-atlas-crate/cli-command-list.md)
- CLI UX and output contract: [../../docs/bijux-atlas-crate/cli-ux-contract.md](../../docs/bijux-atlas-crate/cli-ux-contract.md)
- exit codes: [../../docs/bijux-atlas-crate/exit-codes.md](../../docs/bijux-atlas-crate/exit-codes.md)
- plugin contract: [../../docs/bijux-atlas-crate/plugin-contract.md](../../docs/bijux-atlas-crate/plugin-contract.md)
- API docs index: [../../docs/api/index.md](../../docs/api/index.md)
- crate API stability reference: [../../docs/reference/crate-api-stability.md](../../docs/reference/crate-api-stability.md)

## Using The Library

The primary supported interface is the CLI and the documented server/API contracts. The Rust library is available for workspace use and integration tests, but contributors should prefer the canonical module roots:

- `bijux_atlas::adapters`
- `bijux_atlas::app`
- `bijux_atlas::contracts`
- `bijux_atlas::domain`
- `bijux_atlas::runtime`

Avoid depending on undocumented internal helpers or convenience paths outside those owners.

## Development Notes

- the crate forbids unsafe Rust
- deterministic behavior is an explicit design goal
- contract tests, compatibility tests, and golden outputs are part of the maintenance model
- cache and artifact output are expected to live under the repository artifacts root, not crate-local scratch directories
