# Capacity planning worksheet

- Owner: `bijux-atlas-operations`
- Type: `guide`
- Audience: `operator`
- Stability: `stable`
- Last verified against: `main@2026-03-01`
- Reason to exist: provide one repeatable worksheet for translating observed load into CPU, memory, and storage planning.

## Example worksheet

| Dimension | Current | Target | Headroom rule |
| --- | --- | --- | --- |
| request rate | 100 rps | 250 rps | keep 30% spare under target |
| p95 latency | 180 ms | 220 ms max | stay below the published threshold |
| memory working set | 2.5 GiB | 4 GiB budget | keep 20% free under sustained load |
| storage growth | 200 GiB | 350 GiB forecast | keep one rollback release plus growth margin |

## Verify success

```bash
make ops-load-smoke
make ops-readiness-scorecard
```

## Rollback

If a sizing change worsens readiness or latency, revert the change and return to the last stable capacity envelope.

## Next

- [Capacity planning](capacity-planning.md)
- [Load testing](../load/index.md)
- [Release operations](../release-operations.md)
