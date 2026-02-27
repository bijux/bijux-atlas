# Docs Convergence Policy

- Owner: `bijux-atlas-operations`
- Tier: `tier2`
- Audience: `mixed`
- Source-of-truth: `docs/operations/ops-docs-contract.md`, `ops/TIER1_ROOT_SURFACE.md`, `ops/CONTRACT.md`

## Convergence Rule

If a concept exists in both `ops/` and `docs/operations/`, then:

- `ops/` is normative (Tier-0/Tier-1)
- `docs/operations/` is narrative (Tier-2)
- narrative pages must link back to the normative source

## Deletion Rule

Duplicate narrative pages must be removed after replacement links are added.

## Consumer Rule

Every Tier-2 page must identify audience and source-of-truth headers.

## Enforcement

- `checks_ops_docs_governance`

Related contracts: OPS-ROOT-023, OPS-ROOT-017.
