# Contract

- Area: `ops/schema`
- schema_version: `1`
- Canonical parent contract: `ops/CONTRACT.md`

## Authored vs Generated

| Path | Role |
| --- | --- |
| `ops/schema/**/**.schema.json` | Authored schema contracts |
| `ops/schema/VERSIONING_POLICY.md` | Authored schema evolution policy |
| `ops/schema/BUDGET_POLICY.md` | Authored schema growth policy |
| `ops/schema/SCHEMA_REFERENCE_ALLOWLIST.md` | Authored schema locality allowlist |
| `ops/schema/generated/schema-index.json` | Generated schema registry index |
| `ops/schema/generated/schema-index.md` | Generated schema registry document |
| `ops/schema/generated/compatibility-lock.json` | Generated compatibility lock |

## Schema References

| Artifact | Schema |
| --- | --- |
| `ops/schema/generated/schema-index.json` | `ops/schema/meta/schema-index.schema.json` |
| `ops/schema/generated/compatibility-lock.json` | `ops/schema/meta/compatibility-lock.schema.json` |

## Invariants

- No duplicate authored truth is allowed; schema definitions are authored once under `ops/schema/**`.
- Schema references must resolve only to `ops/schema/**`.
- Behavior source is forbidden in `ops/schema`; content is contract-only.
- The semantic domain name `obs` is prohibited; only canonical `observe` naming is valid.
- Generated schema artifacts must include `generated_by` and `schema_version` metadata.
- Schema docs and policy files must be linked from `ops/schema/INDEX.md` or top-level `ops/INDEX.md`.
- Compatibility lock and schema index generation must be deterministic for identical schema inputs.
- Orphan schema files are forbidden unless explicitly allowlisted as library schemas.

## Enforcement Links

- `checks_ops_schema_presence`
- `checks_ops_required_files_contracts`
- `checks_ops_domain_contract_structure`
