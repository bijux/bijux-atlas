# Ops Authority Tiers

- Owner: `bijux-atlas-operations`
- Purpose: `define authority tiers for ops documents and prevent parallel truth across machine, explanatory, and generated surfaces`
- Consumers: `checks_ops_docs_governance`
- Authority Tier: `machine`
- Audience: `contributors`

## Tiers

Reference narrative contract: `docs/operations/ops-docs-contract.md`

- `tier0-machine`: machine-readable source of truth (`ops/inventory/**`, `ops/schema/**`, structured contracts) enforced by checks or schemas.
- `tier1-normative`: minimal human-readable normative contracts in `ops/` that define rules and must be backed by machine enforcement.
- `tier2`: tutorials, walkthroughs, summaries, and workflow guides under `docs/operations/**` that explain authoritative sources and must not introduce new normative rules.
- `generated`: generated documentation artifacts derived from machine truth; never manually edited.

## Tier Rules

- Every top-level `ops/*.md` document must declare `Authority Tier` and `Audience`.
- Normative rules must live in Tier-0 machine truth and minimal Tier-1 normative docs backed by schemas/checks.
- Explanatory docs must reference machine truth and avoid standalone rule sections.
- Generated docs must be regenerated from canonical producers and must not be edited manually.

## Audience Tags

- `contributors`: authors and maintainers changing contracts/checks
- `operators`: engineers running ops workflows and interpreting evidence
- `reviewers`: approvers validating changes and release readiness
- `mixed`: documents intended for more than one audience category

## Enforcement Links

- `checks_ops_docs_governance`
- `checks_ops_domain_contract_structure`

## Authority Exceptions

- Temporary exceptions must be listed in `ops/inventory/authority-tier-exceptions.json` with `reason` and `expires_on` (YYYY-MM-DD).
