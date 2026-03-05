# Load Testing Philosophy

- Owner: `bijux-atlas-operations`
- Type: `policy`
- Audience: `operator`
- Stability: `stable`
- Reason to exist: establish why and how load tests are used to protect production behavior.

## Principles

1. Load tests must reflect production request patterns.
2. Deterministic seeds are required for repeatable comparisons.
3. Failures are evidence, not noise; each failure needs classification.
4. Capacity decisions rely on trend data, not single-run spikes.
