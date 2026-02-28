# How to Add Check

- Owner: `platform`
- Type: `guide`
- Audience: `contributor`
- Stability: `stable`
- Last verified against: `main@d489531b`
- Reason to exist: provide a practical walkthrough for adding a new control-plane check.

## Steps

1. Define the check intent and owning policy scope.
2. Implement deterministic ordering and machine-readable output.
3. Add focused contract tests and expected evidence outputs.
4. Wire command surface in control-plane registry.
5. Document user impact in canonical docs pages.

## Verify Success

- Focused check runs locally with deterministic output.
- Contract and policy tests pass.
- `make test` remains green.

## What to Read Next

- [Control Plane](control-plane.md)
- [Reports and CI Consumption](reports-and-ci-consumption.md)
