# Operations Ownership

- Owner: `bijux-atlas-operations`
- Type: `policy`
- Audience: `contributor`
- Stability: `stable`
- Reason to exist: define durable ownership for operational specifications under `ops/`.

## Canonical Registry

Machine-owned section mapping for control-plane checks lives in `configs/inventory/ops-owners.json`.

## Requirements

- Every `ops/` section must map to exactly one owner group.
- Changes touching multiple `ops/` sections require approvals from each mapped owner group.
- New `ops/` sections must be added to `configs/inventory/ops-owners.json` in the same change.
