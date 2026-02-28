# Endpoints Contract

- Owner: `docs-governance`

## What

Defines the `Endpoints Contract` registry contract.

## Why

Prevents drift between SSOT JSON, generated code, and operational consumers.

## Scope

Applies to producers and consumers of this registry.

## Non-goals

Does not define implementation internals outside this contract surface.

## Contracts

- `GET /debug/dataset-health` telemetry class: `debug`
- `GET /debug/datasets` telemetry class: `debug`
- `GET /debug/registry-health` telemetry class: `debug`
- `GET /healthz` telemetry class: `health`
- `GET /healthz/overload` telemetry class: `health`
- `GET /metrics` telemetry class: `metrics`
- `GET /readyz` telemetry class: `health`
- `GET /v1/_debug/echo` telemetry class: `debug`
- `GET /v1/datasets` telemetry class: `catalog`
- `GET /v1/datasets/{release}/{species}/{assembly}` telemetry class: `catalog`
- `GET /v1/diff/genes` telemetry class: `diff`
- `GET /v1/diff/region` telemetry class: `diff`
- `GET /v1/genes` telemetry class: `query`
- `GET /v1/genes/count` telemetry class: `query`
- `GET /v1/genes/{gene_id}/sequence` telemetry class: `sequence`
- `GET /v1/genes/{gene_id}/transcripts` telemetry class: `transcript`
- `GET /v1/openapi.json` telemetry class: `control`
- `POST /v1/query/validate` telemetry class: `query`
- `GET /v1/releases/{release}/species/{species}/assemblies/{assembly}` telemetry class: `catalog`
- `GET /v1/sequence/region` telemetry class: `sequence`
- `GET /v1/trantooling/{tx_id}` telemetry class: `transcript`
- `GET /v1/version` telemetry class: `control`

OpenAPI reference:
- `configs/openapi/v1/openapi.yaml`

## Failure modes

Invalid or drifted registry content is rejected by contract checks and CI gates.

## Examples

```bash
$ bijux dev atlas contracts generate --generators openapi
```

Expected output: generator succeeds and endpoint drift checks pass.

## How to verify

```bash
$ make contracts
$ make contracts-docs
```

Expected output: both commands exit status 0 and print contract generation/check success.

## See also

- [Contracts Index](index.md)
- [SSOT Workflow](../../governance/contract-ssot-workflow.md)
- [Terms Glossary](../../glossary.md)
