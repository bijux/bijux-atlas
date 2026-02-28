# Performance budget

- Owner: `platform`
- Type: `reference`
- Audience: `contributor`
- Stability: `stable`
- Last verified against: `main@331751e4`
- Reason to exist: define practical runtime expectations for control-plane lanes.

## Budget model

- Local lane: optimized for iterative contributor feedback.
- PR lane: stricter, still bounded for developer throughput.
- Merge and release lanes: broader coverage with deterministic artifacts.

## Budget rules

- New checks must declare expected cost class.
- High-cost checks require lane justification.
- No hidden expensive work behind low-cost lane commands.

## Verify success

Lane runtime remains predictable and regressions are visible in CI evidence.

## Next steps

- [How suites work](how-suites-work.md)
- [Tooling dependencies](tooling-dependencies.md)
