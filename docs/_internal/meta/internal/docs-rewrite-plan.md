# Documentation Rewrite Plan

Owner: `docs-governance`  
Status: `active`  
Date: `2026-02-28`

## Scope

This plan covers consolidation controls after inventory and triage, with explicit execution decisions.

## Target Size

- Final authored page target: `150` pages.
- Hard ceiling remains `200` authored pages.

## Consolidation Clusters And Owners

1. Reader spine cluster (`docs/index.md`, `docs/start-here.md`) owner: `platform`
2. Product cluster (`docs/product/**`) owner: `product`
3. Architecture cluster (`docs/architecture/**`) owner: `architecture`
4. API cluster (`docs/api/**`) owner: `api-contracts`
5. Operations cluster (`docs/operations/**`) owner: `bijux-atlas-operations`
6. Development cluster (`docs/development/**`) owner: `platform`
7. Reference cluster (`docs/reference/**`) owner: `docs-governance`
8. Governance cluster (`docs/_internal/governance/**`, `docs/ownership.md`, `docs/style.md`) owner: `docs-governance`
9. Generated outputs cluster (`docs/_internal/generated/**`) owner: `docs-governance`
10. Draft quarantine cluster (`docs/_drafts/**`) owner: `docs-governance`

## Deletion Sequence

1. Delete legacy root mirror content under `docs/root/` after extracting required canonical facts.
2. Delete legacy onboarding entrypoints under `docs/start/` and `docs/quickstart/` after migration into `start-here` and `operations`.
3. Delete duplicated governance pages under `docs/_internal/style/` after merge into `docs/_internal/governance/`.
4. Delete duplicated operations reference tables under `docs/operations/reference/` after reference consolidation.
5. Delete obsolete historical-only migration pages once active policies no longer depend on them.

## Generated Output Freeze

- `docs/_internal/generated/**` is frozen as output-only surface.
- Policy source: `docs/_internal/governance/generated-content-policy.md`.

## `_style` Consolidation Decision

- Decision: converge governance policy pages into `docs/_internal/governance/`.
- Keep `_style` only if content is uniquely implementation-specific; otherwise delete duplicates.

## Governance Placement Decision

- Decision: `docs/_internal/governance/` remains the single canonical location for docs policy controls.
- Existing governance pages outside `docs/_internal/governance/` must be merged or removed.

## ADR Placement Decision

- Move `docs/adrs/` content to `docs/_internal/governance/adrs/`.
- `docs/_internal/governance/adrs/` becomes the canonical decision-record directory.
