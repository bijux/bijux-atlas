# Command inventory

- Owner: `docs-governance`
- Type: `reference`
- Audience: `contributor`
- Stability: `stable`
- Last updated for release: `v1`
- Reason to exist: provide the canonical inventory entrypoint for command surfaces.

## Inventory model

- This page is the human-readable inventory of published command families.
- Generated command JSON belongs in docs artifacts and the docs dashboard, not in reader navigation.

## Published command families

- `bijux atlas ...`: product CLI surface for querying, validating, and inspecting Atlas.
- `bijux dev atlas ...`: control-plane surface for contributors, CI, and contract enforcement.
- `make <target>`: stable wrapper layer for documented local, ops, and docs workflows.

## Canonical references

- Command boundaries: [Commands reference](commands.md)
- Make targets: [Make reference](make.md)
- Operator entrypoints: [Operations surface reference](ops-surface.md)

## Next steps

- [Commands reference](commands.md)
- [Control-plane](../control-plane/index.md)
