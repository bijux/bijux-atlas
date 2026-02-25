# Kind Cluster Profile

Canonical kind substrate for local ops workflows.

## Why

Provides deterministic Kubernetes behavior for `make ops-*` flows.

## Node image

Pinned in `ops/stack/kind/cluster.yaml`:
- `kindest/node:v1.31.2@sha256:f226345927d7e348497136874b6d207e0b32cc52154ad8323129352923a3142f`

## Profiles

Set `ATLAS_KIND_PROFILE` to one of:
- `minimal`: minimum local footprint (smoke/dev)
- `small`: minimum local footprint (smoke/dev)
- `normal`: default profile
- `perf`: larger kubelet pod budget for load/perf checks

Config files:
- `cluster-small.yaml`
- `cluster-dev.yaml`
- `cluster.yaml` (normal)
- `cluster-perf.yaml`

## Deterministic ports

- Atlas HTTP: `http://127.0.0.1:18080`
- Atlas HTTPS: `https://127.0.0.1:18443`
- Prometheus: `http://127.0.0.1:19090`

## Resource budget (recommended)

- `small`: >= 4 CPU, 8 GiB RAM
- `normal`: >= 6 CPU, 12 GiB RAM
- `perf`: >= 8 CPU, 16 GiB RAM

## Commands

```bash
make ops-kind-up
make ops-kind-down
make ops-kind-reset
make ops-kind-registry-up
```

Canonical docs: `ops/README.md`, `docs/operations/INDEX.md`.
