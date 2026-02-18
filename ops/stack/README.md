# Ops Stack

Canonical stack manifests and bootstrap scripts for local and CI operations.

- `kind/`: cluster definitions and profile scripts (`small|normal|perf`)
- `minio/`: object store deployment and bootstrap
- `prometheus/`: Prometheus deployment
- `otel/`: OpenTelemetry collector deployment
- `redis/`: optional Redis deployment
- `toxiproxy/`: optional store fault/latency proxy
- `faults/`: fault-injection scripts
- `values/`: canonical values profiles used by ops targets

Use `make ops-up` / `make ops-down` and related `ops-*` targets as the interface.

Kind helpers:
- `make ops-kind-up`
- `make ops-kind-down`
- `make ops-kind-reset`
- `make ops-kind-registry-up`

See top-level ops guide: `ops/README.md`.
