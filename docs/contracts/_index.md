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

## Failure modes

Missing registry updates or missing regeneration will fail drift checks.

## Examples

```bash
$ ./scripts/contracts/format_contracts.py
$ ./scripts/contracts/generate_contract_artifacts.py
```

Expected output: formatted registries and regenerated contract artifacts.

## How to verify

```bash
$ ./scripts/contracts/check_all.sh
```

Expected output: all contract checks pass.

## See also

- [Contracts SSOT](README.md)
- [SSOT Workflow](SSOT_WORKFLOW.md)
- [Terms Glossary](../_style/TERMS_GLOSSARY.md)
