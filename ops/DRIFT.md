# Ops Drift Policy

- Authority Tier: `machine`
- Audience: `contributors`
This page lists drift categories blocked by contract checks.

## Drift Types

- Pin drift: stack/runtime uses values not rooted in inventory pin sources.
- Suite drift: load or e2e suite references missing or mismatched manifests.
- Schema drift: schema index or compatibility lock does not match schema tree.
- Inventory drift: contracts, surfaces, owners, and gates do not align with runtime surfaces.
- Document drift: domain reference pages point to removed paths or orphan docs.
- Deletion drift: files removed without authority-index/contracts-map updates leave unresolved consumers.

## Enforcement

- `bijux dev atlas ops doctor`
- `bijux dev atlas ops validate`

Any drift category above is release-blocking for v0.1.0.

Deletion safety rule: remove an ops artifact only when its consumer mapping, authority index entry, and contracts-map reference are updated in the same change.
