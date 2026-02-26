# Docs Structure Freeze

- Owner: `bijux-atlas-operations`
- Tier: `tier2`
- Audience: `contributors`
- Source-of-truth: `docs/operations/ops-docs-contract.md`, `docs/operations/DOCS_CONVERGENCE_POLICY.md`

## Version

- `v0.1`

## Frozen Structure

- `docs/operations/reference/` for generated references from SSOT
- `docs/operations/runbooks/` for operator runbooks
- domain narrative subtrees (`k8s`, `observability`, `load`, `e2e`, `security`, `slo`)

## Change Control

- Structure changes require updating this document and `docs/operations/INDEX.md`.
- New narrative pages must include Tier/Audience/Source-of-truth headers.

## Enforcement

- `checks_ops_docs_governance`
