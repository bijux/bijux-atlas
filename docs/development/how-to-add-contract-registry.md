# How to Add Contract Registry

- Owner: `platform`
- Type: `guide`
- Audience: `contributor`
- Stability: `stable`
- Last verified against: `main@d489531b`
- Reason to exist: document how to add a new contract registry without drift.

## Steps

1. Define the contract scope and owning team.
2. Add canonical schema and stable identifiers.
3. Register checks and update deterministic outputs.
4. Add docs/reference links for human discoverability.
5. Add tests covering registry integrity and surface behavior.

## Verify Success

- Registry entries resolve to real files.
- Deterministic outputs are stable.
- Integration tests pass with no ignored new failures.

## What to Read Next

- [Control Plane](control-plane.md)
- [How to Add Check](how-to-add-check.md)
