# Contracts Index

- Owner: `docs-governance`

## What

Index of contract registries and generated outputs.

## Why

Provides one navigation page for all contract surfaces and their stability levels.

## Scope

Includes JSON registries under `docs/reference/contracts/schemas/*.json` and generated contract references.

## Non-goals

Does not replace registry-specific detail pages.

## Contracts

| Registry | SSOT file | Generated docs/code | Stability |
| --- | --- | --- | --- |
| Error codes | `schemas/ERROR_CODES.json` | `errors.md`, API/core generated Rust | Frozen |
| Error envelope schema | `schemas/ERROR_SCHEMA.json` | `errors.md`, envelope validation checks | Stable |
| Error status map | `schemas/ERROR_STATUS_MAP.json` | `errors.md`, status-code validation checks | Stable |
| Metrics | `schemas/METRICS.json` | `metrics.md`, server telemetry constants | Additive |
| Trace spans | `schemas/TRACE_SPANS.json` | `tracing.md`, server span constants | Additive |
| Endpoints | `schemas/ENDPOINTS.json` | `endpoints.md`, OpenAPI drift checks | Additive |
| Config keys | `schemas/CONFIG_KEYS.json` | `config-keys.md`, config-key checks | Additive |
| Chart values | `schemas/CHART_VALUES.json` | `chart-values.md`, chart values checks | Additive |
| Artifact schema | `schemas/ARTIFACT_SCHEMA.json` | `artifacts/manifest-contract.md` | Experimental |
| Policy schema | `schemas/POLICY_SCHEMA.json` | `../../governance/index.md`, policy checks | Experimental |
| QC schema | `schemas/QC_SCHEMA.json` | `qc.md` | Stable |
| Normalized format schema | `schemas/NORMALIZED_FORMAT_SCHEMA.json` | `normalized-format.md` | Evolving |
| Release diff schema | `schemas/DIFF_SCHEMA.json` | `contract-diff.md` | Additive |
| Sharding schema | `schemas/SHARDING_SCHEMA.json` | `../configs.md` | Additive |
| Cursor schema | `schemas/CURSOR_SCHEMA.json` | `../schemas.md` | Additive |
| GC policy schema | `schemas/GC_POLICY.json` | `../../operations/retention-and-gc.md` | Additive |

## Canonical references

- [Error Codes](errors.md)
- [Endpoints](endpoints.md)
- [Metrics](metrics.md)
- [Trace Spans](tracing.md)
- [Config Keys](config-keys.md)
- [Chart Values](chart-values.md)
- [Artifact Contracts](artifacts/index.md)
- [Schema Index](../schemas.md)

Note: generated contract artifacts are validated in CI and may not be committed as markdown pages.

## Failure modes

Missing registry updates or missing regeneration will fail drift checks.

## Examples

```bash
$ make contracts
```

Expected output: formatted registries and regenerated contract artifacts.

## How to verify

```bash
$ make contracts
```

Expected output: all contract checks pass.

## See also

- [Contracts](index.md)
- [SSOT Workflow](ssot-workflow.md)
