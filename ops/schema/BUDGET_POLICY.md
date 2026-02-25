# Schema Budget Policy

Schema growth is capped per release window to keep governance maintainable.

## Budget

- Baseline cap: `90` governed schemas under `ops/schema/**` with suffix `.schema.json`.
- If count exceeds cap, the change must include:
  - a justification entry in `ops/schema/SCHEMA_BUDGET_EXCEPTIONS.md`
  - a migration or consolidation note

## Determinism

- Schema index artifacts are generated deterministically from sorted paths.
- Compatibility lock stays synchronized with protected schema contracts.
