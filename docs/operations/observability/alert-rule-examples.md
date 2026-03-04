# Alert Rule Examples

High error ratio:

```yaml
- alert: AtlasHighErrorRatio
  expr: sum(rate(atlas_http_request_errors_total[5m])) / clamp_min(sum(rate(atlas_http_requests_total[5m])), 1) > 0.05
  for: 10m
```

Sustained high latency:

```yaml
- alert: AtlasHighP95Latency
  expr: histogram_quantile(0.95, sum(rate(atlas_http_request_duration_seconds_bucket[5m])) by (le)) > 1.0
  for: 15m
```
