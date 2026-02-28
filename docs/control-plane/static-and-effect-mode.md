# Static and effect mode

- Owner: `platform`
- Type: `concept`
- Audience: `contributor`
- Stability: `stable`
- Last verified against: `main@331751e4`
- Reason to exist: explain behavior differences between static-only and effect-enabled execution.

## Static mode

- No side-effecting operations.
- Validates structure, metadata, contracts, and deterministic projections.

## Effect mode

- Allows controlled subprocess, network, or filesystem effects where required.
- Uses explicit capability flags and lane constraints.

## Consequences

- Static mode is safer for fast local checks.
- Effect mode is required for deploy-like validations and some ops checks.

## Verify success

```bash
cargo run -q -p bijux-dev-atlas -- docs build --help
```

## Next steps

- [Capabilities model](capabilities-model.md)
- [Tooling dependencies](tooling-dependencies.md)
