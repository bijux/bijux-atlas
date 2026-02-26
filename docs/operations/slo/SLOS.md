# SLO Targets (v1)

- Owner: `bijux-atlas-operations`
- Tier: `tier2`
- Audience: `operators`
- Source-of-truth: `ops/CONTRACT.md`, `ops/inventory/**`, `ops/schema/**`

- Generated from `configs/ops/slo/slo.v1.json`.

| SLO ID | SLI | Target | Window | Threshold |
|---|---|---:|---|---|
| `readyz-availability-30d` | `/readyz availability` | `0.999` | `30d` | - |
| `cheap-success-30d` | `Cheap success rate` | `0.9999` | `30d` | - |
| `standard-success-30d` | `Standard success rate` | `0.999` | `30d` | - |
| `heavy-success-30d` | `Heavy success rate` | `0.99` | `30d` | - |
| `cheap-latency-p99-1h` | `Cheap endpoint class latency p99` | `0.99` | `1h` | `lt 50 ms` |
| `standard-latency-p99-1h` | `Standard endpoint class latency p99` | `0.99` | `1h` | `lt 300 ms` |
| `heavy-latency-p99-1h` | `Heavy endpoint class latency p99` | `0.99` | `1h` | `lt 2000 ms` |
| `cheap-overload-survival-15m` | `Cheap survival under overload shedding` | `0.9999` | `15m` | - |
| `heavy-shed-rate-30m` | `Heavy shed rate` | `0.95` | `30m` | - |
| `standard-shed-rate-30m` | `Standard shed rate` | `0.98` | `30m` | `lt 2 percent` |
| `registry-refresh-freshness-10m` | `Registry refresh freshness` | `0.99` | `10m` | `lt 600 s` |
| `store-p95-latency-1h` | `Store p95 latency` | `0.99` | `1h` | `lt 500 ms` |
| `store-error-rate-24h` | `Store error rate` | `0.995` | `24h` | `lt 0.5 percent` |

## v1 Pragmatic Targets

- `/readyz` availability: `99.9%` over `30d`.
- Success: cheap `99.99%`, standard `99.9%`, heavy `99.0%` over `30d`.
- Latency p99 thresholds: cheap `< 50ms`, standard `< 300ms`, heavy `< 2s`.
- Overload cheap survival: `> 99.99%`.
- Shed policy: heavy shedding tolerated; standard shedding bounded.
- Registry freshness: refresh age `< 10m` for `99%` of windows.
- Store objective: p95 latency bounded and error rate `< 0.5%`.
