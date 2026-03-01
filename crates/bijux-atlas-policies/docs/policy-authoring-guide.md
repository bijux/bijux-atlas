# Policy Authoring Guide

## Authoring Flow

1. Add or update policy data in `configs/policy/policy.json`.
2. Keep schema in `configs/policy/policy.schema.json` aligned with new fields.
3. Use stable `policy.*` violation IDs; never recycle an ID for a different rule.
4. Add table-driven tests and update golden fixtures under `tests/fixtures/evaluation/`.

## Violation IDs

Violation IDs are machine-stable keys used by automation and logs.

Rules:
- Prefix with `policy.`
- Use durable nouns and constraints, e.g. `policy.cache.disk_bytes.exceeded`
- Keep semantics stable over time

## Severity Taxonomy

- `info`: informational guidance
- `warning`: policy exceeded but service may continue
- `error`: policy must fail validation or enforcement

## Relaxing a Policy Safely

1. Adjust only the necessary budget/constraint field.
2. Preserve existing IDs and evidence format.
3. Update golden fixtures to reflect expected new outcomes.
4. If contract semantics change, bump schema version according to compatibility rules.
