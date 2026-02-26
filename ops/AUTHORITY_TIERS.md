# Ops Authority Tiers

- Owner: `bijux-atlas-operations`
- Purpose: `define authority tiers for ops documents and prevent parallel truth across machine, explanatory, and generated surfaces`
- Consumers: `checks_ops_docs_governance`
- Authority Tier: `machine`
- Audience: `contributors`

## Tiers

- `machine`: normative contract or policy documents that may define rules and are expected to be enforced by checks or schemas.
- `explanatory`: tutorials, walkthroughs, summaries, and workflow guides that explain machine truth but must not introduce new normative rules.
- `generated`: generated documentation artifacts derived from machine truth; never manually edited.

## Tier Rules

- Every top-level `ops/*.md` document must declare `Authority Tier` and `Audience`.
- Normative rules must live in machine-tier docs or schemas/checks.
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
