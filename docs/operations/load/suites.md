# Load suite catalog

- Owner: `bijux-atlas-operations`
- Type: `reference`
- Audience: `operator`
- Stability: `stable`
- Last verified against: `main@c59da0bf`
- Reason to exist: define suite intent and when to use each suite.

## Gating suites

- `mixed`: baseline release gate.
- `cold-start-p99`: startup latency guard.
- `spike-overload-proof`: burst resilience guard.
- `store-outage-under-spike`: degraded store behavior guard.

## Extended suites

- `sharded-fanout`: fanout behavior at scale.
- `multi-release`: cross-release consistency.
- `diff-heavy`: diff endpoint pressure profile.
- `soak-30m`: sustained load and memory growth check.

Authoritative suite definitions live in `ops/load/suites/suites.json`.

## Verify success

```bash
make ops-load-nightly
```

Expected result: suite list resolves, each suite writes a result artifact, and gating suites pass.

## Rollback

If a suite change introduces false positives or hides a real regression, revert the suite definition change and rerun the affected suites.

## Next

- [k6 execution](k6.md)
- [Load Failure Triage](../runbooks/load-failure-triage.md)
