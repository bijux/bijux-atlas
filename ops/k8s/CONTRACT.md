# Contract

- Area: `ops/k8s`
- schema_version: `1`
- Canonical parent contract: `ops/CONTRACT.md`

## Authored vs Generated

| Path | Role |
| --- | --- |
| `ops/k8s/charts/bijux-atlas/Chart.yaml` | Authored chart metadata |
| `ops/k8s/charts/bijux-atlas/values.yaml` | Authored chart defaults |
| `ops/k8s/install-matrix.json` | Authored install profile matrix |
| `ops/k8s/values/kind.yaml` | Authored kind profile overrides |
| `ops/k8s/values/dev.yaml` | Authored dev profile overrides |
| `ops/k8s/values/ci.yaml` | Authored CI profile overrides |
| `ops/k8s/values/prod.yaml` | Authored prod profile overrides |
| `ops/k8s/generated/render-artifact-index.json` | Generated render artifact index |
| `ops/k8s/generated/inventory-index.json` | Generated k8s inventory index |
| `ops/k8s/generated/release-snapshot.json` | Generated release snapshot |

## Schema References

| Artifact | Schema |
| --- | --- |
| `ops/k8s/install-matrix.json` | `ops/schema/k8s/install-matrix.schema.json` |
| `ops/k8s/tests/suites.json` | `ops/schema/k8s/suite-report.schema.json` |
| `ops/k8s/generated/render-artifact-index.json` | `ops/schema/k8s/render-artifact-index.schema.json` |
| `ops/k8s/generated/inventory-index.json` | `ops/schema/k8s/inventory-index.schema.json` |
| `ops/k8s/generated/release-snapshot.json` | `ops/schema/k8s/release-snapshot.schema.json` |

## Invariants

- Helm chart content under `ops/k8s/charts/` is specification-only and does not execute behavior.
- Install matrix profile names are unique, deterministic, and lexicographically sorted.
- Canonical values profiles include `kind`, `dev`, `ci`, and `prod`.
- Render determinism is required: the same chart and values inputs must produce identical rendered manifests.
- Release snapshot and render artifact index are generated outputs and must never be hand-edited.
- CRD compatibility policy and rollout/rollback expectations remain backward-compatible for promoted versions.
