# Configs contracts

Configs contracts are enforced by `bijux dev atlas contracts configs --mode static`.

Scope:
- config tree shape and canonical locations
- schema registry and strictness rules
- owner and consumer mappings
- generated config inventory and drift checks

Related suites:
- `configs_required`: PR-required static config checks
- `configs`: full configs-domain suite

Related sources:
- `crates/bijux-dev-atlas/src/contracts/configs/`
- `configs/OWNERS.json`
- `configs/CONSUMERS.json`
- `ops/inventory/registry.toml`
