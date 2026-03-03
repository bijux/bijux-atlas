# Documentation Ownership

- Owner: `docs-governance`
- Review cadence: `quarterly`
- Type: `policy`
- Audience: `contributor`
- Stability: `stable`
- Last verified against: `main@240605bb1dd034f0f58f07a313d49d280f81556c`
- Reason to exist: define owner assignment and review cadence requirements for documentation.

## Section Owners

- `product/`: `bijux-atlas-product`
- `architecture/`: `bijux-atlas-architecture`
- `api/`: `bijux-atlas-api`
- `operations/`: `bijux-atlas-operations`
- `development/`: `bijux-atlas-developer-experience`
- `reference/`: `bijux-atlas-docs`
- `governance/`: `bijux-atlas-docs`

## Review Requirements

- Every docs change must be approved by the owner of the touched top-level section.
- Cross-section changes require approvals from every impacted section owner.
- Stable pages must declare an owner and a review cadence in page metadata.

## Ownership Registry

The canonical section-entry registry lives in `docs/_internal/governance/docs-ownership-registry.json`.

## Service Levels

- Urgent correctness fixes: merged within 24 hours.
- Standard correctness fixes: merged within 72 hours.

## Enforcement Rules

- A section without a listed owner is non-compliant.
- A docs PR without section-owner approval is non-compliant.
- SLA misses require a follow-up corrective action item in governance review.
