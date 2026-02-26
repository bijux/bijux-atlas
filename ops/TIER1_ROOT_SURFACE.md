# Tier-1 Root Surface

- Owner: `bijux-atlas-operations`
- Purpose: `define the minimal normative Tier-1 documentation surface allowed at ops root`
- Consumers: `checks_ops_docs_governance`
- Authority Tier: `tier1-normative`
- Audience: `contributors`

## Allowed Root Tier-1 Docs

- `ops/README.md`
- `ops/CONTRACT.md`
- `ops/SSOT.md`
- `ops/ERRORS.md`
- `ops/DRIFT.md`
- `ops/NAMING.md`
- `ops/ARTIFACTS.md`
- `ops/GENERATED_LIFECYCLE.md`
- `ops/AUTHORITY_TIERS.md`
- `ops/CONTROL_PLANE.md`
- `ops/DIRECTORY_BUDGET_POLICY.md`
- `ops/DOMAIN_DOCUMENT_TEMPLATE_CONTRACT.md`

## Budgets

- `max_tier1_root_docs`: `12`

## Rules

- Top-level `ops/*.md` documents not listed here must use `Authority Tier: tier2` or `generated`.
- Tier-1 docs must remain normative and link to `docs/operations/**` for narrative workflows.

## Enforcement Links

- `checks_ops_docs_governance`
