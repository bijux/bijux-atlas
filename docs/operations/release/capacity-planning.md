# Capacity Planning

- Owner: `bijux-atlas-operations`
- Type: `guide`
- Audience: `operator`
- Stability: `stable`
- Last verified against: `main@8641e5b0`
- Reason to exist: define CPU, memory, and disk planning guidance for release scaling.

## Why you are reading this

Use this page to estimate resource needs before increasing dataset size or traffic volume.

## Planning dimensions

- CPU: query concurrency and read-path saturation.
- Memory: cache behavior and peak request footprint.
- Disk: dataset artifacts, indexes, and retention windows.

## Planning loop

1. Collect baseline from observability dashboards.
2. Run load suite for target profile.
3. Compare saturation and latency against SLO targets.
4. Apply minimal resource adjustments.

## Verify success

```bash
make ops-load-smoke
make ops-readiness-scorecard
```

Expected result: capacity headroom remains within target under expected load.

## Next

- [Load Testing](../load/index.md)
- [Capacity planning worksheet](capacity-planning-worksheet.md)
- [Release Operations](index.md)
