# Bijux Atlas

Bijux Atlas is a contract-governed data platform for validating source datasets, producing immutable release artifacts, and serving stable query surfaces over HTTP and CLI workflows.

This repository is both the product codebase and the repository control plane that governs it. The runtime, docs, configs, ops inputs, and review evidence live together on purpose, but they do not all have the same stability level.

## Start Here

- Overview: [`docs/index.md`](docs/index.md)
- New user path: [`docs/02-getting-started/index.md`](docs/02-getting-started/index.md)
- Operator path: [`docs/04-operations/index.md`](docs/04-operations/index.md)
- Contributor path: [`docs/06-development/index.md`](docs/06-development/index.md)
- Runtime reference: [`docs/07-reference/index.md`](docs/07-reference/index.md)
- Compatibility and contracts: [`docs/08-contracts/index.md`](docs/08-contracts/index.md)

## What Is In This Repo

| Path | Purpose |
| --- | --- |
| `crates/bijux-atlas/` | Runtime crate and user-facing binaries |
| `crates/bijux-dev-atlas/` | Maintainer control-plane binary for checks, docs, governance, configs, ops, and reports |
| `configs/` | Source-of-truth policy, schema, registry, and repository inputs |
| `ops/` | Deployment, observability, release, and operations data |
| `makes/` | Thin GNU Make wrapper surface over governed commands |
| `docs/` | Canonical reader-facing documentation in the numbered docs spine |
| `artifacts/` | Generated outputs and local evidence; not a source-of-truth tree |

## Supported Command Surfaces

Runtime surfaces:

- `bijux-atlas`
- `bijux-atlas-server`
- `bijux-atlas-openapi`

Maintainer surfaces:

- `bijux-dev-atlas`
- `make`, only as a thin wrapper layer rooted at [`makes/root.mk`](makes/root.mk)

Useful discovery commands:

```bash
cargo run -q -p bijux-atlas --bin bijux-atlas -- --help
cargo run -q -p bijux-atlas --bin bijux-atlas-server -- --help
cargo run -q -p bijux-dev-atlas -- --help
make help
```

The canonical maintainer command reference is [`docs/07-reference/automation-command-surface.md`](docs/07-reference/automation-command-surface.md). The runtime command reference is [`docs/07-reference/command-surface.md`](docs/07-reference/command-surface.md).

## What Is Stable

Treat these as the public or strongly-governed surfaces:

- runtime behavior described in [`docs/07-reference/index.md`](docs/07-reference/index.md)
- compatibility promises described in [`docs/08-contracts/index.md`](docs/08-contracts/index.md)
- checked-in configs and ops inputs that are validated by the control plane
- curated `make` targets listed in [`makes/root.mk`](makes/root.mk)

Do not treat the rest of the repository as an accidental public API. Internal implementation details, generated artifacts, and undocumented file layouts can change unless a contract or reference page says otherwise.

## Current Shape, Honestly

Atlas is maintainers-first engineering software. It has a clear runtime and a heavily governed repository, but it is not pretending every checked-in file is a polished public product surface.

The important boundaries are:

- `bijux-atlas` is the runtime-facing product surface
- `bijux-dev-atlas` is the canonical automation and governance surface
- `make` exists for convenience and CI ergonomics, not as the primary place for orchestration logic
- `docs/07-reference` and `docs/08-contracts` matter more than historical README text, ad hoc scripts, or debug output

## Fast Evaluation Path

Run these from the workspace root if you want a quick signal that the repository is healthy:

```bash
cargo run -q -p bijux-dev-atlas -- check doctor --format json
cargo run -q -p bijux-dev-atlas -- governance validate --format json
cargo check --workspace
make ci-fast
```

If those commands disagree with a claim in root docs, trust the command output and the numbered docs spine first.

## Repository Reading Order

If you are new here, this sequence usually gives the fastest honest understanding:

1. [`docs/01-introduction/what-atlas-is.md`](docs/01-introduction/what-atlas-is.md)
2. [`docs/02-getting-started/run-atlas-locally.md`](docs/02-getting-started/run-atlas-locally.md)
3. [`docs/05-architecture/system-overview.md`](docs/05-architecture/system-overview.md)
4. [`docs/06-development/automation-control-plane.md`](docs/06-development/automation-control-plane.md)
5. [`docs/05-architecture/source-layout-and-ownership.md`](docs/05-architecture/source-layout-and-ownership.md)

## Release Line

The current workspace version is `0.1.1`. The active release line is `0.1.x`.

Release and deprecation expectations are documented in [`docs/06-development/release-and-versioning.md`](docs/06-development/release-and-versioning.md).

## Security And Contribution

- Contribution guide: [`CONTRIBUTING.md`](CONTRIBUTING.md)
- Security policy: [`SECURITY.md`](SECURITY.md)
- Code ownership: [`.github/CODEOWNERS`](.github/CODEOWNERS)
- Makes surface overview: [`makes/README.md`](makes/README.md)
