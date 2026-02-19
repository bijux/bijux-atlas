# Layer Contract

- Contract version: `1.0.0`
- Compatibility policy: Minor/patch updates are backward-compatible; major updates may remove or rename fields.

## Namespaces
- `e2e`: `atlas-e2e`
- `k8s`: `atlas-e2e`
- `stack`: `atlas-e2e`

## Services
- `atlas`: service `atlas-e2e-bijux-atlas`, selector `{"app.kubernetes.io/instance": "atlas-e2e", "app.kubernetes.io/name": "bijux-atlas"}`
- `grafana`: service `grafana`, selector `{"app": "grafana"}`
- `minio`: service `minio`, selector `{"app": "minio"}`
- `otel`: service `otel-collector`, selector `{"app": "otel-collector"}`
- `prometheus`: service `prometheus`, selector `{"app": "prometheus"}`
- `redis`: service `redis`, selector `{"app": "redis"}`

## Ports
- `atlas`: `{"container": 8080, "service": 8080}`
- `grafana`: `{"container": 3000, "service": 3000}`
- `minio`: `{"api": 9000, "console": 9001}`
- `otel`: `{"grpc": 4317, "http": 4318}`
- `prometheus`: `{"container": 9090, "service": 9090}`
- `redis`: `{"container": 6379, "service": 6379}`

## Labels
- Required labels:
- `app.kubernetes.io/name`
- `app.kubernetes.io/instance`

## Release Metadata
- Required fields: `release_name, namespace, chart_name, chart_version, app_version`
- Defaults: `{"app_version": "0.1.0", "chart_name": "bijux-atlas", "chart_version": "0.1.0", "namespace": "atlas-e2e", "release_name": "atlas-e2e"}`
