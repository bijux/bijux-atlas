# Ops Invariants

- Owner: `bijux-atlas-operations`
- Purpose: `distill ops philosophy into enforceable invariants and decision rules`
- Consumers: `checks_ops_final_polish_contracts`
- Authority Tier: `tier2`
- Audience: `mixed`

## Core Invariants

- Single authored authority per semantic fact; duplicates are blocked by inventory and governance checks.
- Generated artifacts must be derivable, carry lineage metadata, and remain outside authored truth surfaces.
- Runtime outputs are written under `artifacts/` and never under authored `ops/` paths except `ops/_generated/`.
- Schema-backed JSON contracts must resolve to `ops/schema/**` and preserve compatibility governance.
- Release readiness requires complete evidence, deterministic generation, and explicit sign-off workflows.
- Portability and deletion safety changes must update contracts and enforcement in the same commit series.

## Decision Rules

- Prefer enforcement over prose when a policy can be checked deterministically.
- Prefer repo-relative portable paths in all contracts and inventories.
- Treat missing evidence as release-blocking unless explicitly documented and approved.

## Enforcement Links

- `checks_ops_inventory_contract_integrity`
- `checks_ops_evidence_bundle_discipline`
- `checks_ops_final_polish_contracts`
