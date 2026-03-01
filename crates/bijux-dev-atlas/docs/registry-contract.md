# Dev Control Plane Registry Contract

- Owner: bijux-dev-atlas
- Stability: evolving

## Purpose

Defines how checks are registered, selected, and validated in the dev control-plane.

## Registry Sources

- authored registry specification in `ops/inventory/registry.toml`
- check implementations in `src/core/checks/**`

## Registry Invariants

- every registered check ID must map to an implementation
- every implemented public check must have a registry entry
- registry check entries are sorted by `id`
- suites and selectors expand deterministically
- internal and slow checks remain explicitly classified

## Naming Contract

- check identifiers use `checks_*` intent naming
- identifiers are stable once referenced by suites, docs, or goldens

## Selection Contract

- selection depends on explicit selectors only
- hidden checks require explicit include flags
- machine outputs must describe applied selector state
