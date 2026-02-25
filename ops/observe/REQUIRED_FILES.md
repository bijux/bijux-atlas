# Required Files

```yaml
required_files:
  - ops/observe/README.md
  - ops/observe/OWNER.md
  - ops/observe/CONTRACT.md
  - ops/observe/INDEX.md
  - ops/observe/REQUIRED_FILES.md
  - ops/observe/alert-catalog.json
  - ops/observe/slo-definitions.json
  - ops/observe/readiness.json
  - ops/observe/telemetry-drills.json
  - ops/observe/generated/telemetry-index.json
required_dirs:
  - ops/observe/dashboards
  - ops/observe/drills
  - ops/observe/rules
  - ops/observe/prom
  - ops/observe/otel
  - ops/observe/generated
forbidden_patterns:
  - legacy-observe-alias/
  - legacy-observe-schema-alias/
notes:
  - authored_root: ops/observe/alert-catalog.json
  - generated_output: ops/observe/generated/telemetry-index.json
```
