# Ops API Contract

`ops/` is the spec.

`atlasctl ops ...` is the API that executes against that spec.

## Rule

- `ops/` stores declarative inventories, schemas, manifests, thresholds, examples, and generated docs.
- `atlasctl ops` commands and workflows provide the behavior (validation, rendering, deploy, test, evidence collection).
- Changes to runtime behavior should prefer `packages/atlasctl/src/atlasctl/ops/**` and `commands/ops/**`, not ad-hoc scripts under `ops/`.

## Examples

- Update inventory/toolchain pins in `ops/inventory/toolchain.yaml`, then run `atlasctl ops env doctor`.
- Update load suite thresholds in `ops/load/thresholds/*.thresholds.json`, then run `atlasctl ops test load`.
- Update namespace/port contracts in `ops/inventory/layers.json`, then run repo checks for ops namespace/ports invariants.
