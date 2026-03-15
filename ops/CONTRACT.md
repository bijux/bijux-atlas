# Ops Contract

## Scope

- Governed surface: `ops/` and this document.
- Source of truth for operational metadata: `ops/inventory/contracts-map.json`, `ops/inventory/authority-index.json`, and `ops/inventory/surfaces.json`.
- Current validation entrypoint: `bijux-dev-atlas ops validate --format json`.
- Current focused entrypoints: `bijux-dev-atlas ops profiles ...`, `bijux-dev-atlas ops render ...`, `bijux-dev-atlas ops install ...`, and `bijux-dev-atlas ops stack ...`.
- This document describes boundaries and durable invariants. It does not replace machine validation output.

## Durable Rules

- Authored truth lives under domain directories and `ops/inventory/`; generated examples live under `ops/_generated.example/`.
- Inventory files describe authorities, consumers, schemas, and command surfaces. They must not claim commands that the CLI does not expose.
- Schema files under `ops/schema/` validate operational inputs and generated reports. They must match current file paths and current runtime producers.
- Effectful operations require explicit opt-in flags such as `--allow-subprocess`, `--allow-network`, or `--allow-write`.
- Human walkthroughs belong in `docs/operations/`; `ops/` stores operational data, contracts, inventories, schemas, fixtures, and generated examples.

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

## Evidence

- Whole-tree validation report: `bijux-dev-atlas ops validate --format json`
- Profile validation report: `bijux-dev-atlas ops profiles validate --allow-subprocess --format json`
- Generated example registry snapshot: `ops/_generated.example/contracts-registry-snapshot.json`
- Generated example inventory index: `ops/_generated.example/inventory-index.json`
