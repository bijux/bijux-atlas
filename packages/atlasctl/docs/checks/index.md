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
