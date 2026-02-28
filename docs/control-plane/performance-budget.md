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

## Cost classes

- cheap: suitable for repeated local runs
- medium: acceptable in PR lanes
- expensive: reserved for nightly, merge, or release evidence

## Budget rules

- New checks must declare expected cost class.
- High-cost checks require lane justification.
- No hidden expensive work behind low-cost lane commands.

## Verification surfaces

- `make ci-fast` should stay in the fast-feedback budget.
- `make ci-pr` can be broader but should still fit pull-request cadence.
- `make ci-nightly` is the place for slow or internal-heavy suites.

## Verify success

Lane runtime remains predictable and regressions are visible in CI evidence.

## Next steps

- [How suites work](how-suites-work.md)
- [Tooling dependencies](tooling-dependencies.md)
- [Known limitations](known-limitations.md)
