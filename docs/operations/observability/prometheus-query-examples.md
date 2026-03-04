# Prometheus Query Examples

Request error ratio:

```promql
sum(rate(atlas_http_request_errors_total[5m])) / clamp_min(sum(rate(atlas_http_requests_total[5m])), 1)
```

P95 request latency:

```promql
histogram_quantile(0.95, sum(rate(atlas_http_request_duration_seconds_bucket[5m])) by (le))
```

Slow query rate:

```promql
sum(rate(atlas_slow_queries_total[5m]))
```
