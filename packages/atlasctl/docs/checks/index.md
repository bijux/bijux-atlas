# Check Domains

Atlasctl checks are registry-driven under `src/atlasctl/checks/`.

## Domains

- `repo`: repository structure, path boundaries, and policy checks.
- `make`: makefile and command invocation policy checks.
- `docs`: docs-specific policy checks.
- `ops`: operations policy checks.
- `checks`: checks-system integrity checks.
- `configs`: config domain checks.
- `python`: Python runtime/tooling checks.
- `docker`: docker domain checks.
- `contracts`: schema and contract checks.

Use `atlasctl check list --json` for machine-readable inventory.

## Layout Domains

- `checks/layout/root`: repository root-shape and determinism layout policies.
- `checks/layout/artifacts`: artifact layout and evidence hygiene checks.
- `checks/layout/makefiles`: makefile surface and contract checks.
- `checks/layout/ops`: ops layout and policy checks.
- `checks/layout/scripts`: script-area and naming/bucket checks.
- `checks/layout/docs`: docs/help surface integrity checks.
- `checks/layout/workflows`: workflow invocation and entrypoint checks.
- `checks/layout/contracts`: legacy contract subgroup during migration.
- `checks/layout/governance`: gate orchestration and inventory checks.
- `checks/layout/public_surface`: public target/help surface checks.
