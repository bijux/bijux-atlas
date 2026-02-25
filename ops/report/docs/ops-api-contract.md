# Ops API Contract

`ops/` is the spec.

`bijux dev atlas ops ...` is the API that executes against that spec.

## Rule

- `ops/` stores declarative inventories, schemas, manifests, thresholds, examples, and generated docs.
- `bijux dev atlas ops` commands and workflows provide the behavior (validation, rendering, deploy, test, evidence collection).
- Changes to runtime behavior should prefer `crates/bijux-dev-atlas/src/bijux-dev-atlas/ops/**` and `commands/ops/**`, not ad-hoc scripts under `ops/`.

## Examples

- Update inventory/toolchain pins in `ops/inventory/toolchain.json`, then run `bijux dev atlas ops env validate`.
- Update load suite thresholds in `ops/load/thresholds/*.thresholds.json`, then run `bijux dev atlas ops load check`.
- Update namespace/port contracts in `ops/inventory/layers.json`, then run repo checks for ops namespace/ports invariants.
