# Contributing

## Style

- Keep module boundaries aligned with `core/`, `cli/`, `commands/`, `checks/`, `contracts/`.
- New commands implement `configure(parser)` and `run(ctx, ns)`.
- New checks register via `atlasctl/checks` registry.

## Tests

- Unit tests are default (`pytest -m unit`).
- Integration tests run separately (`pytest -m integration`).
- Use isolated temp paths for writes in tests.

## Release and Versioning

- Follow package versioning and compatibility docs.
- Update docs and schema contracts together with behavior changes.

## Lock Update Policy

- Lock-resolved dependencies are required for deterministic CI.
- Update lock artifacts and validation checks in the same change when deps change.
