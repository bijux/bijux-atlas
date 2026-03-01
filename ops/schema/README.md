# ops/schema

- Owner: `bijux-atlas-operations`
- Purpose: `schema contracts for ops authored and generated artifacts`
- Consumers: `checks_ops_schema_presence, checks_ops_domain_contract_structure`

Canonical ops schemas. Generated and validation tooling should reference this directory.

## Generated Artifacts

- `ops/schema/generated/schema-index.json`: canonical sorted schema inventory.
- `ops/schema/generated/schema-index.md`: human-readable schema index.
- `ops/schema/generated/compatibility-lock.json`: required-field compatibility lock for breaking-change detection.

## Governance Policies

- `docs/reference/ops-schema/versioning-policy.md`: schema versioning and compatibility policy.
- `docs/reference/ops-schema/budget-policy.md`: schema count growth budget policy.
- `docs/reference/ops-schema/budget-exceptions.md`: approved cap exceptions with rationale.
- `docs/reference/ops-schema/schema-reference-allowlist.md`: documented non-runtime schema references.

Placeholder extension directories tracked with `.gitkeep`: `ops/schema`, `ops/schema/generated`, `ops/schema/inventory`.
