# No Serving Writes Invariant

- Owner: `atlas-server`
- Stability: `stable`

## Invariant

Serving runtime must never write to dataset artifact SQLite files.

## Enforced By

- Read-only open flags (`mode=ro`, `immutable=1`).
- `PRAGMA query_only=ON` for opened artifact connections.
- Test coverage in server cache manager tests asserts write attempts fail.

## Failure Mode

Any successful write indicates artifact immutability violation and must fail CI.
