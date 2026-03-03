# Performance Contracts

- Owner: `platform`
- Stability: `stable`
- Last verified against: `main@db6478db1e24e685969191669cb17ca4d2fe06fb`

## PERF-SLO

- `PERF-SLO-001`: the governed SLO and budget sources validate and required targets are present.

## PERF-LOAD

- `PERF-LOAD-001`: the load report is structurally valid.
- `PERF-LOAD-002`: p99 latency stays within the governed SLO.
- `PERF-LOAD-003`: error rate stays within the governed SLO.
- `PERF-LOAD-004`: throughput stays above the governed minimum.

## PERF-BENCH

- `PERF-BENCH-001`: the bench registry matches the on-disk bench files.
- `PERF-BENCH-002`: macro benches are not enabled by default.

## PERF-MEM / PERF-CPU / PERF-COLD / PERF-KIND

- `PERF-MEM-001`: measured RSS stays within the governed memory budget.
- `PERF-CPU-001`: best-effort CPU sampling stays within the governed CPU budget.
- `PERF-COLD-001`: cold start reaches ready within the governed threshold.
- `PERF-KIND-001`: the perf profile gate passes when kind is reachable and the governed load check
  succeeds.

## PERF-EXC

- `PERF-EXC-001`: no committed performance exception is past its expiry date.
