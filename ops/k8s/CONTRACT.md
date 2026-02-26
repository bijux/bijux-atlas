# Contract

- Area: `ops/k8s`
- schema_version: `1`
- contract_version: `1.0.0`
- contract_taxonomy: `hybrid`
- Canonical parent contract: `ops/CONTRACT.md`
- Owner: `bijux-atlas-operations`
- Purpose: `kubernetes delivery contracts and render/install validation surfaces`
- Consumers: `checks_ops_inventory_contract_integrity, checks_ops_domain_contract_structure`

## Authored vs Generated

| Path | Role |
| --- | --- |
| `ops/k8s/charts/bijux-atlas/Chart.yaml` | Authored chart metadata |
| `ops/k8s/charts/bijux-atlas/values.yaml` | Authored chart defaults |
| `ops/k8s/install-matrix.json` | Authored install profile matrix |
| `ops/k8s/rollout-safety-contract.json` | Authored rollout and warmup safety policy |
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
| `ops/k8s/rollout-safety-contract.json` | `ops/schema/k8s/rollout-safety-contract.schema.json` |
| `ops/k8s/tests/suites.json` | `ops/schema/k8s/suite-report.schema.json` |
| `ops/k8s/generated/render-artifact-index.json` | `ops/schema/k8s/render-artifact-index.schema.json` |
| `ops/k8s/generated/inventory-index.json` | `ops/schema/k8s/inventory-index.schema.json` |
| `ops/k8s/generated/release-snapshot.json` | `ops/schema/k8s/release-snapshot.schema.json` |

## Contract Taxonomy

- Structural contract: chart metadata, values overlays, and install matrix define stable delivery inputs.
- Behavioral contract: render determinism and release snapshot outputs define rollout/runtime behavior expectations.

## Invariants

- No duplicate authored truth is allowed; k8s authored inputs are constrained to chart, values, and install matrix paths.
- Schema references for this domain must resolve only to `ops/schema/**`.
- Behavior source is forbidden in `ops/k8s`; execution logic remains outside `ops/`.
- The semantic domain name `obs` is prohibited; only canonical `observe` naming is valid.
- Generated k8s artifacts must include `generated_by` and `schema_version` metadata.
- K8s docs must be linked from `ops/k8s/INDEX.md`; orphan docs are forbidden.
- Render determinism is mandatory: same chart plus same values must produce identical manifests.
- Rollout safety contract must align with install matrix profile names, values files, and suite assignments.
- Release snapshot and inventory index generation must be deterministic for identical authored inputs.

## Runtime Evidence Mapping

- Render index evidence: `ops/k8s/generated/render-artifact-index.json`
- Release snapshot evidence: `ops/k8s/generated/release-snapshot.json`
- Conformance evidence: `ops/k8s/generated/conformance-report.json`

## Enforcement Links

- `checks_ops_domain_contract_structure`
- `checks_ops_required_files_contracts`
