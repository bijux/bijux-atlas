# Required Files

```yaml
required_files:
  - ops/datasets/README.md
  - ops/datasets/OWNER.md
  - ops/datasets/CONTRACT.md
  - ops/datasets/INDEX.md
  - ops/datasets/REQUIRED_FILES.md
  - ops/datasets/manifest.json
  - ops/datasets/manifest.lock
  - ops/datasets/promotion-rules.json
  - ops/datasets/qc-metadata.json
  - ops/datasets/rollback-policy.json
  - ops/datasets/real-datasets.json
  - ops/datasets/generated/fixture-inventory.json
required_dirs:
  - ops/datasets/generated
  - ops/datasets/fixtures
forbidden_patterns:
  - ops/obs/
  - ops/schema/obs/
notes:
  - authored_root: ops/datasets/manifest.json
  - authored_root: ops/datasets/promotion-rules.json
  - generated_output: ops/datasets/generated/fixture-inventory.json
```
