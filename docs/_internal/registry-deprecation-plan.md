# Registry Deprecation Plan

Owner: `platform`

## Purpose

Keep one governance object model while retiring legacy registries safely.

## Rules

- A domain registry can be deprecated only after its records are emitted into governance objects.
- A registry stays readable until downstream consumers switch to `bijux dev atlas governance list` and `governance explain`.
- Deletions are allowed only after one release cycle with zero reads in CI and local workflows.

## Adapter Policy

- Adapter mapping is declared in `governance/domain-registry-map.json`.
- Deprecated source paths remain listed in that map until deletion is complete.
- Governance validation fails if a mapped registry path disappears before replacement coverage exists.

## Exit Criteria

- Every stable domain object has authority source, owner, reviewed date, and evidence.
- Governance coverage stays at or above baseline in `governance/coverage-baseline.json`.
- No forbidden `registry.json` files exist outside approved paths.
