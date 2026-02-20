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

- Tooling command surfaces are exposed via `make` and `bijux-atlas-scripts`.
- Documentation pages must stay linked through index navigation.

## Failure modes

- Missing nav references can create orphan tooling pages.
- Drift between command surfaces and docs can make instructions stale.

## How to verify

- `make scripts-check`
- `make docs-check`

## See also

- [Development](../INDEX.md)

## Pages

- [bijux-atlas-scripts](bijux-atlas-scripts.md)
