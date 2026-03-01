# Configs reference

- Owner: `bijux-atlas-operations`
- Type: `reference`
- Audience: `user`
- Stability: `stable`
- Last updated for release: `v1`
- Reason to exist: define canonical runtime configuration keys, defaults, and setting surfaces.

## Canonical sources

- Runtime config inventory: `configs/inventory/configs.json`
- Runtime config guide: `configs/README.md`
- Config governance contract: `configs/CONTRACT.md`

## What this page defines

- Runtime key names and semantics live in [Config keys contract](contracts/config-keys.md).
- Helm chart names and defaults live in [Chart values contract](contracts/chart-values.md).
- Procedures for setting values live outside reference docs.

## Usage boundaries

- API consumers should treat runtime keys as operator-owned implementation facts.
- Operators should use [Deploy](../operations/deploy.md) and [Values mapping to config keys](../operations/values-mapping-to-config-keys.md) for procedures.
- Contributors should use control-plane guides when generating or validating config inventories.

## Next steps

- [Schemas reference](schemas.md)
- [Operations config](../operations/config.md)
