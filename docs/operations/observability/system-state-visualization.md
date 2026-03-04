# System State Visualization

Use this minimal state model for dashboards and incident notes:

- Health: `/healthz`, `/ready`, `/live`
- Runtime pressure: `/debug/runtime-stats`
- Planner pressure: `/debug/query-planner-stats`
- Cache pressure: `/debug/cache-stats`
- Dataset coverage: `/debug/dataset-registry`
- Routing topology: `/debug/shard-map`

Represent each as a timestamped panel and keep panel order stable for postmortems.
