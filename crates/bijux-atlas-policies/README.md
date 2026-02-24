# bijux-atlas-policies

Policy contracts, validation rules, and machine-readable enforcement results for atlas runtime behavior.

## Scope

This crate owns:
- `PolicySet` parsing and validation
- policy evaluation output (`PolicyViolation`)
- severity taxonomy (`PolicySeverity`)
- repository budget evaluation (`RepositoryMetrics`)

## Extend Policies

1. Add data fields in `configs/policy/policy.json` and schema updates in `configs/policy/policy.schema.json`.
2. Add or update evaluation rules with stable IDs.
3. Add table-driven tests and golden fixtures.

## Interpret Violations

Every violation includes:
- `id`: stable machine identifier
- `severity`: `info`, `warning`, or `error`
- `message`: deterministic rule summary
- `evidence`: concrete field/value context

## Docs

- `docs/POLICY_AUTHORING_GUIDE.md`
- `docs/SCHEMA.md`
- `docs/CONFIG_SCHEMA.md`
- `docs/public-api.md`
