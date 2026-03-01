# Make contracts

Make contracts are enforced by `bijux dev atlas contracts make --mode static`.

Scope:
- root `Makefile` minimalism
- wrapper-module boundaries and curated targets
- help and target registry stability
- CI parity and delegated execution through `bijux dev atlas`

Related suites:
- `make_required`: PR-required static make checks
- `make_fast`: fast make checks

Related sources:
- `crates/bijux-dev-atlas/src/contracts/make/`
- `make/CONTRACT.md`
- `ops/inventory/registry.toml`
