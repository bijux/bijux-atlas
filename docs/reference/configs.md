# Configs Reference

- Owner: `bijux-atlas-operations`
- Type: `reference`
- Audience: `user`
- Stability: `stable`
- Last updated for release: `v1`
- Reason to exist: define canonical runtime configuration keys, defaults, and setting surfaces.

## Configuration sources

- Runtime config inventory: `configs/inventory/configs.json`
- Runtime config guide: `configs/README.md`
- Config governance contract: `configs/CONTRACT.md`

## How to set config

- Local/dev workflows: use documented control-plane commands.
- Kubernetes deploys: set chart values mapped to runtime keys.

## Canonical mappings

- Runtime keys and semantics: [Config Keys Contract](contracts/config-keys.md)
- Helm values mapping: [Chart Values Contract](contracts/chart-values.md)
- Operator procedures: [Deploy](../operations/deploy.md)

## Next

- [Schemas Reference](schemas.md)
- [Operations Config](../operations/config.md)
