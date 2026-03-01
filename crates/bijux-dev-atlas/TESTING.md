# Testing Policy

The crate uses unit, integration, and snapshot contracts.
Tests must be deterministic and isolated from external state.

## Rules

- Use fixed fixtures and avoid random data.
- Validate both human and JSON surfaces where applicable.
- Keep golden files updated intentionally with semantic changes.
- Guardrails enforce architecture, effects, and documentation boundaries.
- New contracts include focused tests and evidence assertions.

## Execution

Primary local gate is `make test` from the repository root.
