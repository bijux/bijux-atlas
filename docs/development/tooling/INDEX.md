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
- [control-plane migration rationale](control-plane-migration-rationale.md)
- [command inventory](command-inventory.md)
- [build outputs](build-outputs.md)
- [ops command inventory](ops-command-inventory.md)
- [atlasctl deletion PR checklist](atlasctl-deletion-pr-checklist.md)
- [atlasctl migration map](atlasctl-migration-map.md)
- [bijux-atlas-py roadmap](bijux-atlas-py.md)
- [Python version policy](python-version-policy.md)
- [Scripts air-gapped mode](scripts-air-gapped.md)
- [Control-plane migration completion checklist](scripts-migration-complete-checklist.md)
- [tooling naming policy](naming-policy.md)
- [release packaging plan](release-packaging-plan.md)
- [versioning alignment](versioning-alignment.md)
- [scripts compatibility matrix](compat-matrix.md)
- [scripts compatibility policy](scripts-compat-policy.md)
- [scripts changelog](scripts-changelog.md)
- [tooling directory intent map](tools.md)
