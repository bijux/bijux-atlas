# Commands reference

- Owner: `bijux-atlas-operations`
- Type: `reference`
- Audience: `user`
- Stability: `stable`
- Last updated for release: `v1`
- Reason to exist: define stable command surfaces and where each command family is used.

## Command families

- Product CLI: `bijux atlas ...`
- Control-plane CLI: `bijux dev atlas ...`
- Wrapper entrypoints: `make <target>`

## Canonical command documents

- Surface inventory: [Command inventory](command-inventory.md)
- Make wrappers: [Make reference](make.md)
- Operator procedures: [Operations](../operations/index.md)
- Contributor procedures: [Control-plane](../control-plane/index.md)

## Boundaries

This page names stable command families and their intended audiences. It does not define runbooks or onboarding flows.

## Command ownership

- `bijux atlas ...` serves API users and operators.
- `bijux dev atlas ...` serves contributors and CI lanes.
- `make <target>` is the supported wrapper layer for published workflows.

## Next steps

- [Command inventory](command-inventory.md)
- [Make reference](make.md)
- [Operations surface reference](ops-surface.md)
