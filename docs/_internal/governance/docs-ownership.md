# Documentation Ownership

- Owner: `docs-governance`
- Review cadence: `quarterly`
- Type: `policy`
- Audience: `contributor`
- Stability: `stable`
- Last verified against: `main@7dea4f4b9a65a61796b0f7ac8c2d185c0eaddb07`
- Reason to exist: define owner assignment and review cadence requirements for documentation.

## Section Owners

- `product/`: `bijux-atlas-product`
- `architecture/`: `bijux-atlas-architecture`
- `api/`: `bijux-atlas-api`
- `operations/`: `bijux-atlas-operations`
- `development/`: `bijux-atlas-developer-experience`
- `reference/`: `bijux-atlas-docs`
- `ops/governance/repository/`: `bijux-atlas-docs`

## Review Requirements

- Every docs change must be approved by the owner of the touched top-level section.
- Cross-section changes require approvals from every impacted section owner.
- Stable pages must declare an owner and a review cadence in page metadata.

## Ownership Registry

Narrative ownership policy lives in `docs/_internal/governance/docs-ownership-registry.json`.

Machine-owned section mapping for control-plane checks lives in `configs/inventory/docs-owners.json`.

## Service Levels

- Urgent correctness fixes: merged within 24 hours.
- Standard correctness fixes: merged within 72 hours.

## Enforcement Rules

- A section without a listed owner is non-compliant.
- A docs PR without section-owner approval is non-compliant.
- SLA misses require a follow-up corrective action item in governance review.
