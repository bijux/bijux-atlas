# Schema Versioning Policy

Schema artifacts under `ops/schema/` are versioned API contracts.

## Rules

- Every governed schema file name ends with `.schema.json`.
- Every governed schema includes `properties.schema_version` and lists `schema_version` in `required`.
- Backward-incompatible changes to required fields are forbidden unless `ops/schema/generated/compatibility-lock.json` is updated in the same change set.
- Compatibility lock entries are deterministic and sorted by `schema_path`.
- Schema additions must be indexed in:
  - `ops/schema/generated/schema-index.json`
  - `ops/schema/generated/schema-index.md`

## Embedded Schema Exceptions

The following are tool-native embedded schemas and are not governed by `ops/schema/**` naming rules:

- `ops/k8s/charts/bijux-atlas/values.schema.json`
- `ops/observe/drills/result.schema.json`
- `ops/observe/pack/compose.schema.json`
- `ops/inventory/policies/dev-atlas-policy.schema.json`
