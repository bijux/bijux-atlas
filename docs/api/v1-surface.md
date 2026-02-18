# V1 API Surface

- Owner: `api`
- Stability: `stable`

- Contract version: `v1`
- Compatibility: additive-only within v1

## Endpoints

| Method | Path | Semantics |
| --- | --- | --- |
| GET | `/debug/dataset-health` | debug-only diagnostics |
| GET | `/debug/datasets` | debug-only diagnostics |
| GET | `/debug/registry-health` | debug-only diagnostics |
| GET | `/healthz` | health/readiness signal |
| GET | `/healthz/overload` | health/readiness signal |
| GET | `/metrics` | Prometheus metrics |
| GET | `/readyz` | health/readiness signal |
| GET | `/v1/_debug/echo` | debug-only diagnostics |
| GET | `/v1/datasets` | dataset catalog/metadata |
| GET | `/v1/datasets/{release}/{species}/{assembly}` | dataset catalog/metadata |
| GET | `/v1/diff/genes` | cross-release diff |
| GET | `/v1/diff/region` | cross-release diff |
| GET | `/v1/genes` | gene query/search |
| GET | `/v1/genes/count` | gene query/search |
| GET | `/v1/genes/{gene_id}/sequence` | sequence retrieval |
| GET | `/v1/genes/{gene_id}/transcripts` | transcript retrieval |
| GET | `/v1/openapi.json` | version/control metadata |
| POST | `/v1/query/validate` | gene query/search |
| GET | `/v1/releases/{release}/species/{species}/assemblies/{assembly}` | dataset catalog/metadata |
| GET | `/v1/sequence/region` | sequence retrieval |
| GET | `/v1/transcripts/{tx_id}` | transcript retrieval |
| GET | `/v1/version` | version/control metadata |

## Source Of Truth

- `docs/contracts/endpoints.md` is authoritative for paths/params/responses.
- `configs/openapi/v1/openapi.generated.json` is generated from contract-constrained API spec.
