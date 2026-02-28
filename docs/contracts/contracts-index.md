# Contracts Index

- Owner: `docs-governance`

## What

Index of contract registries and generated outputs.

## Why

Provides one navigation page for all contract surfaces and their stability levels.

## Scope

Includes JSON registries under `docs/contracts/*.json` and generated contract references.

## Non-goals

Does not replace registry-specific detail pages.

## Contracts

| Registry | SSOT file | Generated docs/code | Stability |
| --- | --- | --- | --- |
| Error codes | `ERROR_CODES.json` | `errors.md`, API/core generated Rust | Frozen |
| Error envelope schema | `ERROR_SCHEMA.json` | `errors.md`, envelope validation checks | Stable |
| Error status map | `ERROR_STATUS_MAP.json` | `errors.md`, status-code validation checks | Stable |
| Metrics | `METRICS.json` | `metrics.md`, server telemetry constants | Additive |
| Trace spans | `TRACE_SPANS.json` | `tracing.md`, server span constants | Additive |
| Endpoints | `ENDPOINTS.json` | `endpoints.md`, OpenAPI drift checks | Additive |
| Config keys | `CONFIG_KEYS.json` | `config-keys.md`, config-key checks | Additive |
| Chart values | `CHART_VALUES.json` | `chart-values.md`, chart values checks | Additive |
| Artifact schema | `artifacts/ARTIFACT_SCHEMA.json` | `artifacts/manifest-contract.md` | Experimental |
| Policy schema | `POLICY_SCHEMA.json` | `../governance/index.md`, policy checks | Experimental |
| QC schema | `QC_SCHEMA.json` | `qc.md` | Stable |
| Normalized format schema | `NORMALIZED_FORMAT_SCHEMA.json` | `normalized-format.md` | Evolving |
| Release diff schema | `DIFF_SCHEMA.json` | `contract-diff.md` | Additive |
| Sharding schema | `SHARDING_SCHEMA.json` | `../reference/configs.md` | Additive |
| Cursor schema | `CURSOR_SCHEMA.json` | `../reference/schemas.md` | Additive |
| GC policy schema | `GC_POLICY.json` | `operations/retention-and-gc.md` | Additive |

## Canonical references

- [Error Codes](errors.md)
- [Endpoints](endpoints.md)
- [Metrics](metrics.md)
- [Trace Spans](tracing.md)
- [Config Keys](config-keys.md)
- [Chart Values](chart-values.md)
- [Artifact Contracts](artifacts/INDEX.md)
- [Schema Index](../reference/schemas.md)

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

- [Contracts SSOT](INDEX.md)
- [SSOT Workflow](ssot-workflow.md)
- [Terms Glossary](../_style/terms-glossary.md)
