# Required Files

```yaml
required_files:
  - ops/e2e/README.md
  - ops/e2e/OWNER.md
  - ops/e2e/CONTRACT.md
  - ops/e2e/INDEX.md
  - ops/e2e/REQUIRED_FILES.md
  - ops/e2e/suites/suites.json
  - ops/e2e/scenarios/scenarios.json
  - ops/e2e/expectations/expectations.json
  - ops/e2e/fixtures/allowlist.json
  - ops/e2e/reproducibility-policy.json
  - ops/e2e/taxonomy.json
required_dirs:
  - ops/e2e/generated
  - ops/e2e/datasets
  - ops/e2e/expectations
  - ops/e2e/manifests
forbidden_patterns:
  - legacy-observe-alias/
  - legacy-observe-schema-alias/
notes:
  - authored_root: ops/e2e/suites/suites.json
  - authored_root: ops/e2e/scenarios/scenarios.json
  - generated_output: ops/e2e/generated/e2e-summary.json
```
