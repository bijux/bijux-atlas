# bijux-atlas-dev

[![Rust 1.86+](https://img.shields.io/badge/rust-1.86%2B-DEA584?logo=rust&logoColor=white)](https://github.com/bijux/bijux-atlas)
[![License: Apache-2.0](https://img.shields.io/badge/license-Apache--2.0-0F766E)](https://github.com/bijux/bijux-atlas/blob/main/LICENSE)
[![CI](https://github.com/bijux/bijux-atlas/workflows/repo%20/%20ci/badge.svg)](https://github.com/bijux/bijux-atlas/actions/workflows/ci.yml?query=branch%3Amain)
[![docs-maintainer](https://img.shields.io/badge/docs-maintainer-2563EB?logo=materialformkdocs&logoColor=white)](https://bijux.io/bijux-atlas/bijux-atlas-dev/)
[![GitHub Repository](https://img.shields.io/badge/github-bijux%2Fbijux--atlas-181717?logo=github)](https://github.com/bijux/bijux-atlas)

`bijux-atlas-dev` is the repository control plane for Atlas. It turns
governance, release planning, documentation checks, security validation,
reporting, and maintainer workflows into owned Rust commands instead of
disconnected shell glue.

This crate is repository-only infrastructure. Its Cargo manifest sets
`publish = false`, so it is not part of the crates.io release family even
though it is central to how the repository is maintained.

## Choose This Crate When

- you are changing repository law, docs, release metadata, or workflow policy
- you need machine-readable evidence from Atlas maintainer commands
- you want automation to live in Rust with explicit ownership and test coverage

This crate is repository infrastructure. Its primary supported interfaces are the `bijux dev atlas ...` umbrella namespace and the direct `bijux-atlas-dev` CLI rather than an external Rust SDK.

## What This Crate Owns

- repository governance and invariant checks
- documentation and reference generation
- policy loading, validation, and report emission
- operational inventory and install-status validation
- registry, report, release, load, security, and tutorial control-plane workflows

This crate does not own the product-facing Atlas runtime. Dataset, server, API, and end-user CLI behavior live in [`bijux-atlas`](../bijux-atlas/README.md).

## Supported Entry Points

- maintainers and CI should start with `bijux dev atlas ...` or the direct `bijux-atlas-dev` CLI
- report consumers should start from the documented report and registry contracts
- contributors may use the Rust modules internally, but the stable operational surface is the CLI plus the documented contracts and registries

Internal module paths are implementation detail unless they are explicitly
documented as a contract surface.

## Command Surface

The CLI is broad because it is the workspace control plane. The top-level families include:

- repository and policy workflows: `check`, `checks`, `audit`, `governance`, `policies`, `invariants`, `security`, `ci`
- docs and reference workflows: `docs`, `configs`, `registry`, `reports`
- runtime and ops workflows: `ops`, `system`, `runtime`, `observe`, `load`, `perf`
- support workflows: `tutorials`, `migrations`, `datasets`, `ingest`, `suites`, `tests`
- discovery and execution helpers: `list`, `describe`, `run`, `validate`

For the exact command registry, use the generated command reference linked below.

## Common Maintainer Workflows

- inspect the available surface: `bijux dev atlas --help`
- list registered domains, suites, and runnable ids: `bijux dev atlas list`
- inspect check-oriented surfaces: `bijux dev atlas check --help`
- inspect docs validation and generation flows: `bijux dev atlas docs --help`

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
bijux dev atlas --help
cargo run -p bijux-atlas-dev -- --help
```

List registered commands:

```bash
bijux dev atlas list
cargo run -p bijux-atlas-dev -- list
```

Inspect the check and docs command families:

```bash
bijux dev atlas check --help
bijux dev atlas docs --help
cargo run -p bijux-atlas-dev -- check --help
cargo run -p bijux-atlas-dev -- docs --help
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

- docs index: [../../docs/index.md](../../docs/index.md)
- command surface: [../../docs/bijux-atlas-dev/automation/automation-command-surface.md](../../docs/bijux-atlas-dev/automation/automation-command-surface.md)
- commands and usage: [../../docs/bijux-atlas-dev/automation/automation-control-plane.md](../../docs/bijux-atlas-dev/automation/automation-control-plane.md)

Governance and contracts:

- automation contracts: [../../docs/bijux-atlas-dev/governance/automation-contracts.md](../../docs/bijux-atlas-dev/governance/automation-contracts.md)
- report reference: [../../docs/bijux-atlas-dev/automation/automation-reports-reference.md](../../docs/bijux-atlas-dev/automation/automation-reports-reference.md)
- decision records and ownership: [../../docs/bijux-atlas-dev/workspace/decision-records-and-ownership.md](../../docs/bijux-atlas-dev/workspace/decision-records-and-ownership.md)
- errors and exit codes: [../../docs/bijux-atlas/interfaces/error-codes-and-exit-codes.md](../../docs/bijux-atlas/interfaces/error-codes-and-exit-codes.md)

Contributor references:

- architecture: [../../docs/bijux-atlas-dev/governance/automation-architecture.md](../../docs/bijux-atlas-dev/governance/automation-architecture.md)
- testing and evidence: [../../docs/bijux-atlas-dev/governance/testing-and-evidence.md](../../docs/bijux-atlas-dev/governance/testing-and-evidence.md)
- contributor workflow: [../../docs/bijux-atlas-dev/workspace/contributor-workflow.md](../../docs/bijux-atlas-dev/workspace/contributor-workflow.md)

## Working on This Crate

- prefer adding or extending commands in Rust instead of adding new control-plane shell scripts
- keep new output formats contract-owned and documented
- treat registries and workspace path references as single sources of truth
- prefer the CLI, report contracts, and generated references over ad hoc local conventions
- preserve deterministic output and explicit permission boundaries when adding new commands

## Relationship to `bijux-atlas`

`bijux-atlas` is the product-facing Atlas crate. `bijux-atlas-dev` is the workspace-facing control-plane crate that validates, documents, audits, and governs the repository around it.

Runtime ownership boundaries for this crate are defined in
[`runtime-ownership-boundary.md`](../../docs/bijux-atlas-dev/workspace/runtime-ownership-boundary.md).
