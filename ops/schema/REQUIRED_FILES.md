# Required Files

```yaml
required_files:
  - ops/schema/README.md
  - ops/schema/OWNER.md
  - ops/schema/REQUIRED_FILES.md
  - ops/schema/VERSIONING_POLICY.md
  - ops/schema/BUDGET_POLICY.md
  - ops/schema/SCHEMA_BUDGET_EXCEPTIONS.md
  - ops/schema/SCHEMA_REFERENCE_ALLOWLIST.md
  - ops/schema/generated/schema-index.json
  - ops/schema/generated/schema-index.md
  - ops/schema/generated/compatibility-lock.json
required_dirs:
  - ops/schema/inventory
  - ops/schema/env
  - ops/schema/k8s
  - ops/schema/load
  - ops/schema/observe
  - ops/schema/e2e
  - ops/schema/datasets
  - ops/schema/report
  - ops/schema/stack
  - ops/schema/configs
  - ops/schema/generated
forbidden_patterns:
  - legacy-observe-alias/
  - legacy-observe-schema-alias/
notes:
  - authored_root: ops/schema/inventory/pins.schema.json
  - generated_output: ops/schema/generated/schema-index.json
```
