# Local Stack (Make Only)

- Owner: `bijux-atlas-operations`
- Tier: `tier2`
- Audience: `operators`
- Source-of-truth: `ops/CONTRACT.md`, `ops/inventory/**`, `ops/schema/**`

Run the local reference stack using make targets only.

```bash
make ops-up
make ops-deploy
make ops-publish
make ops-warm
make ops-smoke
make ops-k8s-tests
make ops-load-smoke
make ops-observability-validate
```

One-command flow:

```bash
make ops-full
```

Required ref-grade local gate:

```bash
make ops-ref-grade-local
```

Canonical targets: `ops-up`, `ops-deploy`, `ops-warm`, `ops-smoke`, `ops-k8s-tests`, `ops-load-smoke`, `ops-observability-validate`, `ops-full`.
Legacy dev alias retained for compatibility: `local-full` (preferred replacement: `ops-local-full`).

Atlas deploy profile targets:
- `make ops-deploy PROFILE=local`
- `make ops-deploy PROFILE=offline`
- `make ops-deploy PROFILE=perf`
- `make ops-undeploy`
- `make ops-redeploy PROFILE=local`

Stack service lifecycle targets:
- `ops-minio-up`, `ops-minio-down`, `ops-minio-reset`, `ops-minio-ready`, `ops-minio-bucket-check`
- `ops-prom-up`, `ops-prom-down`, `ops-prom-ready`, `ops-prom-scrape-atlas-check`
- `ops-grafana-up`, `ops-grafana-down`, `ops-grafana-ready`, `ops-grafana-datasource-check`, `ops-grafana-dashboards-check`
- `ops-otel-up`, `ops-otel-down`, `ops-otel-spans-check`
- `ops-redis-up`, `ops-redis-down`, `ops-redis-optional-check`, `ops-redis-used-check`
- `ops-toxi-up`, `ops-toxi-down`, `ops-toxi-latency-inject`, `ops-toxi-cut-store`

Validation targets:
- `ops-stack-validate`
- `ops-stack-order-check`
- `ops-stack-security-check`

Helm policy:
- Atlas deploys always use `--atomic --wait --timeout`.
- On failure, bundle includes rendered manifest and values used under `artifacts/ops/<run-id>/helm-render/`.
