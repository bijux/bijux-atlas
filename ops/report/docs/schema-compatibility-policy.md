# Schema Compatibility Policy

- Owner: bijux-atlas-operations
- Stability: stable

## Scope

Applies to schemas under `ops/schema/` that define persisted or contract artifacts.

## Rules

- Schemas are versioned APIs.
- Additive changes are preferred for minor evolution.
- Removing required fields is a breaking change.
- Breaking changes require:
  - schema version bump,
  - migration notes,
  - updated compatibility lock.

## Enforcement

- `ops doctor` validates schema-version fields in key schemas.
- `ops doctor` validates schema drift against generated schema index.
- `ops doctor` validates required-field compatibility lock for breaking change detection.
