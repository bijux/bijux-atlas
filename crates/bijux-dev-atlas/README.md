# bijux-dev-atlas

`bijux-dev-atlas` is the repository control-plane crate for the Bijux workspace. It turns governance, documentation, policy, validation, reporting, and operational workflows into a Rust command surface instead of a shell-script control plane.

Use this crate when you need to:

- run workspace checks in CI
- generate or validate governed reports and documentation
- inspect or enforce repository policy
- extend the workspace control plane in Rust instead of adding shell-script glue

This crate is repo-local infrastructure. It is intentionally `publish = false`, and its primary supported interface is the `bijux-dev-atlas` CLI rather than an external Rust SDK.

## What This Crate Owns

- repository governance and invariant checks
- documentation and reference generation
- policy loading, validation, and report emission
- operational inventory and install-status validation
- registry, report, release, load, security, and tutorial control-plane workflows

This crate does not own the product-facing Atlas runtime. Dataset, server, API, and end-user CLI behavior live in [`bijux-atlas`](../bijux-atlas/README.md).

## Supported Entry Points

- maintainers and CI should start with the `bijux-dev-atlas` CLI
- report consumers should start from the documented report and registry contracts
- contributors may use the Rust modules internally, but the stable operational surface is the CLI plus the documented contracts and registries

Internal module paths are implementation detail unless they are explicitly documented as a contract surface.

## Command Surface

The CLI is broad because it is the workspace control plane. The top-level families include:

- repository and policy workflows: `check`, `checks`, `audit`, `governance`, `policies`, `invariants`, `security`, `ci`
- docs and reference workflows: `docs`, `configs`, `registry`, `reports`
- runtime and ops workflows: `ops`, `system`, `runtime`, `observe`, `load`, `perf`
- support workflows: `tutorials`, `migrations`, `datasets`, `ingest`, `suites`, `tests`
- discovery and execution helpers: `list`, `describe`, `run`, `validate`

For the exact command registry, use the generated command reference linked below.

## Common Maintainer Workflows

- inspect the available surface: `bijux-dev-atlas --help`
- list registered domains, suites, and runnable ids: `bijux-dev-atlas list`
- inspect check-oriented surfaces: `bijux-dev-atlas check --help`
- inspect docs validation and generation flows: `bijux-dev-atlas docs --help`

## Control-Plane Rules

- repository automation should flow through crate commands, not shell scripts as the source of truth
- outputs should be deterministic and suitable for contract checks and CI snapshots
- network, subprocess, filesystem mutation, and git-sensitive behavior should be explicit, auditable choices
- contracts, registries, and policy documents should have one obvious owner path

## Execution Model

- machine-readable output is available through `--json` and related format flags
- repository-scoped commands should respect `--repo-root` instead of assuming the current directory
- many commands prefer hermetic behavior by default and require explicit allow-flags before performing external actions
- checks and reports are intended to be automatable, reproducible, and readable in CI logs

## Source Layout

This crate contains several large internal areas, but contributors should think about it in terms of ownership:

- `src/core`: foundational validation, checks, governance objects, and inventory logic
- `src/domains`: domain-specific control-plane workflows such as docs, ops, release, security, tutorials, and configs
- `src/engine`: shared execution and reporting machinery
- `src/registry`: command, config, and report registries
- `src/reference`: canonical workspace paths and structural references used by checks
- `src/docs`, `src/policies`, `src/ui`: support surfaces for documentation, policy modeling, and terminal presentation

The internal tree is broader than the supported public story. The important rule is that maintainers should treat the CLI, registries, reference documents, and explicitly documented contracts as the source of truth, not arbitrary module barrels.

## Quick Start

Show the control-plane surface:

```bash
cargo run -p bijux-dev-atlas -- --help
```

List registered commands:

```bash
cargo run -p bijux-dev-atlas -- list
```

Inspect the check and docs command families:

```bash
cargo run -p bijux-dev-atlas -- check --help
cargo run -p bijux-dev-atlas -- docs --help
```

## Stability and Contract Policy

- machine-readable output is designed to be deterministic
- command behavior is driven by Rust code, contracts, registries, and policy documents
- report shapes and validation rules are expected to remain explicit and test-covered
- repository checks should point at canonical workspace owners rather than historical compatibility paths

The following are not stable promises:

- arbitrary internal module paths
- convenience reexports that are not part of documented contract surfaces
- implementation details of terminal rendering or internal plumbing modules

## Documentation Map

Repository docs in this worktree:

- crate docs index: [../../docs/--archive/bijux-dev-atlas-docs/index.md](../../docs/--archive/bijux-dev-atlas-docs/index.md)
- command surface: [../../docs/--archive/bijux-dev-atlas-docs/cli-command-list.md](../../docs/--archive/bijux-dev-atlas-docs/cli-command-list.md)
- commands and usage: [../../docs/--archive/bijux-dev-atlas-docs/commands.md](../../docs/--archive/bijux-dev-atlas-docs/commands.md)

Governance and contracts:

- control-plane contract: [../../docs/--archive/bijux-dev-atlas-docs/contract.md](../../docs/--archive/bijux-dev-atlas-docs/contract.md)
- control-plane contracts: [../../docs/--archive/bijux-dev-atlas-docs/control-plane-contracts.md](../../docs/--archive/bijux-dev-atlas-docs/control-plane-contracts.md)
- registry contract: [../../docs/--archive/bijux-dev-atlas-docs/registry-contract.md](../../docs/--archive/bijux-dev-atlas-docs/registry-contract.md)
- errors and exit codes: [../../docs/--archive/bijux-dev-atlas-docs/errors-and-exit-codes.md](../../docs/--archive/bijux-dev-atlas-docs/errors-and-exit-codes.md)

Contributor references:

- architecture: [../../docs/--archive/bijux-dev-atlas-docs/architecture.md](../../docs/--archive/bijux-dev-atlas-docs/architecture.md)
- testing: [../../docs/--archive/bijux-dev-atlas-docs/testing.md](../../docs/--archive/bijux-dev-atlas-docs/testing.md)
- benchmark docs: [../../docs/--archive/bijux-dev-atlas-docs/benchmarks/index.md](../../docs/--archive/bijux-dev-atlas-docs/benchmarks/index.md)

## Working on This Crate

- prefer adding or extending commands in Rust instead of adding new control-plane shell scripts
- keep new output formats contract-owned and documented
- treat registries and workspace path references as single sources of truth
- prefer the CLI, report contracts, and generated references over ad hoc local conventions
- preserve deterministic output and explicit permission boundaries when adding new commands

## Relationship to `bijux-atlas`

`bijux-atlas` is the product-facing Atlas crate. `bijux-dev-atlas` is the workspace-facing control-plane crate that validates, documents, audits, and governs the repository around it.
