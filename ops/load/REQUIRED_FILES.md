# Required Files

```yaml
required_files:
  - ops/load/README.md
  - ops/load/OWNER.md
  - ops/load/CONTRACT.md
  - ops/load/INDEX.md
  - ops/load/REQUIRED_FILES.md
  - ops/load/load.toml
  - ops/load/suites/suites.json
  - ops/load/contracts/deterministic-seed-policy.json
  - ops/load/contracts/query-pack-catalog.json
required_dirs:
  - ops/load/scenarios
  - ops/load/thresholds
  - ops/load/k6/suites
  - ops/load/k6/manifests
  - ops/load/k6/thresholds
  - ops/load/generated
forbidden_patterns:
  - ops/obs/
  - ops/schema/obs/
  - ops/load/k6/manifests/suites.json
notes:
  - authored_root: ops/load/suites/suites.json
  - authored_root: ops/load/thresholds
  - generated_output: ops/load/generated/suites.manifest.json
```
