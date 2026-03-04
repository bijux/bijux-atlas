# Grafana Dashboard Examples

Recommended dashboard groups:

- API request health: rate, latency, error ratio by route.
- Capacity: queue depth, backlog, thread usage, memory pressure.
- Query pipeline: planner latency, execution latency, slow query count.
- Dataset runtime: shard hit/miss and eviction trend.

Each dashboard should include:

- burn-rate panel
- top failing routes panel
- saturation panel
