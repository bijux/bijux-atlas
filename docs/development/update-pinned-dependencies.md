# Update Pinned Dependencies

- Owner: `platform`
- Type: `guide`
- Audience: `contributor`
- Stability: `stable`
- Last verified against: `main@240605bb1dd034f0f58f07a313d49d280f81556c`
- Reason to exist: define safe update workflow for pinned dependencies.

## Steps

1. Identify the pinned dependency and impact scope.
2. Update pin in canonical source and lockfiles.
3. Run affected contract and integration checks.
4. Update any docs or compatibility notes required by the change.

## Verify Success

```bash
make check
make test
```

All required lanes should remain green with deterministic artifacts.

## What to Read Next

- [Contributing](contributing.md)
- [CI Overview](ci-overview.md)
