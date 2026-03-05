# Dashboard Provisioning Instructions

- Owner: `bijux-atlas-operations`
- Type: `runbook`
- Audience: `operator`
- Stability: `stable`

## Kubernetes Provisioning

1. Apply `ops/observe/pack/k8s/grafana-config.yaml`.
2. Apply `ops/observe/pack/k8s/grafana.yaml`.
3. Confirm ConfigMaps mount under `/var/lib/grafana/dashboards`.

## Docker Compose Provisioning

1. Start `ops/observe/pack/compose/docker-compose.yml`.
2. Mount dashboard JSON files under Grafana dashboards path.
3. Reload Grafana dashboard provisioning.
