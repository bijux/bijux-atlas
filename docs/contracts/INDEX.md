# Contracts SSOT

- Owner: `docs-governance`
- Stability: `stable`

## What

`docs/contracts/` is the single source of truth for machine-facing registries.

## Why

A single source prevents drift across API, CLI, server telemetry, and chart surfaces.

## Scope

Covers contract registry JSON and generated contract references.

## Non-goals

Does not define runtime implementation details outside contract surfaces.

## Contracts

- Registry files live in this directory as JSON SSOT.
- Generated documentation and code must be derived from these registries.
- Workflow details live only in `ssot-workflow.md`.

## Failure modes

Drift between SSOT and generated artifacts fails contract checks and CI gates.

## Examples

```bash
$ make ssot-check
```

Expected output: contract checks and drift checks pass with exit status 0.

## How to verify

```bash
$ make ssot-check
$ make docs-freeze
```

Expected output: both commands exit 0.

## See also

- [Contracts Index](contracts-index.md)
- [Contracts README](README.md)
- [Ingest QC Contract](qc.md)
- [Normalized Format Contract](normalized-format.md)
- [GFF3 Acceptance Contract](gff3-acceptance.md)
- [Release Diffs (Evolution)](../reference/evolution/release-diffs.md)
- [Sharding Schema](SHARDING_SCHEMA.json)
- [SSOT Workflow](ssot-workflow.md)
- [Terms Glossary](../_style/terms-glossary.md)
