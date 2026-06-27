# Ops Contract

## Scope

- Governed surface: `ops/` and this document.
- Source of truth for operational metadata: `ops/inventory/contracts-map.json`, `ops/inventory/authority-index.json`, and `ops/inventory/surfaces.json`.
- Current validation entrypoint: `bijux-atlas-dev ops validate --format json`.
- Current focused entrypoints: `bijux-atlas-dev ops profiles ...`, `bijux-atlas-dev ops render ...`, `bijux-atlas-dev ops install ...`, and `bijux-atlas-dev ops stack ...`.
- This document describes boundaries and durable invariants. It does not replace machine validation output.

## Durable Rules

- Authored truth lives under domain directories and `ops/inventory/`; generated examples live under `ops/_generated.example/`.
- Inventory files describe authorities, consumers, schemas, and command surfaces. They must not claim commands that the CLI does not expose.
- Schema files under `ops/schema/` validate operational inputs and generated reports. `ops/schema/generated/schema-index.json` is the authoritative schema index.
- Effectful operations require explicit opt-in flags such as `--allow-subprocess`, `--allow-network`, or `--allow-write`.
- Human walkthroughs belong in `docs/04-operations/`, `docs/06-development/`, and `docs/07-reference/`; `ops/` stores operational data, contracts, inventories, schemas, fixtures, and generated examples.
- Markdown inside `ops/` is limited to five root documents. Deep directories must stay boring and machine-readable.
- Release readiness and runbook generation are proved by data authorities, not by nested prose files.

## Machine Authorities

| Concern | Authority |
| --- | --- |
| Contract catalog | `ops/inventory/contracts.json` |
| Contract source mapping | `ops/inventory/contracts-map.json` |
| Command-to-gate mapping | `ops/inventory/contract-gate-map.json` |
| Operational command surface | `ops/inventory/surfaces.json` |
| Inventory authority hierarchy | `ops/inventory/authority-index.json` |
| Authoritative path list | `ops/inventory/authoritative-file-list.json` |
| Schema coverage | `ops/schema/generated/schema-index.json` |
| Breaking release notes | `ops/release/notes/breaking.json` |

## Evidence

- Whole-tree validation report: `bijux-atlas-dev ops validate --format json`
- Profile validation report: `bijux-atlas-dev ops profiles validate --allow-subprocess --format json`
- Generated example registry snapshot: `ops/_generated.example/contracts-registry-snapshot.json`
- Generated example inventory index: `ops/_generated.example/inventory-index.json`

## Minimal Release Surface

- `ops/inventory/contracts-map.json`
- `ops/inventory/authority-index.json`
- `ops/load/suites/suites.json`
- `ops/observe/drills.json`
- `ops/report/generated/readiness-score.json`

Removing or renaming any of these files is a release-surface change and must update the same commit's inventories and validators.
