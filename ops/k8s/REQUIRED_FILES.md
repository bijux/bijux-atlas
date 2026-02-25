# Required Files

```yaml
required_files:
  - ops/k8s/README.md
  - ops/k8s/OWNER.md
  - ops/k8s/CONTRACT.md
  - ops/k8s/INDEX.md
  - ops/k8s/REQUIRED_FILES.md
  - ops/k8s/install-matrix.json
  - ops/k8s/tests/suites.json
  - ops/k8s/tests/manifest.json
  - ops/k8s/values/kind.yaml
  - ops/k8s/values/dev.yaml
  - ops/k8s/values/ci.yaml
  - ops/k8s/values/prod.yaml
required_dirs:
  - ops/k8s/generated
forbidden_patterns:
  - ops/obs/
  - ops/schema/obs/
notes:
  - authored_root: ops/k8s/install-matrix.json
  - generated_output: ops/k8s/generated/release-snapshot.json
```
