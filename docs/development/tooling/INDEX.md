# Tooling

## What

Tooling references for local development and repository automation.

## Why

Provide a stable landing page for tooling docs and avoid orphaned pages.

## Scope

Developer tooling contracts and command surfaces used in this repository.

## Non-goals

This section does not duplicate full command reference details from each tooling page.

## Contracts

- Tooling command surfaces are exposed via `make` and `bijux dev atlas`.
- Documentation pages must stay linked through index navigation.

## Failure modes

- Missing nav references can create orphan tooling pages.
- Drift between command surfaces and docs can make instructions stale.

## How to verify

- `make scripts-check`
- `make docs`

## See also

- [Development](../INDEX.md)

## Pages

- [control plane](control-plane.md)
- [control-plane transition rationale](control-plane-transition-rationale.md)
- [command inventory](command-inventory.md)
- [build outputs](build-outputs.md)
- [rust toolchain contract](rust-toolchain.md)
- [ops command inventory](ops-command-inventory.md)
- [tooling naming policy](naming-policy.md)
- [release packaging plan](release-packaging-plan.md)
- [versioning alignment](versioning-alignment.md)
- [tooling compatibility matrix](compat-matrix.md)
- [tooling compatibility policy](scripts-compat-policy.md)
- [tooling directory intent map](tools.md)
