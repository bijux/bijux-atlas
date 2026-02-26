# Required Files

```yaml
required_files:
  - ops/stack/README.md
  - ops/stack/OWNER.md
  - ops/stack/CONTRACT.md
  - ops/stack/INDEX.md
  - ops/stack/REQUIRED_FILES.md
  - ops/stack/profiles.json
  - ops/stack/stack.toml
  - ops/stack/service-dependency-contract.json
  - ops/stack/evolution-policy.json
  - ops/stack/generated/version-manifest.json
  - ops/stack/generated/stack-index.json
  - ops/stack/generated/dependency-graph.json
  - ops/stack/generated/artifact-metadata.json
  - ops/stack/generated/versions.json
required_dirs:
  - ops/stack/generated
forbidden_patterns:
  - legacy-observe-alias/
  - legacy-observe-schema-alias/
notes:
  - authored_root: ops/stack/profiles.json
  - authored_root: ops/stack/service-dependency-contract.json
  - authored_root: ops/stack/evolution-policy.json
  - generated_output: ops/stack/generated/version-manifest.json
```
