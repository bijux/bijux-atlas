# How to Add Gate Policy

- Owner: `platform`
- Type: `policy`
- Audience: `contributor`
- Stability: `stable`
- Last verified against: `main@50be979f`
- Reason to exist: define requirements for introducing a new gate policy safely.

## Policy Requirements

- Gate intent and ownership must be explicit.
- Pass/fail semantics must be deterministic.
- Failure output must include reproducible command context.
- CI lane placement must be declared (`local`, `pr`, `merge`, `release`).

## Steps

1. Define policy intent and scope.
2. Add policy configuration and stable identifiers.
3. Wire gate execution in control-plane checks.
4. Add tests for pass/fail and output shape.
5. Update docs for contributor discoverability.

## Verify Success

- Gate appears in expected lanes.
- Contract checks and tests pass.
- No hidden bypass path exists.

## What to Read Next

- [Control Plane](control-plane.md)
- [Error Handling Philosophy](error-handling-philosophy.md)
