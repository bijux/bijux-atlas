# Bijux Metrics Naming Conventions

## Metric Prefix

- All Prometheus metrics must use `bijux_` prefix.

## Label Contract

Every runtime metric must include:

- `subsystem` (example: `atlas`)
- `version` (service semantic version)
- `dataset` (`all` or explicit dataset id when cardinality-safe)

## Naming Pattern

- Counters: `bijux_<domain>_<event>_total`
- Gauges: `bijux_<domain>_<value>`
- Latency summaries/p95 gauges: `bijux_<domain>_<operation>_p95_seconds`
