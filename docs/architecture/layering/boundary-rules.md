# Layer Boundary Rules

- `stack`: The stack layer provisions and tears down substrate services only and must not contain product-level test assertions.
- `k8s`: The k8s layer owns install/upgrade/render/health deployment mechanics and must not embed e2e scenario orchestration.
- `e2e`: The e2e layer consumes canonical `bijux dev atlas ops ...` entrypoints (or thin make wrappers) and must never patch, mutate, or repair infrastructure directly.
- `observe`: The observability layer validates telemetry contracts and drill evidence but must not perform deployment fixups.
- `load`: The load layer drives workload execution and baseline scoring only and must not alter cluster configuration.
