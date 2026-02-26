# SLIs (v1)

- Owner: `bijux-atlas-operations`
- Tier: `tier2`
- Audience: `operators`
- Source-of-truth: `ops/CONTRACT.md`, `ops/inventory/**`, `ops/schema/**`

- Generated from `configs/ops/slo/slis.v1.json`.

| SLI | Goal | Primary Metric | Secondary Metric | Status |
|---|---|---|---|---|
| Availability SLI | fraction of time /readyz returns 200 | `bijux_http_requests_total` | - | `enforced` |
| Success rate SLI | 2xx / (2xx + 5xx) per endpoint class | `bijux_http_requests_total` | - | `enforced` |
| Latency SLI (p95/p99) | p95 and p99 request duration per endpoint class | `bijux_http_request_latency_p95_seconds` | `bijux_http_request_latency_p99_seconds` | `enforced` |
| Overload survival SLI | cheap endpoints keep succeeding under load shedding | `bijux_http_requests_total` | - | `enforced` |
| Shed rate SLI | fraction of requests shed due to overload | `atlas_shed_total` | - | `enforced` |
| Cold start SLI | time from pod start to first successful standard query | `bijux_request_stage_latency_p95_seconds` | - | `planned` |
| Cache hit ratio SLI | hit / (hit + miss) for dataset cache | `bijux_dataset_hits` | `bijux_dataset_misses` | `enforced` |
| Store dependency SLI | store backend p95 latency and error rate | `bijux_store_fetch_latency_p95_seconds` | `atlas_store_errors_total` | `enforced` |
| Registry freshness SLI | time since last successful registry refresh | `bijux_registry_refresh_age_seconds` | - | `planned` |
| Dataset availability SLI | requests failing because dataset missing when expected | `bijux_dataset_misses` | - | `enforced` |
| Correctness guard SLI | genes/count vs genes/list mismatch count | `bijux_genes_count_list_mismatch_total` | - | `planned` |

## Endpoint Class Mapping

- `cheap`: `^/health$`, `^/version$`, `^/metrics$`
- `standard`: `^/v1/genes$`, `^/v1/genes/[^/]+$`
- `heavy`: `^/v1/genes/[^/]+/(diff|region|sequence)$`
