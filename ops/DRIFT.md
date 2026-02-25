# Ops Drift Policy

This page lists drift categories blocked by contract checks.

## Drift Types

- Pin drift: stack/runtime uses values not rooted in inventory pin sources.
- Suite drift: load or e2e suite references missing or mismatched manifests.
- Schema drift: schema index or compatibility lock does not match schema tree.
- Inventory drift: contracts, surfaces, owners, and gates do not align with runtime surfaces.
- Document drift: domain reference pages point to removed paths or orphan docs.

## Enforcement

- `bijux dev atlas ops doctor`
- `bijux dev atlas ops validate`

Any drift category above is release-blocking for v0.1.0.
