# Atlas E2E Stack

Canonical local end-to-end environment for `bijux-atlas`.

## Components

- `kind`: single-node Kubernetes cluster with fixed port mappings and resource-conscious kubelet settings.
- `minio`: S3-like artifact store required by the stack.
- `redis`: optional cache/rate-limit backend (off by default).
- `prometheus`: lightweight Prometheus for scraping `/metrics`.
- `otel`: optional OpenTelemetry collector.

## Canonical Workflow

1. Start stack:

```bash
./e2e/scripts/up.sh
```

2. Publish a dataset (ingest + catalog publish):

```bash
./e2e/scripts/publish_dataset.sh \
  --gff3 fixtures/minimal/minimal.gff3 \
  --fasta fixtures/minimal/minimal.fa \
  --fai fixtures/minimal/minimal.fa.fai \
  --release 110 --species homo_sapiens --assembly GRCh38
```

3. Tear down stack:

```bash
./e2e/scripts/down.sh
```

## Notes

- Redis and OTEL are optional. Enable them with env flags consumed by `up.sh`.
- All manifests in this directory are deterministic and intended for local repeatable testing.

## Additional Scripts

- `e2e/scripts/deploy_atlas.sh`: installs chart into `atlas-e2e` namespace.
- `e2e/scripts/warmup.sh`: runs warmup verification and checks cache-hit progression.
- `e2e/scripts/smoke_queries.sh`: executes canonical API smoke queries.
- `e2e/scripts/verify_metrics.sh`: validates critical metrics presence and activity.
- `e2e/scripts/verify_traces.sh`: optional OTEL trace verification.
- `e2e/scripts/soak.sh`: bounded soak run with memory growth guard.
- `e2e/scripts/cleanup_store.sh`: deterministic local store reset.
