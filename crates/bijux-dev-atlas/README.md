# bijux-dev-atlas

`bijux-dev-atlas` is the repository control-plane crate for the Bijux workspace. It turns governance, documentation, policy, validation, reporting, and operational checks into a Rust command surface instead of a shell-script control plane.

This crate is for:

- maintainers running repository checks, report generation, and release validation
- CI jobs that need deterministic, contract-driven control-plane behavior
- contributors extending docs, policy, registry, ops, security, tutorial, or audit workflows

This crate is repo-local infrastructure. It is intentionally `publish = false`, and its primary supported interface is the `bijux-dev-atlas` CLI rather than an external Rust SDK.

## What This Crate Owns

- repository governance and invariant checks
- documentation and reference generation
- policy loading, validation, and report emission
- operational inventory and install-status validation
- registry, report, release, load, security, and tutorial control-plane workflows

## Command Surface

The CLI is broad because it is the workspace control plane. The top-level families include:

- repository and policy workflows: `check`, `checks`, `audit`, `governance`, `policies`, `invariants`, `security`, `ci`
- docs and reference workflows: `docs`, `configs`, `registry`, `reports`
- runtime and ops workflows: `ops`, `system`, `runtime`, `observe`, `load`, `perf`
- support workflows: `tutorials`, `migrations`, `datasets`, `ingest`, `suites`, `tests`
- discovery and execution helpers: `list`, `describe`, `run`, `validate`

For the exact command registry, use the generated command reference linked below.

## Control-Plane Rules

- repository automation should flow through crate commands, not shell scripts as the source of truth
- outputs should be deterministic and suitable for contract checks and CI snapshots
- network, subprocess, filesystem mutation, and git-sensitive behavior should be explicit, auditable choices
- contracts, registries, and policy documents should have one obvious owner path

## Source Layout

This crate contains several large internal areas, but contributors should think about it in terms of ownership:

- `src/core`: foundational validation, checks, governance objects, and inventory logic
- `src/domains`: domain-specific control-plane workflows such as docs, ops, release, security, tutorials, and configs
- `src/engine`: shared execution and reporting machinery
- `src/registry`: command, config, and report registries
- `src/reference`: canonical workspace paths and structural references used by checks
- `src/docs`, `src/policies`, `src/ui`: support surfaces for documentation, policy modeling, and terminal presentation

Some legacy internal layout remains in the source tree because this crate is still converging, but the supported entrypoint for maintainers is the CLI and the documented contracts, not arbitrary module barrels.

## Quick Start

Show the control-plane surface:

```bash
cargo run -p bijux-dev-atlas -- --help
```

List registered commands:

```bash
cargo run -p bijux-dev-atlas -- list
```

Inspect the docs command family:

```bash
cargo run -p bijux-dev-atlas -- docs --help
```

## Stable Guarantees

- machine-readable output is designed to be deterministic
- command behavior is driven by Rust code, contracts, registries, and policy documents
- report shapes and validation rules are expected to remain explicit and test-covered
- repository checks should point at canonical workspace owners rather than historical compatibility paths

## Documentation Map

- crate docs index: [../../docs/bijux-dev-atlas-docs/index.md](../../docs/bijux-dev-atlas-docs/index.md)
- command surface: [../../docs/bijux-dev-atlas-docs/cli-command-list.md](../../docs/bijux-dev-atlas-docs/cli-command-list.md)
- commands and usage: [../../docs/bijux-dev-atlas-docs/commands.md](../../docs/bijux-dev-atlas-docs/commands.md)
- control-plane contract: [../../docs/bijux-dev-atlas-docs/contract.md](../../docs/bijux-dev-atlas-docs/contract.md)
- control-plane contracts: [../../docs/bijux-dev-atlas-docs/control-plane-contracts.md](../../docs/bijux-dev-atlas-docs/control-plane-contracts.md)
- registry contract: [../../docs/bijux-dev-atlas-docs/registry-contract.md](../../docs/bijux-dev-atlas-docs/registry-contract.md)
- errors and exit codes: [../../docs/bijux-dev-atlas-docs/errors-and-exit-codes.md](../../docs/bijux-dev-atlas-docs/errors-and-exit-codes.md)
- architecture: [../../docs/bijux-dev-atlas-docs/architecture.md](../../docs/bijux-dev-atlas-docs/architecture.md)
- testing: [../../docs/bijux-dev-atlas-docs/testing.md](../../docs/bijux-dev-atlas-docs/testing.md)
- benchmark docs: [../../docs/bijux-dev-atlas-docs/benchmarks/index.md](../../docs/bijux-dev-atlas-docs/benchmarks/index.md)

## How To Work With This Crate

- prefer adding or extending commands in Rust instead of adding new control-plane shell scripts
- keep new output formats contract-owned and documented
- treat registries and workspace path references as single sources of truth
- prefer the CLI, report contracts, and generated references over ad hoc local conventions

## Relationship To `bijux-atlas`

`bijux-atlas` is the product-facing Atlas crate. `bijux-dev-atlas` is the workspace-facing control-plane crate that validates, documents, audits, and governs the repository around it.
