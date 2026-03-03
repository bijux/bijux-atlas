# Toolchain Reference

- Owner: `bijux-atlas-operations`
- Tier: `generated`
- Audience: `operators`
- Source-of-truth: `ops/inventory/toolchain.json`

## Tools

| Tool | Required | Probe Args |
| --- | --- | --- |
| `curl` | `true` | `--version` |
| `helm` | `true` | `version --short` |
| `k6` | `false` | `version` |
| `kind` | `true` | `--version` |
| `kubeconform` | `false` | `-v` |
| `kubectl` | `true` | `version --client --short` |

## Images

| Image Key | Reference |
| --- | --- |
| `generated_by` | `bijux dev atlas ops generate` |
| `kind_node_image` | `kindest/node:v1.31.2@sha256:f226345927d7e348497136874b6d207e0b32cc52154ad8323129352923a3142f` |
| `minio` | `minio/minio:RELEASE.2025-01-20T14-49-07Z@sha256:ed9be66eb5f2636c18289c34c3b725ddf57815f2777c77b5938543b78a44f144` |
| `minio_mc` | `minio/mc:RELEASE.2025-01-17T23-25-50Z@sha256:b55b1283c0b81b8bb473c94133d4e00a552518c4796a954ddb04bb7b6e05927d` |
| `otel_collector` | `otel/opentelemetry-collector-contrib:0.111.0@sha256:a2a52e43c1a80aa94120ad78c2db68780eb90e6d11c8db5b3ce2f6a0cc6b5029` |
| `prometheus` | `prom/prometheus:v2.54.1@sha256:f6639335d34a77d9d9db382b92eeb7fc00934be8eae81dbc03b31cfe90411a94` |
| `redis` | `redis:7.4-alpine@sha256:8b81dd37ff027bec4e516d41acfbe9fe2460070dc6d4a4570a2ac5b9d59df065` |
| `toxiproxy` | `ghcr.io/shopify/toxiproxy:2.12.0@sha256:9378ed52a28bc50edc1350f936f518f31fa95f0d15917d6eb40b8e376d1a214e` |

## GitHub Actions Pins

| Action | Ref | SHA |
| --- | --- | --- |
| `actions/cache/restore` | `v4` | `0057852bfaa89a56745cba8c7296529d2fc39830` |
| `actions/cache/save` | `v4` | `0057852bfaa89a56745cba8c7296529d2fc39830` |
| `actions/checkout` | `v4` | `34e114876b0b11c390a56381ad16ebd13914f8d5` |
| `actions/dependency-review-action` | `v4.8.3` | `05fe4576374b728f0c523d6a13d64c25081e0803` |
| `actions/upload-artifact` | `v4` | `ea165f8d65b6e75b540449e92b4886f43607fa02` |
| `dorny/paths-filter` | `v3` | `de90cc6fb38fc0963ad72b210f1f284cd68cea36` |
| `dtolnay/rust-toolchain` | `stable` | `631a55b12751854ce901bb631d5902ceb48146f7` |
| `helm/kind-action` | `v1` | `ef37e7f390d99f746eb8b610417061a60e82a6cc` |
| `peter-evans/create-pull-request` | `v6` | `c5a7806660adbe173f04e3e038b0ccdcd758773c` |
