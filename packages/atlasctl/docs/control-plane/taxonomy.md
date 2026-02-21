# Module Taxonomy

Single taxonomy for `packages/atlasctl/src/atlasctl`.

## Command Layer

- `cli/`: parsing, registry wiring, dispatch.
- `commands/`: command handlers and orchestration by group (`docs`, `configs`, `dev`, `ops`, `policies`, `internal`).

## Check Layer

- `checks/`: policy/check definitions as data + pure functions.
- `checks/repo/enforcement/`: structural and architectural policy enforcement.
- Checks must not own effectful execution plumbing.

## Runtime/Core Layer

- `core/`: effect boundaries and runtime primitives (`fs`, `exec`, `env`, `process`, `network`, context).
- `contracts/`: canonical schema/catalog and validation helpers.
- `reporting/`: canonical output/report assembly.
- `suite/`: deterministic suite runner and artifacts model.

## Canonical Concept Homes

- Registry: `registry/`
- Runner: `suite/`
- Contracts: `contracts/`
- Output/report emission: `reporting/`

Do not create duplicate concept packages for the same role.
