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
| Metrics | `METRICS.json` | `metrics.md`, server telemetry constants | Additive |
| Trace spans | `TRACE_SPANS.json` | `tracing.md`, server span constants | Additive |
| Endpoints | `ENDPOINTS.json` | `endpoints.md`, OpenAPI drift checks | Additive |
| Config keys | `CONFIG_KEYS.json` | `config-keys.md`, config-key checks | Additive |
| Chart values | `CHART_VALUES.json` | `chart-values.md`, chart values checks | Additive |
| Artifact schema | `artifacts/ARTIFACT_SCHEMA.json` | `_generated/contracts/ARTIFACT_SCHEMA.md` | Experimental |
| Policy schema | `POLICY_SCHEMA.json` | `_generated/contracts/POLICY_SCHEMA.md` | Experimental |
| QC schema | `QC_SCHEMA.json` | `_generated/contracts/QC_SCHEMA.md` | Stable |
| Normalized format schema | `NORMALIZED_FORMAT_SCHEMA.json` | `_generated/contracts/NORMALIZED_FORMAT_SCHEMA.md` | Evolving |
| Release diff schema | `DIFF_SCHEMA.json` | `_generated/contracts/DIFF_SCHEMA.md` | Additive |
| Sharding schema | `SHARDING_SCHEMA.json` | `_generated/contracts/SHARDING_SCHEMA.md` | Additive |
| Cursor schema | `CURSOR_SCHEMA.json` | `_generated/contracts/CURSOR_SCHEMA.md` | Additive |
| GC policy schema | `GC_POLICY.json` | `operations/retention-and-gc.md` | Additive |

## Generated docs

- [_generated/contracts/ERROR_CODES.md](../_generated/contracts/ERROR_CODES.md)
- [_generated/contracts/ENDPOINTS.md](../_generated/contracts/ENDPOINTS.md)
- [_generated/contracts/METRICS.md](../_generated/contracts/METRICS.md)
- [_generated/contracts/TRACE_SPANS.md](../_generated/contracts/TRACE_SPANS.md)
- [_generated/contracts/CONFIG_KEYS.md](../_generated/contracts/CONFIG_KEYS.md)
- [_generated/contracts/CHART_VALUES.md](../_generated/contracts/CHART_VALUES.md)
- [_generated/contracts/ARTIFACT_SCHEMA.md](../_generated/contracts/ARTIFACT_SCHEMA.md)
- [_generated/contracts/POLICY_SCHEMA.md](../_generated/contracts/POLICY_SCHEMA.md)
- [_generated/contracts/QC_SCHEMA.md](../_generated/contracts/QC_SCHEMA.md)
- [_generated/contracts/NORMALIZED_FORMAT_SCHEMA.md](../_generated/contracts/NORMALIZED_FORMAT_SCHEMA.md)
- [_generated/contracts/DIFF_SCHEMA.md](../_generated/contracts/DIFF_SCHEMA.md)
- [_generated/contracts/SHARDING_SCHEMA.md](../_generated/contracts/SHARDING_SCHEMA.md)
- [_generated/contracts/CURSOR_SCHEMA.md](../_generated/contracts/CURSOR_SCHEMA.md)

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
