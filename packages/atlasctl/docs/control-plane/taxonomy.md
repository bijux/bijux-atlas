# Module Taxonomy

Target topology for `packages/atlasctl/src/atlasctl` is:

1. `cli/`
2. `commands/`
3. `checks/`
4. `suite/`
5. `contracts/`
6. `core/`
7. `registry/`
8. `reporting/`
9. `docs/`
10. `test_tools/`

`__init__.py` and `__main__.py` remain at package root.

## Grouping Rules

- `cli/`: parsing/dispatch/help only.
- `commands/`: all command handlers (domain folders under this package).
- `checks/`: check definitions and runners only.
- `suite/`: suite manifests, execution, and suite coverage checks.
- `contracts/`: schema IDs, catalog, validation, samples.
- `core/`: effect boundaries and runtime primitives.
- `registry/`: declarative inventories only; no runtime orchestration.
- `reporting/`: canonical report/artifact formatting and writing.
- `docs/`: docs generation and validation code.
- `test_tools/`: deterministic test orchestration tools.

## Migration Status

- `install` and `release` have been moved under `commands/dev/`.
- Remaining top-level packages outside this target are transitional and must be migrated into one of the 10 canonical modules.
