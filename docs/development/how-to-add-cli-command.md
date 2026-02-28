# How to Add CLI Command

- Owner: `platform`
- Type: `guide`
- Audience: `contributor`
- Stability: `stable`
- Last verified against: `main@50be979f`
- Reason to exist: define the canonical process for adding a CLI command without breaking surface contracts.

## Steps

1. Define command intent and owning surface (`bijux atlas` vs `bijux dev atlas`).
2. Add command wiring in the correct crate and namespace.
3. Update command documentation in canonical reference pages.
4. Add contract tests for help output and command behavior.
5. Run required checks and confirm deterministic outputs.

## Verify Success

```bash
make check
make test
```

## What to Read Next

- [Control Plane](control-plane.md)
- [How to Add Check](how-to-add-check.md)
- [Reference Commands](../reference/commands.md)
