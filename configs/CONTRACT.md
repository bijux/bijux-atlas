# Configs Contract

## Scope

- Governed surface: `configs/`
- Primary enforcement surfaces: `bijux dev atlas configs validate`, `bijux dev atlas configs doctor`, and `bijux dev atlas configs list --allow-write`
- Registry authorities: `configs/registry/inventory/configs.json`, `configs/registry/owners.json`, `configs/registry/consumers.json`, `configs/registry/schemas.json`, and `configs/registry/contracts.json`

## What This File Is

This document is the human summary of the configs contract surface.

It does not define executable checks by itself.
The executable registry is `configs/registry/contracts.json`, and the command surface that enforces the catalog is the `configs` validator and doctor commands.

## What The Contract Requires

- Every governed file in `configs/` must be declared by the inventory registry.
- Every governed public or generated file must declare ownership, consumer coverage, and schema coverage where applicable.
- Generated indexes must stay committed, deterministic, and consistent with the current tree.
- Authored inputs, generated artifacts, examples, registries, and validation schemas must stay in clearly separated parts of the tree.

## Output Artifacts

- `configs/generated/configs-index.json`
- `configs/schemas/registry/generated/schema-index.json`

## Notes

- `configs/registry/contracts.json` is a contract catalog, not a separate CLI namespace.
- `enforced_by.test_id` values in that catalog are stable registry keys used to name validator expectations.
- Exception authority remains in the governance sources and registries, not in this document.
