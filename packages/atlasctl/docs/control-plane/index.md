# Atlasctl Control Plane

`atlasctl` is the SSOT control plane for repository automation.

## Constitution

- `atlasctl` owns orchestration, checks, contracts, and reporting behavior.
- Makefiles are dispatch-only wrappers that delegate to `atlasctl` entrypoints.
- The target top-level command groups are:
  - `docs`
  - `configs`
  - `dev`
  - `ops`
  - `policies`
  - `internal`
- No new top-level package may be added under `packages/atlasctl/src/atlasctl/` unless it maps to one of these groups.
- Checks are defined as data + pure functions; effectful execution is handled by runtime/core helpers.
- Deep package nesting is capped by policy (see execution model and repo checks).
- Duplicate concepts are forbidden: use one canonical home for registry/runner/contracts/output.

## Migration Rule

When moving modules, keep the old import path working only within the same PR, then delete compatibility aliases before merge.

## Documents

- [Control Plane Surface](surface.md)
- [Execution Model](execution-model.md)
- [Module Taxonomy](taxonomy.md)
- [Shell Policy](shell-policy.md)
